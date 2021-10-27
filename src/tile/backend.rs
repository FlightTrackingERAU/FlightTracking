use async_trait::async_trait;
use image::{ImageBuffer, Rgba};

use std::error::Error;

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

#[async_trait]
pub trait Backend {
    async fn request(
        &self,
        tile: TileId,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
        let bytes = self.request_inner(tile).await?;
        Ok(load_tile(bytes)?)
    }

    /// Requests a tile from the this backend, returning the image bytes if the tile could be
    /// requested successfully
    async fn request_inner(&self, tile: TileId) -> Result<Vec<u8>, Box<dyn Error>>;
}

fn load_tile(bytes: Vec<u8>) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
    let image = image::load_from_memory(&bytes)?.into_rgba();
    //Images must be square
    assert_eq!(image.width(), image.height());
    Ok(image)
}

fn get_tile_path(folder_name: &str, tile: TileId) -> String {
    format!("./{}/{}/{}/{}.jpg", folder_name, tile.zoom, tile.x, tile.y)
}

pub struct DiskTileCache {
    folder_name: String,
}

#[async_trait]
impl Backend for DiskTileCache {
    async fn request_inner(&self, id: TileId) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(tokio::fs::read(get_tile_path(self.folder_name.as_str(), id)).await?)
    }
}
