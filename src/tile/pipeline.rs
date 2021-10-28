use super::*;

use image::{ImageBuffer, Rgba};
use std::error::Error;

/// Holds multiple levels of cache for requesting tiles in a generic manner
pub struct TilePipeline {
    cache: Vec<Box<dyn Backend>>,
}

impl TilePipeline {
    pub fn new(cache: Vec<Box<dyn Backend>>) -> Self {
        Self { cache }
    }

    pub async fn get_tile(
        &self,
        tile: TileId,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
        for backend in &self.cache {
            if let Ok(Some(image)) = backend.request(tile).await {
                return Ok(image);
            }
        }
        return Err("Failed to get tile".into());
    }
}
