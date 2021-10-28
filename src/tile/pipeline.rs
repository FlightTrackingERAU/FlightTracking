use super::*;
use crate::{TileId, WorldViewport};

use image::{ImageBuffer, Rgba};
use intmap::IntMap;
use tokio::sync::mpsc::UnboundedReceiver;

use std::error::Error;
use std::sync::Mutex;

/// Holds multiple levels of cache for requesting tiles in a generic manner.
/// Handles preemption and de-duplicating tile requests so that only one is sent out
pub struct TilePipeline {
    backends: Vec<Box<dyn Backend>>,

    /// The cache of tiles on the GPU
    // Use a blocking mutex here because contention is low, and the critical section is short
    cache: Mutex<IntMap<CachedTile>>,
}

#[derive(Debug, Copy, Clone)]
enum CachedTile {
    Pending,
    Cached(conrod_core::image::Id),
}

impl TilePipeline {
    pub fn new(cache: Vec<Box<dyn Backend>>) -> Self {
        //Use large initial size here because we will have a few hundred tiles on the GPU at
        //minimum, and rehashing is EXPENSIVE
        Self {
            backends: cache,
            cache: Mutex::new(IntMap::with_capacity(1024)),
        }
    }

    pub async fn request_tile(
        &self,
        tile: TileId,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
        for backend in &self.backends {
            if let Ok(Some(image)) = backend.request(tile).await {
                return Ok(image);
            }
        }
        return Err("Failed to get tile".into());
    }

    pub fn get_tile(&self, tile: TileId) -> Option<conrod_core::image::Id> {
        None
    }

    /// Called each frame to allow the pipeline to upload newly fetched tiles to the GPU.
    ///
    /// `viewport`: The viewport of the currently rendered scene. This is used for preemption
    pub fn update(
        &self,
        viewport: &WorldViewport,
        display: &glium::Display,
        image_map: &mut conrod_core::image::Map<glium::Texture2d>,
    ) {
    }
}

const ZOOM_BITS: u32 = 5;
const MAX_ZOOM: u32 = 2u32.pow(ZOOM_BITS); //32

//Bits for x and y
const COORD_BITS: u32 = 24;
const MAX_COORD: u32 = 2u32.pow(COORD_BITS);

const X_OFFSET: u32 = COORD_BITS + ZOOM_BITS;
const Y_OFFSET: u32 = ZOOM_BITS;

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
}
