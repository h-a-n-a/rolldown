use rolldown_common::{ChunkId, ModuleId};
use rustc_hash::FxHashMap;

use crate::Chunk;

pub(crate) struct ChunkGraph {
  pub(crate) chunk_by_id: FxHashMap<ChunkId, Chunk>,
  pub(crate) split_point_to_chunk: FxHashMap<ModuleId, ChunkId>,
}
