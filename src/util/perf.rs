use lazy_static::lazy_static;
use parking_lot::Mutex;
use simple_moving_average::{SumTreeSMA, SMA};

pub struct ApiTimeData {
    pub api_secs: SumTreeSMA<f32, f32, 8>,
    pub decode_secs: SumTreeSMA<f32, f32, 16>,
    pub upload_secs: SumTreeSMA<f32, f32, 16>,
}

pub struct PerformanceData {
    pub tiles_rendered: usize,
    pub tiles_on_gpu: usize,
    pub tiles_in_memory: usize,
    pub zoom: u32,
    pub satellite: ApiTimeData,
    pub weather: ApiTimeData,
}

//Same as other structs but are Clone
#[derive(Clone)]
pub struct ApiTimeDataSnapshot {
    pub api_secs: f32,
    pub decode_secs: f32,
    pub upload_secs: f32,
}

#[derive(Clone)]
pub struct PerformanceDataSnapshot {
    pub tiles_rendered: usize,
    pub tiles_on_gpu: usize,
    pub tiles_in_memory: usize,
    pub zoom: u32,
    pub satellite: ApiTimeDataSnapshot,
    pub weather: ApiTimeDataSnapshot,
}

lazy_static! {
    pub static ref PERF_DATA: Mutex<PerformanceData> = Mutex::new(Default::default());
}

impl PerformanceData {
    pub fn snapshot(&mut self) -> PerformanceDataSnapshot {
        PerformanceDataSnapshot {
            tiles_rendered: self.tiles_rendered,
            tiles_on_gpu: self.tiles_on_gpu,
            tiles_in_memory: self.tiles_in_memory,
            zoom: self.zoom,
            satellite: ApiTimeDataSnapshot {
                api_secs: self.satellite.api_secs.get_average(),
                decode_secs: self.satellite.decode_secs.get_average(),
                upload_secs: self.satellite.upload_secs.get_average(),
            },
            weather: ApiTimeDataSnapshot {
                api_secs: self.weather.api_secs.get_average(),
                decode_secs: self.weather.decode_secs.get_average(),
                upload_secs: self.weather.upload_secs.get_average(),
            },
        }
    }
}

impl Default for ApiTimeData {
    fn default() -> Self {
        Self {
            api_secs: SumTreeSMA::new(),
            decode_secs: SumTreeSMA::new(),
            upload_secs: SumTreeSMA::new(),
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
            satellite: Default::default(),
            weather: Default::default(),
        }
    }
}
