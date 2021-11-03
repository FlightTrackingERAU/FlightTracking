//atomic enum generates a compare_and_swap function that calls to AtomicUsize's compare_and_swap
//which is deprecated.
//We dont use compare_and_swap function so its fine
//Because it generates this from within a macro, we unfortunately need to disable deprecation
//warnings for the whole file :[
#![allow(deprecated)]

use async_trait::async_trait;
use rain_viewer::RequestArguments;
use simple_moving_average::SMA;

use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use super::{disk_cache::DiskCacheData, Backend, ReadinessStatus, TileError, TileId};
use crate::tile::backend::load_tile;

#[atomic_enum::atomic_enum]
#[derive(Eq, PartialEq)]
enum WeatherDataState {
    /// We have no data yet, and nobody has started getting it yet
    Uninitialized,
    /// We have no data, but someone has started loading it
    Initializing,
    /// The data is available
    Available,
    /// The data is available and new data is being loaded simultaneously
    AvailableUpdating,
}

#[derive(Clone, Debug)]
struct WeatherData {
    data: rain_viewer::AvailableData,
    time: Instant,
}

pub struct WeatherRequester {
    available: tokio::sync::RwLock<Option<WeatherData>>,
    state: AtomicWeatherDataState,
    tile_size: u32,
    cache_data: DiskCacheData,
}

impl WeatherRequester {
    pub fn new(cache_data: DiskCacheData) -> Self {
        Self {
            available: tokio::sync::RwLock::new(None),
            state: AtomicWeatherDataState::new(WeatherDataState::Uninitialized),
            tile_size: 512,
            cache_data,
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
            let state = self.state.load(Ordering::Acquire);
            match state {
                WeatherDataState::Uninitialized => {
                    if self
                        .state
                        .compare_exchange(
                            WeatherDataState::Uninitialized,
                            WeatherDataState::Initializing,
                            Ordering::AcqRel,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        //We were able to modify the state to Initializing.
                        //This means its our responsibility to load the data
                        let mut guard = self.available.write().await;
                        if guard.is_some() {
                            println!("Someone initialized available already?!");
                            continue;
                        }
                        match self.update_maps().await {
                            Ok(data) => {
                                *guard = Some(data);
                                self.state
                                    .store(WeatherDataState::Available, Ordering::Release);
                            }
                            Err(err) => {
                                self.state
                                    .store(WeatherDataState::Uninitialized, Ordering::Release);
                                println!("Failed to get data while initializing: {:?}", err);
                            }
                        }
                    }
                }
                WeatherDataState::Initializing => {
                    //We basically have to wait for initialization to finish here.
                    //Yield so we give the initializing task a change to grab write access
                    tokio::task::yield_now().await;

                    //We assume the initializing task now has write access, so we can know when it
                    //finishes based off when it release write access (we get read access).
                    //If we are ahead of them and `yield_now().await` returns before they get write
                    //access, then we will get read access, drop the guard immediately, then go around the
                    //loop again and yield, giving it another chance to get write access.
                    //This repeats until the data is available
                    let _ = self.available.read().await;
                }
                WeatherDataState::Available | WeatherDataState::AvailableUpdating => {
                    let guard = self.available.read().await;
                    let available = guard.as_ref().unwrap();
                    if state == WeatherDataState::Available {
                        //We need to check for new updates if we are available
                        if Instant::now().duration_since(available.time)
                            > Duration::from_secs(60 * 5)
                            && self
                                .state
                                .compare_exchange(
                                    WeatherDataState::Available,
                                    WeatherDataState::AvailableUpdating,
                                    Ordering::AcqRel,
                                    Ordering::Relaxed,
                                )
                                .is_ok()
                        {
                            //We were able to modify the state to AvailableUpdating.
                            //This means its our responsibility to load the new data
                            drop(guard);
                            println!("Task is getting new data");
                            if let Ok(new_data) = self.update_maps().await {
                                *self.available.write().await = Some(new_data);
                                println!("Loaded new data");
                            }

                            self.state
                                .store(WeatherDataState::Available, Ordering::Release);
                            continue;
                        }
                    }

                    if let Some(last_frame) = available.data.nowcast_radar.last() {
                        if let Ok(mut args) = RequestArguments::new_tile(tile.x, tile.y, tile.zoom)
                        {
                            args.set_size(self.tile_size).unwrap();
                            args.set_color(rain_viewer::ColorKind::TheWeatherChannel);
                            match rain_viewer::get_tile(&available.data, last_frame, args).await {
                                Ok(bytes) => {
                                    let _ =
                                        self.cache_data.cache_tile(tile, bytes.as_slice()).await;
                                    return Ok(Some(bytes));
                                }
                                Err(err) => {
                                    println!("failed to get tile {:?}: {:?}", tile, err);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn readiness(&self, _tile: TileId) -> ReadinessStatus {
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

    fn tile_size(&self) -> Option<u32> {
        Some(self.tile_size)
    }
}
