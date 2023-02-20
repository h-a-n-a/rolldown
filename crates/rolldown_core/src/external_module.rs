use rolldown_common::{ModuleId, Symbol};
use rolldown_runtime_helpers::RuntimeHelpers;
use rustc_hash::FxHashMap;
use swc_core::{common::SyntaxContext, ecma::atoms::JsWord};

/// Currently, the usages of ExternalModule are:
/// - Help with union all imported symbols with the same `imported` name  from the same external module.
#[derive(Debug)]
pub struct ExternalModule {
  pub exec_order: usize,
  pub id: ModuleId,
  pub(crate) top_level_ctxt: SyntaxContext,
  pub(crate) runtime_helpers: RuntimeHelpers,
  pub(crate) exports: FxHashMap<JsWord, Symbol>,
}

impl ExternalModule {
  pub(crate) fn find_exported_symbol(&mut self, exported_name: &JsWord) -> &Symbol {
    self
      .exports
      .raw_entry_mut()
      .from_key(exported_name)
      .or_insert_with(|| {
        (
          exported_name.clone(),
          Symbol::new(exported_name.clone(), self.top_level_ctxt),
        )
      })
      .1
  }
}
