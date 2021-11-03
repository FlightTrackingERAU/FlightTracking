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

use std::time::Duration;
use enum_map::{enum_map, Enum, EnumMap};
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
    let weather_expiry = Duration::from_secs(60 * 5);
    enum_map! {
        TileKind::Satellite => TilePipeline::new(vec![
            Box::new(DiskCache::new(".cache/satellite", "jpg", None)),
            Box::new(SatelliteRequester::new())
        ], runtime),
        TileKind::Weather => TilePipeline::new(vec![
            Box::new(DiskCache::new(".cache/weather", "png", Some(weather_expiry))),
            Box::new(WeatherRequester::new())
        ], runtime),
    }
}
