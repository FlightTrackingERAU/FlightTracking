use async_trait::async_trait;
use image::{ImageBuffer, Rgba};
use simple_moving_average::{SumTreeSMA, SMA};

use std::time::Duration;
use thiserror::Error;

use super::TileId;

/// The different levels of readiness when of a tile within a backend.
///
/// This enum allows users of [`Backend`] to make better decisions between calling [`Backend::readiness`]
/// and [`Backend::request`].
pub enum ReadinessStatus {
    /// The tile is known to be available [`Backend::request`] should return Ok(Some(...)) in
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

/// An error produced if loading a tile fails
#[derive(Error, Debug)]
pub enum TileError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image: {0}")]
    Image(#[from] image::ImageError),
    #[error("Join: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("Maptiler: {0}")]
    Maptiler(#[from] maptiler_cloud::errors::Error),
}

pub type Texture = ImageBuffer<Rgba<u8>, Vec<u8>>;

/// A low level construct for requesting map tiles form a single source, such as an api,
/// disk cache, or memory cache.
///
/// The main concern of this trait is to abstract loading tiles from arbitrary locations, therefore
/// it does not support caching or even an external api for querying which requests are in progress.
/// These are left to higher level layers, notably [`crate::tile::TilePipeline`].
#[async_trait]
pub trait Backend: Send + Sync {
    /// Initiates an asynchronous request to obtain the image data for `tile`.
    ///
    /// For some backends this make take upwards of a second.
    /// To check weather a given backend can obtain a tile without actually going through the process of requesting it,
    /// use [`Backend::readiness`].
    async fn request(&self, tile: TileId) -> Result<Option<Texture>, TileError> {
        let start = std::time::Instant::now();

        let result = self.request_inner(tile).await?;

        let duration = start.elapsed();
        {
            let mut guard = crate::MAP_PERF_DATA.lock();
            guard
                .backend_request_secs
                .entry(self.name())
                .or_insert_with(|| SumTreeSMA::from_zero(Duration::ZERO))
                .add_sample(duration);
        }
        match result {
            Some(bytes) => Ok(Some(load_tile(bytes).await?)),
            None => Ok(None),
        }
    }

    /// Queries the readiness status for a given tile in this backend. 
    ///
    /// Can be used to improve the performance when doing tile preemption
    async fn readiness(&self, tile: TileId) -> ReadinessStatus;

    /// The name of this backend
    fn name(&self) -> &'static str;

    /// The size of tiles returned by this backend.
    ///
    /// Returns `None` if unknown
    fn tile_size(&self) -> Option<u32>;

    /// Requests a tile from the this backend, returning the image bytes if the tile could be
    /// requested successfully
    async fn request_inner(&self, tile: TileId) -> Result<Option<Vec<u8>>, TileError>;
}

/// Decodes a compressed png or jpeg image into a RGBA memory byte buffer. 
///
/// Users will usually call this and then upload the result to the GPU
pub async fn load_tile(bytes: Vec<u8>) -> Result<Texture, TileError> {
    let result: Result<Texture, TileError> = tokio::task::spawn_blocking(move || {
        let start = std::time::Instant::now();

        let image = image::load_from_memory(&bytes)?.into_rgba();

        let duration = start.elapsed();
        let mut guard = crate::MAP_PERF_DATA.lock();
        guard.tile_decode_time.add_sample(duration);
        Ok(image)
    })
    .await?;
    let image = result?;

    //Images must be square
    assert_eq!(image.width(), image.height());
    Ok(image)
}
