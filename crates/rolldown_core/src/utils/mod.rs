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

pub fn syntax_by_loader(loader: &Loader) -> swc_core::ecma::parser::Syntax {
  use swc_core::ecma::parser::{EsConfig, Syntax, TsConfig};

  match loader {
    Loader::Js => Syntax::Es(EsConfig {
      jsx: false,
      fn_bind: false,
      decorators: false,
      decorators_before_export: false,
      export_default_from: false,
      import_assertions: false,
      allow_super_outside_method: false,
      allow_return_outside_function: false,
    }),
    Loader::Jsx => Syntax::Es(EsConfig {
      jsx: true,
      fn_bind: false,
      decorators: false,
      decorators_before_export: false,
      export_default_from: false,
      import_assertions: false,
      allow_super_outside_method: false,
      allow_return_outside_function: false,
    }),
    Loader::Ts => Syntax::Typescript(TsConfig {
      tsx: false,
      decorators: false,
      dts: false,
      no_early_errors: false,
    }),
    Loader::Tsx => Syntax::Typescript(TsConfig {
      tsx: true,
      decorators: false,
      dts: false,
      no_early_errors: false,
    }),
  }
}
