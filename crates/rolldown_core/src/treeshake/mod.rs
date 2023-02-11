use std::sync::{
  atomic::{AtomicBool, AtomicUsize, Ordering},
  Mutex,
};

use rayon::prelude::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rolldown_common::{ImportedSpecifier, ModuleId, Symbol};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::ecma::atoms::JsWord;

use crate::{treeshake::statement_part::Include, BundleError, NormalModule};

mod graph;
mod statement_part;

#[derive(Debug)]
struct TreeshakeContext<'a> {
  id_to_module: FxHashMap<&'a ModuleId, TreeshakeNormalModule<'a>>,
  pub(crate) errors: Mutex<Vec<rolldown_error::Error>>,
  pub(crate) deeps: AtomicUsize,
}

impl<'a> TreeshakeContext<'a> {
  pub(crate) fn add_error(&self, error: rolldown_error::Error) {
    self.errors.lock().unwrap().push(error);
  }

  pub(crate) fn inc_and_check_deep(&self) {
    let deep = self.deeps.fetch_add(1, Ordering::SeqCst);
    if deep > 1000 {
      panic!("Too deep");
    }
  }
}

#[derive(Debug)]
pub(crate) struct TreeshakeNormalModule<'m> {
  pub(crate) is_included: AtomicBool,
  pub(crate) module: &'m NormalModule,
  pub(crate) imported_as_symbol_to_importee_id: FxHashMap<&'m Symbol, &'m ModuleId>,
  pub(crate) imported_as_symbol_to_imported_specifier: FxHashMap<&'m Symbol, &'m ImportedSpecifier>,
}

impl<'m> TreeshakeNormalModule<'m> {
  fn is_entry(&self) -> bool {
    self.module.is_dynamic_entry || self.module.is_user_defined_entry
  }

  pub(crate) fn new(module: &'m NormalModule) -> Self {
    let imported_as_symbol_to_importee_id = module
      .linked_imports
      .iter()
      .flat_map(|(id, import)| import.into_iter().map(move |spec| (&spec.imported_as, id)))
      .collect();

    let imported_as_symbol_to_imported_specifier = module
      .linked_imports
      .iter()
      .flat_map(|(_id, import)| {
        import
          .into_iter()
          .map(move |spec| (&spec.imported_as, spec))
      })
      .collect();

    Self {
      is_included: Default::default(),
      module,
      imported_as_symbol_to_importee_id,
      imported_as_symbol_to_imported_specifier,
    }
  }

  fn try_define_by_declared_id(
    &self,
    ctx: &TreeshakeContext,
    symbol: &Symbol,
  ) -> Option<FxHashSet<Symbol>> {
    ctx.inc_and_check_deep();
    tracing::trace!(
      "try_define_by_declared_id: {:?} in {:?}",
      symbol,
      self.module.id
    );
    self
      .module
      .parts
      .find_parts_where_symbol_declared(symbol)
      .map(|parts| {
        parts
          .into_iter()
          .flat_map(|p| p.include(ctx, self))
          // There declared ids aren't declared in statements, but created by
          // create_top_level_symbol
          .chain([symbol.clone()])
          .collect()
      })
  }

  fn try_define_by_imported_alias_id(
    &self,
    ctx: &TreeshakeContext,
    id: &Symbol,
  ) -> Option<FxHashSet<Symbol>> {
    ctx.inc_and_check_deep();
    let importee_id = self.imported_as_symbol_to_importee_id.get(id)?;
    let mut included = FxHashSet::from_iter([id.clone()]);

    if importee_id.is_external() {
      Some(included)
    } else {
      let spec = self
        .imported_as_symbol_to_imported_specifier
        .get(id)
        .unwrap();
      let importee = ctx.id_to_module.get(importee_id).unwrap();
      included.extend(importee.define_by_exported_name(ctx, &spec.imported));
      Some(included)
    }
  }

  /// Three type of top level id
  /// 1. declared id by declaration, which stores in StatementParts
  /// 2. imported id by import declaration, which stores in linked_imports
  /// 3. included by generated namespace export, which is already linked in linked_imports
  fn define_by_top_level_id(
    &self,
    ctx: &TreeshakeContext,
    top_level_id: &Symbol,
  ) -> FxHashSet<Symbol> {
    ctx.inc_and_check_deep();
    tracing::trace!(
      "define_by_top_level_id: {:?} in {:?}",
      top_level_id,
      self.module.id
    );
    if let Some(res) = self.try_define_by_declared_id(ctx, top_level_id) {
      res
    } else if let Some(res) = self.try_define_by_imported_alias_id(ctx, top_level_id) {
      res
    } else {
      ctx.add_error(BundleError::panic(&format!(
        "top_level_id: {:?} is not found in {:?}",
        top_level_id, self.module.id
      )));
      Default::default()
    }
  }

  fn define_by_exported_name(
    &self,
    ctx: &TreeshakeContext,
    exported_name: &JsWord,
  ) -> FxHashSet<Symbol> {
    ctx.inc_and_check_deep();
    tracing::trace!(
      "define_by_exported_name: {:?} in {:?}",
      exported_name,
      self.module.id
    );
    if let Some(founded_spec) = self.module.find_exported(exported_name) {
      let is_local_export = founded_spec.owner == self.module.id;
      if is_local_export {
        self.define_by_top_level_id(ctx, &founded_spec.local_id)
      } else {
        ctx
          .id_to_module
          .get(&founded_spec.owner)
          .unwrap()
          .define_by_top_level_id(ctx, &founded_spec.local_id)
      }
    } else if !self.module.external_modules_of_re_export_all.is_empty() {
      // The symbol is maybe imported from external module, just ignore it
      Default::default()
    } else {
      ctx.add_error(BundleError::panic(&format!(
        " \"{:#?}\" is not exported from module {:?}",
        exported_name, self.module.id
      )));
      Default::default()
    }
  }

  fn include(&self, ctx: &TreeshakeContext) -> FxHashSet<Symbol> {
    if self.is_included.swap(true, Ordering::SeqCst) {
      Default::default()
    } else {
      let include_statements_having_side_effects = || {
        self
          .module
          .parts
          .parts
          .par_iter()
          .filter(|p| p.side_effect)
          .flat_map(|part| part.include(ctx, self))
          .collect::<FxHashSet<_>>()
      };

      let include_exports_if_is_entry = || {
        if self.is_entry() {
          self
            .module
            .linked_exports
            .keys()
            .par_bridge()
            .flat_map(|exported_name| self.define_by_exported_name(ctx, exported_name))
            .collect::<FxHashSet<_>>()
        } else {
          Default::default()
        }
      };

      let (mut included_ids, included_ids2) = rayon::join(
        include_statements_having_side_effects,
        include_exports_if_is_entry,
      );

      if !included_ids2.is_empty() {
        included_ids.extend(included_ids2);
      }

      tracing::debug!(
        "Include Module({})\nwith included symbols: {:?}",
        self.module.id,
        included_ids
      );

      included_ids
    }
  }
}
