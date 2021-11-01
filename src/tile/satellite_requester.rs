use super::{Backend, ReadinessStatus, TileError, TileId};

use async_trait::async_trait;
use maptiler_cloud::{Maptiler, TileRequest};
use rand::Rng;

pub struct SatelliteRequester {
    maptiler: Maptiler,
}

impl SatelliteRequester {
    pub fn new() -> Self {
        let api_keys = ["GBnoGxmU64rzYqypBLp9", "VrgC04XoV1a84R5VkUnL"];
        Self {
            maptiler: Maptiler::new(api_keys[rand::thread_rng().gen_range(0..api_keys.len())])
                .expect("Failed to create maptiler TLS backend!"),
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
        println!("Requesting: {:?}", req);
        Ok(Some(self.maptiler.create_request(req).execute().await?))
    }

    async fn readiness(&self, tile: TileId) -> ReadinessStatus {
        ReadinessStatus::Unknown
    }
}
