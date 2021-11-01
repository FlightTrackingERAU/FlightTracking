use std::{collections::HashMap, time::Duration};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use simple_moving_average::{SumTreeSMA, SMA};

pub struct PerformanceData {
    pub tiles_rendered: usize,
    pub tiles_on_gpu: usize,
    pub tiles_in_memory: usize,
    pub zoom: u32,
    pub backend_request_secs: HashMap<&'static str, SumTreeSMA<Duration, u32, 16>>,
    pub tile_decode_time: SumTreeSMA<Duration, u32, 16>,
    pub tile_upload_time: SumTreeSMA<Duration, u32, 16>,
}

#[derive(Clone)]
pub struct PerformanceDataSnapshot {
    pub tiles_rendered: usize,
    pub tiles_on_gpu: usize,
    pub tiles_in_memory: usize,
    pub zoom: u32,
    pub backend_request_secs: Vec<(&'static str, Duration)>,
    pub tile_decode_time: Duration,
    pub tile_upload_time: Duration,
}

lazy_static! {
    pub static ref MAP_PERF_DATA: Mutex<PerformanceData> = Mutex::new(Default::default());
}

impl PerformanceData {
    pub fn snapshot(&mut self) -> PerformanceDataSnapshot {
        PerformanceDataSnapshot {
            tiles_rendered: self.tiles_rendered,
            tiles_on_gpu: self.tiles_on_gpu,
            tiles_in_memory: self.tiles_in_memory,
            zoom: self.zoom,
            tile_decode_time: self.tile_decode_time.get_average(),
            tile_upload_time: self.tile_upload_time.get_average(),
            backend_request_secs: self
                .backend_request_secs
                .iter()
                .map(|(k, v)| (*k, v.get_average()))
                .collect(),
        }
    }
}

impl Default for PerformanceData {
    fn default() -> Self {
        Self {
            tiles_rendered: Default::default(),
            tiles_on_gpu: Default::default(),
            tiles_in_memory: Default::default(),
            zoom: Default::default(),
            backend_request_secs: Default::default(),
            tile_decode_time: SumTreeSMA::from_zero(Duration::ZERO),
            tile_upload_time: SumTreeSMA::from_zero(Duration::ZERO),
        }
    }
}
