mod resolve_id;
use std::{path::Path, str::FromStr};

pub(crate) use resolve_id::*;
mod name_helpers;
pub use name_helpers::*;
mod preset_of_used_names;
pub(crate) use preset_of_used_names::*;
use rolldown_common::Loader;

pub fn extract_loader_by_path(p: &Path) -> Loader {
  p.extension()
    .and_then(|ext| ext.to_str())
    // Unknown extension should treat like JavaScript for Rollup-compatibility
    .map(Loader::from_str)
    .map(|l| l.unwrap_or(Loader::Js))
    .unwrap_or(Loader::Js)
}
