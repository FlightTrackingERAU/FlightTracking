mod backend;
mod pipeline;

pub use backend::*;
pub use pipeline::*;

use enum_map::{enum_map, Enum, EnumMap};
use tokio::runtime::Runtime;

#[derive(Debug, Enum)]
pub enum TileKind {
    Satellite,
    Weather,
}

pub type PipelineMap = EnumMap<TileKind, TilePipeline>;

pub fn pipelines(runtime: &Runtime) -> PipelineMap {
    enum_map! {
        TileKind::Satellite => TilePipeline::new(vec![Box::new(DiskTileCache::new("tile-cache", "jpg"))], runtime),
        TileKind::Weather => TilePipeline::new(vec![], runtime),
    }
}
