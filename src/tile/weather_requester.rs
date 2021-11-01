use async_trait::async_trait;
use rain_viewer::RequestArguments;
use simple_moving_average::SMA;

use std::time::Instant;

use super::{Backend, ReadinessStatus, TileError, TileId};
use crate::tile::backend::load_tile;

#[derive(Clone, Debug)]
struct WeatherData {
    data: rain_viewer::AvailableData,
    time: Instant,
}

pub struct WeatherRequester {
    available: tokio::sync::RwLock<Option<WeatherData>>,
}

impl WeatherRequester {
    pub fn new() -> Self {
        Self {
            available: tokio::sync::RwLock::new(None),
        }
    }
}

impl WeatherRequester {
    async fn update_maps(&self) -> Result<WeatherData, rain_viewer::Error> {
        rain_viewer::available().await.map(|data| WeatherData {
            data,
            time: Instant::now(),
        })
    }
}

#[async_trait]
impl Backend for WeatherRequester {
    fn name(&self) -> &'static str {
        "Weather Requester"
    }

    async fn request_inner(&self, tile: TileId) -> Result<Option<Vec<u8>>, TileError> {
        loop {
            {
                let guard = self.available.read().await;
                match &*guard {
                    Some(available) => {
                        if let Some(last_frame) = available.data.nowcast_radar.last() {
                            if let Ok(mut args) = RequestArguments::new_tile(tile.x, tile.y, tile.zoom)
                            {
                                args.set_color(rain_viewer::ColorKind::TheWeatherChannel);
                                match rain_viewer::get_tile(&available.data, last_frame, args).await
                                {
                                    Ok(bytes) => {
                                        println!("got {} bytes for tile {:?}", bytes.len(), tile);
                                        return Ok(Some(bytes));
                                    }
                                    Err(err) => {
                                        println!("failed to get tile {:?}: {:?}", tile, err);
                                    }
                                }
                            }
                        }
                        return Ok(None);
                    }
                    None => {
                        //Leave this scope so we can get write access and set available
                        println!("No data available frow write size. Trying to get write access");
                    }
                }
            }
            let mut guard = self.available.write().await;
            match &*guard {
                Some(_already_set) => {
                    println!("Got write access. Someone beat use in setting it");
                    //Someone set it before us. Loop around because we can now use it
                }
                None => {
                    //We have exclusive write access and there is no data
                    //Therefore it is our responsibility to load the data
                    println!("Got write access. Its our job to get the data");
                    if let Ok(data) = self.update_maps().await {
                        *guard = Some(data);
                        println!("Set data");
                    }
                }
            }
        }
    }

    async fn readiness(&self, tile: TileId) -> ReadinessStatus {
        ReadinessStatus::Unknown
    }

    async fn request(&self, tile: TileId) -> Result<Option<crate::Texture>, TileError> {
        let start = std::time::Instant::now();

        let result = self.request_inner(tile).await?;

        let duration = start.elapsed();
        {
            let mut guard = crate::MAP_PERF_DATA.lock();
            guard
                .backend_request_secs
                .entry(self.name())
                .or_insert_with(|| {
                    simple_moving_average::SumTreeSMA::from_zero(std::time::Duration::ZERO)
                })
                .add_sample(duration);
        }
        match result {
            Some(bytes) => Ok(Some(load_tile(bytes).await?)),
            None => Ok(None),
        }
    }
}
