use std::sync::{
  atomic::{AtomicBool, Ordering},
  Mutex,
};

use rayon::prelude::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rolldown_common::{ImportedSpecifier, ModuleId, Symbol};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::ecma::atoms::JsWord;

use crate::{treeshake::statement_part::Include, BuildError, NormalModule};

mod graph;
mod statement_part;

#[derive(Debug)]
struct TreeshakeContext<'a> {
  id_to_module: FxHashMap<&'a ModuleId, TreeshakeNormalModule<'a>>,
  pub(crate) errors: Mutex<Vec<rolldown_error::Error>>,
}

impl<'a> TreeshakeContext<'a> {
  pub(crate) fn add_error(&self, error: rolldown_error::Error) {
    self.errors.lock().unwrap().push(error);
  }

  // Use for debugging
  pub(crate) fn emit_warnning_for_debugging(&self, warning: rolldown_error::Error) {
    tracing::warn!("{}", warning);
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
      .flat_map(|(id, import)| import.iter().map(move |spec| (&spec.imported_as, id)))
      .collect();

    let imported_as_symbol_to_imported_specifier = module
      .linked_imports
      .iter()
      .flat_map(|(_id, import)| import.iter().map(move |spec| (&spec.imported_as, spec)))
      .collect();

    Self {
      is_included: Default::default(),
      module,
      imported_as_symbol_to_importee_id,
      imported_as_symbol_to_imported_specifier,
    }
  }

  fn define_symbol_created_by_declaration(
    &self,
    ctx: &TreeshakeContext,
    symbol: &Symbol,
  ) -> FxHashSet<Symbol> {
    self
      .module
      .parts
      .find_parts_where_symbol_declared(symbol)
      .map(|parts| {
        parts
          .into_iter()
          .flat_map(|p| p.include(ctx, self))
          .collect()
      })
      .expect("Must have declaration")
  }

  /// Return `None` if the symbol is not created by import
  fn define_symbol_created_by_import(
    &self,
    ctx: &TreeshakeContext,
    symbol: &Symbol,
  ) -> Option<FxHashSet<Symbol>> {
    // First, we need to find importee
    let importee_id = self.imported_as_symbol_to_importee_id.get(symbol)?;

    // Remember to include the symbol which we try to define
    let mut included = FxHashSet::from_iter([symbol.clone()]);
    let import_spec = self
      .imported_as_symbol_to_imported_specifier
      .get(symbol)
      .expect("Must have imported specifier");

    let mut maybe_the_definer = *importee_id;
    let mut imported_symbol_name = &import_spec.imported;
    let mut visited = vec![];
    let (definer, symbol_owner_defined) = loop {
      if maybe_the_definer.is_external() {
        return Some(included);
      }
      if visited.contains(&(maybe_the_definer, imported_symbol_name)) {
        ctx.emit_warnning_for_debugging(BuildError::circular_dependency(
          visited
            .iter()
            .map(|(module, _)| module.to_string())
            .collect(),
        ));
        return Some(included);
      } else {
        visited.push((maybe_the_definer, imported_symbol_name));
      }

      let exporter = ctx.id_to_module.get(&maybe_the_definer).unwrap();

      let Some(founded_export_spec) = exporter.module.find_exported(imported_symbol_name) else {
        if !exporter
        .module
        .external_modules_of_re_export_all
        .is_empty() {
          // The symbol is maybe imported from external module, just return the symbol itself
          return Some(included);
        } else {
          ctx.add_error(BuildError::panic(format!(
            " \"{:#?}\" is not exported from module {:?}",
            import_spec.imported, exporter.module.id
          )));
          // At least, we could be believe that symbols in `included` are used.
          return Some(included);
        }
      };

      let is_exporter_the_owner_of_export_spec = founded_export_spec.owner == exporter.module.id;

      if is_exporter_the_owner_of_export_spec {
        // The symbol may be defined in the exporter
        if exporter.is_declare_the_symbol(&founded_export_spec.local_id) {
          break (exporter, &founded_export_spec.local_id);
        } else {
          // The symbol in exporter created by import from other module
          let import_spec = exporter
            .imported_as_symbol_to_imported_specifier
            .get(&founded_export_spec.local_id)
            .expect("Must have imported specifier");
          let importee = exporter
            .imported_as_symbol_to_importee_id
            .get(&founded_export_spec.local_id)
            .expect("Must have importee");

          maybe_the_definer = *importee;
          imported_symbol_name = &import_spec.imported;
        }
      } else {
        // The exporter just re-export the symbol from other module
        maybe_the_definer = &founded_export_spec.owner;
        imported_symbol_name = &founded_export_spec.exported_as;
      }

      // Important: they are used, so we need to add it to included
      included.insert(founded_export_spec.local_id.clone());
    };

    included.extend(definer.define_by_top_level_symbol(ctx, symbol_owner_defined));

    Some(included)
  }

  fn is_declare_the_symbol(&self, symbol: &Symbol) -> bool {
    self
      .module
      .parts
      .find_parts_where_symbol_declared(symbol)
      .is_some()
  }

  /// Three type of top level id
  /// 1. declared id by declaration, which stores in StatementParts
  /// 2. imported id by import declaration, which stores in linked_imports
  /// 3. included by generated namespace export, which is already linked in linked_imports
  fn define_by_top_level_symbol(
    &self,
    ctx: &TreeshakeContext,
    top_level_symbol: &Symbol,
  ) -> FxHashSet<Symbol> {
    if self.is_declare_the_symbol(top_level_symbol) {
      self.define_symbol_created_by_declaration(ctx, top_level_symbol)
    } else if let Some(res) = self.define_symbol_created_by_import(ctx, top_level_symbol) {
      res
    } else {
      ctx.add_error(
        BuildError::panic(format!(
          "top_level_id: {:?} is not found in {:?}",
          top_level_symbol, self.module.id
        ))
        .context("Treeshake"),
      );
      Default::default()
    }
  }

  fn define_by_exported_name(
    &self,
    ctx: &TreeshakeContext,
    exported_name: &JsWord,
  ) -> FxHashSet<Symbol> {
    if let Some(founded_spec) = self.module.find_exported(exported_name) {
      let is_local_export = founded_spec.owner == self.module.id;
      if is_local_export {
        self.define_by_top_level_symbol(ctx, &founded_spec.local_id)
      } else {
        ctx
          .id_to_module
          .get(&founded_spec.owner)
          .unwrap()
          .define_by_top_level_symbol(ctx, &founded_spec.local_id)
      }
    } else if !self.module.external_modules_of_re_export_all.is_empty() {
      // The symbol is maybe imported from external module, just ignore it
      Default::default()
    } else {
      ctx.add_error(BuildError::panic(format!(
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

      included_ids
    }
  }
}
