mod backend;
mod pipeline;

pub use backend::*;
pub use pipeline::*;

use enum_map::{enum_map, Enum, EnumMap};

#[derive(Debug, Enum)]
pub enum TileKind {
    Satellite,
    Weather,
}

pub type PipelineMap = EnumMap<TileKind, TilePipeline>;

pub fn pipelines() -> PipelineMap {
    enum_map! {
        TileKind::Satellite => TilePipeline::new(Vec::new()),
        TileKind::Weather => TilePipeline::new(Vec::new()),
    }
}
