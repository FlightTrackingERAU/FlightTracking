use lazy_static::lazy_static;
use parking_lot::Mutex;

#[derive(Debug)]
pub struct TileTypeStats {
    pub api_secs: running_average::RealTimeRunningAverage<f64>,
    pub decode_secs: running_average::RealTimeRunningAverage<f64>,
    pub upload_secs: running_average::RealTimeRunningAverage<f64>,
}

#[derive(Debug)]
pub struct PerformanceData {
    pub tiles_rendered: usize,
    pub tiles_on_gpu: usize,
    pub tiles_in_memory: usize,
    pub zoom: u32,
    pub satellite: TileTypeStats,
    pub weather: TileTypeStats,
}

//Same as other structs but are Clone
#[derive(Debug, Clone)]
pub struct TileTypeStatsSnapshot {
    pub api_secs: f64,
    pub decode_secs: f64,
    pub upload_secs: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceDataSnapshot {
    pub tiles_rendered: usize,
    pub tiles_on_gpu: usize,
    pub tiles_in_memory: usize,
    pub zoom: u32,
    pub satellite: TileTypeStatsSnapshot,
    pub weather: TileTypeStatsSnapshot,
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
            satellite: TileTypeStatsSnapshot {
                api_secs: self.satellite.api_secs.measurement().rate(),
                decode_secs: self.satellite.decode_secs.measurement().rate(),
                upload_secs: self.satellite.upload_secs.measurement().rate(),
            },
            weather: TileTypeStatsSnapshot {
                api_secs: self.weather.api_secs.measurement().rate(),
                decode_secs: self.weather.decode_secs.measurement().rate(),
                upload_secs: self.weather.upload_secs.measurement().rate(),
            },
        }
    }
}

impl Default for TileTypeStats {
    fn default() -> Self {
        Self {
            api_secs: Default::default(),
            decode_secs: Default::default(),
            upload_secs: Default::default(),
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
