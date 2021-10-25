use std::collections::HashMap;

use tokio::runtime::Runtime;

use crate::{tile_requester::TileRequester, MAX_ZOOM_LEVEL};

#[derive(Debug, Copy, Clone)]
pub struct TileId {
    pub x: u32,
    pub y: u32,
    pub zoom: u32,
}

impl TileId {
    pub fn new(x: u32, y: u32, zoom: u32) -> Self {
        Self { x, y, zoom }
    }
}

pub struct Tile {
    pub id: TileId,
    pub image: image::RgbaImage,
}

#[derive(Debug, Copy, Clone)]
enum CachedTile {
    Pending,
    Cached(conrod_core::image::Id),
}

pub struct TileCache {
    tile_requester: TileRequester,
    hashmaps: Vec<HashMap<(u32, u32), CachedTile>>,
}

impl TileCache {
    pub fn new(runtime: &Runtime) -> Self {
        // This is 1+ because it counts the 0th zoom level
        let mut hashmaps = Vec::with_capacity(1 + MAX_ZOOM_LEVEL as usize);

        // Initialize all of the hashmaps
        for _ in 0..(1 + MAX_ZOOM_LEVEL) {
            let hashmap = HashMap::new();

            hashmaps.push(hashmap);
        }

        // GBnoGxmU64rzYqypBLp9
        // VrgC04XoV1a84R5VkUnL
        Self {
            tile_requester: TileRequester::new("GBnoGxmU64rzYqypBLp9", runtime),
            hashmaps,
        }
    }

    pub fn get_tile(&mut self, tile_id: TileId) -> Option<conrod_core::image::Id> {
        if let Some(cached_tile) = self.get_cached_tile(tile_id) {
            if let CachedTile::Cached(image_id) = cached_tile {
                Some(image_id)
            } else {
                None
            }
        } else {
            // Make the request to the tile requester
            self.tile_requester.request(tile_id);

            // Insert a placeholder
            self.set_cached_tile(tile_id, CachedTile::Pending);

            None
        }
    }

    pub fn tile_size(&self) -> Option<u32> {
        self.tile_requester.tile_size()
    }

    fn set_cached_tile(&mut self, tile_id: TileId, cached_tile: CachedTile) {
        // TODO: Optimization: .get_unchecked_mut?
        let hash_map = self.hashmaps.get_mut(tile_id.zoom as usize).unwrap();

        hash_map.insert((tile_id.x, tile_id.y), cached_tile);
    }

    fn get_cached_tile(&self, id: TileId) -> Option<CachedTile> {
        self.hashmaps
            .get(id.zoom as usize)
            .unwrap()
            .get(&(id.x, id.y))
            .copied()
    }

    /// Uploads new images to the GPU as they come in.
    ///
    /// Executes on the main thread. Change MAX_PROCESS_TIME to tune how long this function will
    /// run for
    pub fn process(
        &mut self,
        display: &glium::Display,
        image_map: &mut conrod_core::image::Map<glium::Texture2d>,
    ) {
        // use std::time::{Duration, Instant};
        // const MAX_PROCESS_TIME: Duration = Duration::from_millis(15);
        // let start = std::time::Instant::now();
        // let mut tiles_processed = 0;

        while let Some(tile) = self.tile_requester.next_ready_tile() {
            // let time_spent = Instant::now() - start;
            /*
            if time_spent > MAX_PROCESS_TIME {
                println!(
                    "Breaking from process loop after {} ms. Processed {} tiles",
                    time_spent.as_micros() as f64 / 1000.0,
                    tiles_processed
                );
                break;
            }*/
            let tile_id = tile.id;

            let texture = self.create_texture(display, tile.image);
            let image_id = image_map.insert(texture);

            self.set_cached_tile(tile_id, CachedTile::Cached(image_id));
            // tiles_processed += 1;
        }
    }

    // Creates a texture from a raw RgbaImage by registering it with glium
    fn create_texture(
        &self,
        display: &glium::Display,
        image: image::RgbaImage,
    ) -> glium::Texture2d {
        let image_dimensions = image.dimensions();

        let raw_image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

        glium::texture::Texture2d::new(display, raw_image).unwrap()
    }
}
