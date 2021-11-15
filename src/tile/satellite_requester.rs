use super::{disk_cache::DiskCacheData, Backend, ReadinessStatus, TileError, TileId};

use async_trait::async_trait;
use maptiler_cloud::{Maptiler, TileRequest};
use rand::Rng;

pub struct SatelliteRequester {
    maptiler: Maptiler,
    cache_data: DiskCacheData,
}

impl SatelliteRequester {
    pub fn new(cache_data: DiskCacheData) -> Self {
        let api_keys = ["GBnoGxmU64rzYqypBLp9", "VrgC04XoV1a84R5VkUnL"];
        Self {
            maptiler: Maptiler::new(api_keys[rand::thread_rng().gen_range(0..api_keys.len())])
                .expect("Failed to create maptiler TLS backend!"),
            cache_data,
        }
    }
}

#[async_trait]
impl Backend for SatelliteRequester {
    fn name(&self) -> &'static str {
        "Satellite Requester"
    }

    async fn request_inner(&self, tile: TileId) -> Result<Option<Vec<u8>>, TileError> {
        let req = match TileRequest::new(
            maptiler_cloud::TileSet::Satellite,
            tile.x,
            tile.y,
            tile.zoom,
        ) {
            Ok(req) => req,
            Err(_err) => return Ok(None),
        };
        let bytes = self.maptiler.create_request(req).execute().await?;
        let _ = self.cache_data.cache_tile(tile, bytes.as_slice()).await;
        Ok(Some(bytes))
    }

    async fn readiness(&self, _tile: TileId) -> ReadinessStatus {
        ReadinessStatus::Unknown
    }

    fn tile_size(&self) -> Option<u32> {
        Some(128)
    }

    fn ignore_transparent_tiles(&self) -> bool {
        false
    }
}
