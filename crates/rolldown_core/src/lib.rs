#![feature(map_many_mut)]
#![feature(box_syntax)]
#![feature(hash_raw_entry)]
#![feature(iterator_try_collect)]
use std::sync::Arc;

mod bundler;
pub use bundler::*;
mod chunk;
pub use chunk::*;
mod normal_module;
pub use normal_module::*;
mod external_module;
pub use external_module::*;
mod options;
pub use options::*;
mod graph;
pub use graph::*;
mod module_loader;
use rolldown_common::{ChunkId, ExportedSpecifier, ModuleId};
use rolldown_resolver::Resolver;
use rustc_hash::FxHashMap;
use swc_core::common::{FilePathMapping, Globals, SourceMap};
mod bundle;
mod norm_or_ext;
pub use bundle::*;
use swc_core::ecma::atoms::JsWord;
mod code_splitter;
pub use code_splitter::*;
mod chunk_graph;
pub(crate) use chunk_graph::*;
mod plugin_driver;
pub(crate) use plugin_driver::*;
mod utils;
pub use utils::*;
mod rolldown_output;
mod treeshake;
pub use rolldown_output::*;

pub(crate) type ResolvedModuleIds = FxHashMap<JsWord, ModuleId>;
pub(crate) type MergedExports = FxHashMap<JsWord, ExportedSpecifier>;
pub(crate) type SharedResolver = Arc<Resolver>;
pub(crate) use norm_or_ext::*;
use once_cell::sync::Lazy;

pub(crate) static SOURCE_MAP: Lazy<Arc<SourceMap>> =
  Lazy::new(|| Arc::new(SourceMap::new(FilePathMapping::empty())));

pub(crate) static COMPILER: Lazy<Arc<rolldown_compiler::Compiler>> = Lazy::new(|| {
  let cm = SOURCE_MAP.clone();
  let compiler = rolldown_compiler::Compiler::with_cm(cm);
  Arc::new(compiler)
});

pub(crate) type ModuleById = FxHashMap<ModuleId, NormOrExt>;
pub(crate) type ModuleRefMutById<'a> = FxHashMap<&'a ModuleId, &'a mut NormOrExt>;
pub(crate) type SplitPointIdToChunkId = FxHashMap<ModuleId, ChunkId>;
pub(crate) static SWC_GLOBALS: Lazy<Arc<Globals>> = Lazy::new(|| Arc::new(Globals::new()));

// public exports

pub type BundleResult<T> = rolldown_error::Result<T>;
pub type BundleError = rolldown_error::Error;
