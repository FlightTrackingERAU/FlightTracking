use super::*;
use crate::{TileId, WorldViewport};

use simple_moving_average::SMA;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender, UnboundedReceiver, UnboundedSender};

use intmap::IntMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct MemoryTile {
    pub id: TileId,
    pub image: Option<image::RgbaImage>,
}

/// Holds multiple levels of cache for requesting tiles in a generic manner.
/// Handles preemption and de-duplicating tile requests so that only one is sent out
pub struct TilePipeline {
    /// The list of backends in use by this pipeline. Lower indices are queried first
    backends: Arc<Vec<Box<dyn Backend>>>,

    /// The cache of tiles on the GPU
    cache: IntMap<CachedTile>,
    upload_rx: Receiver<MemoryTile>,
    request_tx: Arc<UnboundedSender<TileId>>,
    tile_size: AtomicU32,
}

#[derive(Debug, Copy, Clone)]
enum CachedTile {
    NotAvailable,
    Pending,
    Cached(conrod_core::image::Id),
}

impl TilePipeline {
    /// Creates a new `TilePipeline` with the given backends.
    ///
    /// Uses `runtime` to spawn required asynchronous background tasks
    pub fn new(backends: Vec<Box<dyn Backend>>, runtime: &Runtime) -> Self {
        //Use large initial size here because we will have a few hundred tiles on the GPU at
        //minimum, and rehashing is EXPENSIVE
        let (upload_tx, upload_rx) = tokio::sync::mpsc::channel(24);
        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel();

        let backends = Arc::new(backends);
        runtime.spawn(tile_requester(upload_tx, request_rx, backends.clone()));
        Self {
            cache: IntMap::with_capacity(1024),
            upload_rx,
            request_tx: Arc::new(request_tx),
            backends,
            tile_size: AtomicU32::new(0),
        }
    }

    /// Fetches the image id of `tile`, or starts loading the texture,
    /// returning None on this frame and subsequent frames until the asynchronous request finishes
    pub fn get_tile(&mut self, tile: TileId) -> Option<conrod_core::image::Id> {
        //TODO: Have the caller pass the lock in so that we dont lock, unlock, then lock again
        {
            match self.cache.get(tile_coord_to_u64(tile)) {
                Some(&CachedTile::Cached(id)) => {
                    return Some(id);
                }
                Some(&CachedTile::NotAvailable) => return None,
                Some(&CachedTile::Pending) => return None,
                None => {}
            };
        }
        assert!(
            self.request_tx.send(tile).is_ok(),
            "Tile request channel closed! Cannot fetch more tiles"
        );

        self.cache
            .insert(tile_coord_to_u64(tile), CachedTile::Pending);
        None
    }

    /// Returns the size of tiles returned by this pipeline, or `None` or unknown
    pub fn tile_size(&self) -> Option<u32> {
        let cached_size = self.tile_size.load(Ordering::Relaxed);
        if cached_size != 0 {
            return Some(cached_size);
        }

        for backend in self.backends.iter() {
            if let Some(size) = backend.tile_size() {
                println!("Backend {} gave size: {}", backend.name(), size);
                self.tile_size.store(size, Ordering::Relaxed);
                return Some(size);
            }
        }
        None
    }

    /// Called each frame to allow the pipeline to upload newly fetched tiles to the GPU.
    ///
    /// `viewport`: The viewport of the currently rendered scene. This is used for preemption
    pub fn update(
        &mut self,
        _viewport: &WorldViewport,
        display: &glium::Display,
        image_map: &mut conrod_core::image::Map<glium::Texture2d>,
    ) {
        //TODO: Pass viewport to preemption code
        const MAX_PROCESS_TIME: Duration = Duration::from_millis(50);
        let start = std::time::Instant::now();
        let mut tiles_processed = 0;

        while let Ok(tile) = self.upload_rx.try_recv() {
            let time_spent = start.elapsed();
            if time_spent > MAX_PROCESS_TIME {
                println!(
                    "Breaking from process loop after {} ms. Processed {} tiles",
                    time_spent.as_micros() as f64 / 1000.0,
                    tiles_processed
                );
                break;
            }
            let tile_id = tile.id;

            match tile.image {
                None => {
                    let _ = self
                        .cache
                        .insert(tile_coord_to_u64(tile_id), CachedTile::NotAvailable);
                }
                Some(image) => {
                    let texture = create_texture(display, image);
                    let image_id = image_map.insert(texture);

                    let id = tile_coord_to_u64(tile_id);
                    match self.cache.get_mut(id) {
                        Some(value) => {
                            *value = CachedTile::Cached(image_id);
                        }
                        None => {
                            self.cache.insert(id, CachedTile::Cached(image_id));
                        }
                    }

                    tiles_processed += 1;
                }
            }
        }
    }
}

/// An infinite async loop that waits for tile requests, and dispatches them through the levels of
/// cache to produce a texture
async fn tile_requester(
    upload_tx: Sender<MemoryTile>,
    mut request_rx: UnboundedReceiver<TileId>,
    backends: Arc<Vec<Box<dyn Backend>>>,
) {
    //TODO: Reduce Arcing here with some king of task queue that we select so that the lifetimes
    //work out
    let upload_tx = Arc::new(upload_tx);
    while let Some(tile) = request_rx.recv().await {
        //TODO: Limit concurrent requests. Maybe use some kind of convar or custom atomicint?
        let upload_tx = upload_tx.clone();
        let backends = backends.clone();
        tokio::spawn(async move {
            for backend in backends.iter() {
                //Go through each level of cache and try to obtain tile
                match backend.request(tile).await {
                    Ok(Some(image)) => {
                        let _ = upload_tx
                            .send(MemoryTile {
                                image: Some(image),
                                id: tile,
                            })
                            .await;
                        return;
                    }
                    Ok(None) => {}
                    Err(err) => {
                        println!("Error getting tile {:?}: {}", tile, err);
                    }
                }
            }
            let _ = upload_tx
                .send(MemoryTile {
                    image: None,
                    id: tile,
                })
                .await;
        });
    }
}

/// Uploads an RGBA texture to the GPU
fn create_texture(display: &glium::Display, image: image::RgbaImage) -> glium::Texture2d {
    let image_dimensions = image.dimensions();
    let start = std::time::Instant::now();

    let raw_image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let result = glium::texture::Texture2d::new(display, raw_image).unwrap();
    {
        let mut guard = crate::MAP_PERF_DATA.lock();
        guard.tile_upload_time.add_sample(start.elapsed());
    }
    result
}

const ZOOM_BITS: u32 = 5;

#[cfg(debug_assertions)]
const MAX_ZOOM: u32 = 2u32.pow(ZOOM_BITS); //32

//Bits for x and y
const COORD_BITS: u32 = 24;
const MAX_COORD: u32 = 2u32.pow(COORD_BITS);

const X_OFFSET: u32 = COORD_BITS + ZOOM_BITS;
const Y_OFFSET: u32 = ZOOM_BITS;

/// Encodes a tile coordinate into a u64
pub fn tile_coord_to_u64(tile: TileId) -> u64 {
    //Nobody provides tiles for zoom levels > 20 anyway so we are okay to turn this off in release mode
    //This function is very hot too
    #[cfg(debug_assertions)]
    {
        assert!(tile.zoom < MAX_ZOOM);
        assert!(tile.x < MAX_COORD);
        assert!(tile.y < MAX_COORD);
    }

    let x = tile.x as u64;
    let y = tile.y as u64;
    let zoom = tile.zoom as u64;

    x << X_OFFSET | y << Y_OFFSET | zoom
}

/// Decodes a tile coordinate from u64
pub fn u64_to_tile_coord(bits: u64) -> TileId {
    const ZOOM_MASK: u64 = 2u64.pow(ZOOM_BITS) - 1;
    const COORD_MASK: u64 = (MAX_COORD - 1) as u64;

    let zoom = bits & ZOOM_MASK;
    let y = (bits >> Y_OFFSET) & COORD_MASK;
    let x = (bits >> X_OFFSET) & COORD_MASK;

    TileId {
        x: x as u32,
        y: y as u32,
        zoom: zoom as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_and_tile() {
        let test_vector = [
            (
                TileId {
                    x: 0,
                    y: 0,
                    zoom: 0,
                },
                0,
            ),
            (
                TileId {
                    x: 0,
                    y: 0,
                    zoom: 15,
                },
                0b1111,
            ),
            (
                TileId {
                    x: 0,
                    y: 5,
                    zoom: 3,
                },
                #[allow(clippy::unusual_byte_groupings)]
                0b101_00011,
            ),
            (
                TileId {
                    x: 7,
                    y: 1,
                    zoom: 9,
                },
                #[allow(clippy::unusual_byte_groupings)]
                0b111__00000000_00000000_00000001__01001,
            ),
        ];
        for (tile, bits) in test_vector {
            assert_eq!(bits, tile_coord_to_u64(tile));
            assert_eq!(tile, u64_to_tile_coord(bits));
        }
    }

    #[test]
    fn tile_and_intmap() {
        let tile = TileId {
            x: 7,
            y: 1,
            zoom: 9,
        };
        let bits = tile_coord_to_u64(tile);
        let mut map = intmap::IntMap::new();
        map.insert(bits, true);

        let bits = tile_coord_to_u64(tile);

        assert_eq!(*map.get(bits).unwrap(), true);
    }
}
