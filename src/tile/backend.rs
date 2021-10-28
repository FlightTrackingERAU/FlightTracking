use async_trait::async_trait;
use image::{ImageBuffer, Rgba};
use simple_moving_average::{SumTreeSMA, SMA};

use thiserror::Error;
use std::time::Duration;

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

/// The different levels of readiness when a tile is requested to be preempted.
///
/// This enum allows users of [`Backend`] to make better decisions between calling [`Backend::preempt`]
/// and [`Backend::request`].
pub enum PreemptStatus {
    /// The tile is known to be available [`PreemptStatus::Ready`] should return Ok(Some(...)) in
    /// most cases
    Available,

    /// The tile is not available to this backend, so calling [`Backend::request`] will likely return
    /// `Ok(None)`. However this is not a hard requirement. Should the tile become available
    /// between a caller reading this status and calling [`Backend::request`], `Ok(Some(...))` is
    /// allowed to be returned.
    NotAvailable,

    /// The readiness state of this tile is unknown.
    ///
    /// Backends that can only query for availability by actually requesting the tile should aim to
    /// return this status to reduce API traffic.
    Unknown,
}

#[derive(Error, Debug)]
pub enum TileError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image: {0}")]
    Image(#[from] image::ImageError),
    #[error("Join: {0}")]
    Join(#[from] tokio::task::JoinError),
}

pub type Texture = ImageBuffer<Rgba<u8>, Vec<u8>>;

/// A low level construct for requesting map tiles form a single source, such as an api,
/// disk cache, or memory cache.
#[async_trait]
pub trait Backend: Send + Sync {
    /// Initiates an asynchronous request to obtain the image data for `tile`.
    ///
    /// For some backends this make take upwards of a second.
    /// To check weather a given backend can obtain a tile without actually going through the process of requesting it,
    /// use [`Backend::preempt`].
    async fn request(&self, tile: TileId) -> Result<Option<Texture>, TileError> {
        let start = std::time::Instant::now();

        let result = self.request_inner(tile).await?;

        let duration = start.elapsed();
        {
            let mut guard = crate::PERF_DATA.lock();
            guard
                .backend_request_secs
                .entry(self.name())
                .or_insert(SumTreeSMA::from_zero(Duration::ZERO))
                .add_sample(duration);
        }
        match result {
            Some(bytes) => Ok(Some(load_tile(bytes).await?)),
            None => Ok(None),
        }
    }

    async fn preempt(&self, tile: TileId) -> PreemptStatus;

    fn name(&self) -> &'static str;

    /// Requests a tile from the this backend, returning the image bytes if the tile could be
    /// requested successfully
    async fn request_inner(&self, tile: TileId) -> Result<Option<Vec<u8>>, TileError>;
}

async fn load_tile(bytes: Vec<u8>) -> Result<Texture, TileError> {
    let result: Result<(Texture, Duration), TileError> =
        tokio::task::spawn_blocking(move || {
            let start = std::time::Instant::now();

            let image = image::load_from_memory(&bytes)?.into_rgba();

            let duration = start.elapsed();
            Ok((image, duration))
        })
        .await?;
    let (image, duration) = result?;

    let mut guard = crate::PERF_DATA.lock();
    guard.tile_decode_time.add_sample(duration);
    //Images must be square
    assert_eq!(image.width(), image.height());
    Ok(image)
}

fn get_tile_path(folder_name: &str, extension: &str, tile: TileId) -> String {
    format!(
        "./{}/{}/{}/{}.{}",
        folder_name, tile.zoom, tile.x, tile.y, extension
    )
}

pub struct DiskTileCache {
    folder_name: String,
    image_extension: String,
}

#[async_trait]
impl Backend for DiskTileCache {
    async fn request_inner(&self, tile: TileId) -> Result<Option<Vec<u8>>, TileError> {
        let path = get_tile_path(
            self.folder_name.as_str(),
            self.image_extension.as_str(),
            tile,
        );
        match std::fs::metadata(&path) {
            Ok(_) => Ok(Some(tokio::fs::read(path).await?)),
            Err(_) => Ok(None),
        }
    }

    async fn preempt(&self, tile: TileId) -> PreemptStatus {
        let path = get_tile_path(
            self.folder_name.as_str(),
            self.image_extension.as_str(),
            tile,
        );
        match std::fs::metadata(&path) {
            Ok(_) => PreemptStatus::Available,
            Err(_) => PreemptStatus::NotAvailable,
        }
    }

    fn name(&self) -> &'static str {
        "Disk"
    }
}
