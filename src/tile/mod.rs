mod backend;
mod disk_cache;
mod pipeline;

mod satellite_requester;
mod weather_requester;

pub use backend::*;
pub use pipeline::*;

use disk_cache::*;
use satellite_requester::*;
use weather_requester::*;

use enum_map::{enum_map, Enum, EnumMap};
use std::time::Duration;
use tokio::runtime::Runtime;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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

#[derive(Debug, Enum)]
pub enum TileKind {
    Satellite,
    Weather,
}

pub type PipelineMap = EnumMap<TileKind, TilePipeline>;

pub fn pipelines(runtime: &Runtime) -> PipelineMap {
    let satellite_cache = DiskCacheData {
        folder_name: ".cache/satellite",
        image_extension: "jpg",
        invalidate_time: Duration::from_secs(60 * 60 * 24 * 30), //One month long cache
    };
    let weather_cache = DiskCacheData {
        folder_name: ".cache/weather",
        image_extension: "png",
        invalidate_time: Duration::from_secs(60 * 5), //Five minute cache
    };
    enum_map! {
        TileKind::Satellite => TilePipeline::new(vec![
            Box::new(DiskCache::new(satellite_cache)),
            Box::new(SatelliteRequester::new(satellite_cache))
        ], runtime),
        TileKind::Weather => TilePipeline::new(vec![
            Box::new(DiskCache::new(weather_cache)),
            Box::new(WeatherRequester::new(weather_cache))
        ], runtime),
    }
}
