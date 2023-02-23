use std::sync::Arc;

use derivative::Derivative;
use itertools::Itertools;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use rolldown_common::{ExportedSpecifier, ImportedSpecifier, ModuleId, Symbol, UnionFind, CWD};
use rolldown_resolver::Resolver;
use rustc_hash::FxHashSet as HashSet;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::common::{Mark, SyntaxContext, GLOBALS};
use swc_core::ecma::atoms::{js_word, JsWord};

use crate::module_loader::ModuleLoader;
use crate::{
  norm_or_ext::NormOrExt, normal_module::NormalModule, options::InputOptions, BuildResult,
  ModuleById, SWC_GLOBALS,
};
use crate::{BuildError, SharedBuildPluginDriver};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Graph {
  pub entries: Vec<ModuleId>,
  pub(crate) module_by_id: ModuleById,
  pub(crate) unresolved_mark: Mark,
  pub(crate) unresolved_ctxt: SyntaxContext,
  #[derivative(Debug = "ignore")]
  pub(crate) uf: UnionFind<Symbol>,
  pub(crate) build_plugin_driver: SharedBuildPluginDriver,
  pub(crate) warnings: Vec<rolldown_error::Error>,
  pub(crate) used_symbols: HashSet<Symbol>,
}

impl Graph {
  pub(crate) fn new(build_plugin_driver: SharedBuildPluginDriver) -> Self {
    let (unresolved_mark, unresolved_ctxt) = GLOBALS.set(&SWC_GLOBALS, || {
      let mark = Mark::new();
      let ctxt = SyntaxContext::empty().apply_mark(mark);
      (mark, ctxt)
    });

    Self {
      entries: Default::default(),
      module_by_id: Default::default(),
      unresolved_mark,
      unresolved_ctxt,
      uf: Default::default(),
      build_plugin_driver,
      warnings: Default::default(),
      used_symbols: Default::default(),
    }
  }

  fn fetch_module<'m>(module_by_id: &'m ModuleById, id: &ModuleId) -> &'m NormOrExt {
    module_by_id
      .get(id)
      .unwrap_or_else(|| panic!("Failed to fetch module: {id:?}"))
  }

  fn fetch_normal_module<'m>(module_by_id: &'m ModuleById, id: &ModuleId) -> &'m NormalModule {
    Self::fetch_module(module_by_id, id)
      .as_norm()
      .unwrap_or_else(|| panic!("Expected NormalModule, got ExternalModule({id:?})"))
  }

  fn fetch_module_mut<'m>(module_by_id: &'m mut ModuleById, id: &ModuleId) -> &'m mut NormOrExt {
    module_by_id
      .get_mut(id)
      .unwrap_or_else(|| panic!("Failed to fetch module: {id:?}"))
  }

  fn fetch_normal_module_mut<'m>(
    module_by_id: &'m mut ModuleById,
    id: &ModuleId,
  ) -> &'m mut NormalModule {
    Self::fetch_module_mut(module_by_id, id)
      .as_norm_mut()
      .unwrap_or_else(|| panic!("Expected NormalModule, got ExternalModule({id:?})"))
  }

  pub(crate) fn add_module(&mut self, module: NormOrExt) {
    debug_assert!(!self.module_by_id.contains_key(module.id()));
    self.module_by_id.insert(module.id().clone(), module);
  }

  #[tracing::instrument(skip_all)]
  fn sort_modules(&mut self) {
    enum Action {
      Enter,
      Exit,
    }
    type Queue = Vec<(Action, ModuleId)>;
    let mut queue = self
      .entries
      .iter()
      .map(|dep| self.module_by_id.get(dep).unwrap())
      .map(|module| (Action::Enter, module.id().clone()))
      .rev()
      .collect::<Vec<_>>();

    let mut entered_ids: HashSet<ModuleId> = FxHashSet::default();
    let mut next_exec_order = 0;
    let mut dynamic_entries = vec![];

    let mut walk = |queue: &mut Queue, mut dynamic_entries: Option<&mut Queue>| {
      while let Some((action, id)) = queue.pop() {
        match action {
          Action::Enter => {
            let module = self.module_by_id.get(&id).unwrap();
            if !entered_ids.contains(&id) {
              entered_ids.insert(id.clone());
              queue.push((Action::Exit, id.clone()));
              module
                .dependencies()
                .into_iter()
                .rev()
                // Early filter modules that are already entered
                .filter(|id| !entered_ids.contains(id))
                .for_each(|dep| {
                  queue.push((Action::Enter, dep.clone()));
                });
              if let Some(dynamic_entries) = dynamic_entries.as_mut() {
                module
                  .dynamic_dependencies()
                  .into_iter()
                  // Early filter modules that are already entered
                  .filter(|module| !entered_ids.contains(module))
                  .for_each(|dep| {
                    dynamic_entries.push((Action::Enter, dep.clone()));
                  });
              }
            }
          }
          Action::Exit => {
            self
              .module_by_id
              .get_mut(&id)
              .unwrap()
              .set_exec_order(next_exec_order);
            next_exec_order += 1;
          }
        }
      }
    };

    walk(&mut queue, Some(&mut dynamic_entries));
    walk(&mut dynamic_entries, None);
    tracing::debug!(
      "sorted modules {:#?}",
      self
        .module_by_id
        .values()
        .sorted_by_key(|m| {
          assert_ne!(m.exec_order(), usize::MAX);
          m.exec_order()
        })
        .map(|m| m.id())
        .collect_vec()
    );
  }

  fn link(&mut self) -> BuildResult<()> {
    let mut order_modules = self
      .module_by_id
      .values()
      .map(|module| module.id().clone())
      .collect::<Vec<_>>();
    order_modules.sort_unstable_by_key(|id| self.module_by_id[id].exec_order());

    self.link_exports(&order_modules)?;
    self.link_imports(&order_modules)?;

    Ok(())
  }

  /// Example
  /// ```ts
  /// // index.ts
  /// export { foo } from "./foo.ts";
  /// export { bar } from "./bar.ts";
  /// ```
  /// If `index.js` is importer, `foo.ts` and `bar.ts` are importee.
  /// `foo` and `bar` are `ReExportedSpecifier`s.
  fn link_exports(&mut self, order_modules: &[ModuleId]) -> BuildResult<()> {
    order_modules
      .iter()
      .filter(|importer_id| {
        // Fast path
        !importer_id.is_external()
      })
      .try_for_each(|importer_id| -> BuildResult<()> {
        let importee_and_re_exports = Self::fetch_normal_module(&self.module_by_id, importer_id)
          .re_exported_ids
          .iter()
          .map(|(importee_id, re_exported_specifier)| {
            (importee_id.clone(), re_exported_specifier.clone())
          })
          .collect::<Vec<_>>();

        importee_and_re_exports.into_iter().try_for_each(
          |(importee_id, re_exports)| -> BuildResult<()> {
            if importer_id == &importee_id {
              let importee = Self::fetch_normal_module_mut(&mut self.module_by_id, &importee_id);
              re_exports
                .into_iter()
                .try_for_each(|spec| -> BuildResult<()> {
                  importee.suggest_name(&spec.imported, &spec.exported_as);
                  if spec.imported == js_word!("*") {
                    importee.mark_namespace_id_referenced();
                  }
                  let original_spec = importee.find_exported(&spec.imported).unwrap();
                  importee.add_to_linked_exports(spec.exported_as, original_spec.clone());
                  Ok(())
                })?;

              return Ok(());
            };

            let [importer, importee] = self
              .module_by_id
              .get_many_mut([importer_id, &importee_id])
              .unwrap();

            let importer = importer.expect_norm_mut();
            match importee {
              NormOrExt::Normal(importee) => {
                re_exports
                  .into_iter()
                  .try_for_each(|spec| -> BuildResult<()> {
                    importee.suggest_name(&spec.imported, &spec.exported_as);
                    // Case: export * as foo from './foo'
                    if spec.imported == js_word!("*") {
                      importee.mark_namespace_id_referenced();
                    }
                    let original_spec = importee
                      .find_exported(&spec.imported)
                      .ok_or_else(|| BuildError::panic(format!("original_id not found: {spec:?}")))?
                      .clone();
                    importer.add_to_linked_exports(spec.exported_as, original_spec);
                    Ok(())
                  })?
              }
              NormOrExt::External(importee) => {
                // We will transform
                // ```js
                // export { resolve } from 'path'
                // ```
                // to
                // ```
                // import { resolve } from 'path'
                // export { resolve }
                // ```
                re_exports.into_iter().for_each(|spec| {
                  let symbol_in_importer =
                    importer.create_top_level_symbol(if spec.exported_as != js_word!("default") {
                      &spec.exported_as
                    } else {
                      &spec.imported
                    });

                  importer
                    .imports
                    .entry(importee.id.clone())
                    .or_default()
                    .insert(ImportedSpecifier {
                      imported_as: symbol_in_importer.clone(),
                      imported: spec.imported.clone(),
                    });
                  importer.add_to_linked_exports(
                    spec.exported_as.clone(),
                    ExportedSpecifier {
                      exported_as: spec.exported_as,
                      local_id: symbol_in_importer,
                      /// NOTE: This is a local export to importer
                      owner: importer_id.clone(),
                    },
                  )
                });
              }
            }
            Ok(())
          },
        )?;

        // Process re-export all

        let importee_of_being_re_exported_all =
          Self::fetch_normal_module_mut(&mut self.module_by_id, importer_id)
            .re_export_all
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        let non_conflicted_names = {
          use std::collections::hash_map::Entry;
          let mut tmp: FxHashMap<&JsWord, Option<&ExportedSpecifier>> = FxHashMap::default();
          importee_of_being_re_exported_all
            .iter()
            .filter_map(|importee_id| Self::fetch_module(&self.module_by_id, importee_id).as_norm())
            .flat_map(|each_importee| each_importee.linked_exports.iter())
            .for_each(|(exported_name, spec)| match tmp.entry(exported_name) {
              Entry::Occupied(mut entry) => {
                match entry.get() {
                  Some(existed_spec) => {
                    // The name is not first seen, we need to check if the specifiers are the same
                    if *existed_spec == spec {
                      // The specifiers are the same, so it's ok
                    } else {
                      // Mark the name as conflicted
                      entry.insert(None);
                    }
                  }
                  None => {
                    // Already conflicted, just ignore the name
                  }
                }
              }
              Entry::Vacant(entry) => {
                // The name is first seen, so it's ok
                entry.insert(Some(spec));
              }
            });
          tmp
            .into_iter()
            .filter_map(|(name, spec)| spec.map(|_| name.clone()))
            .collect::<FxHashSet<_>>()
        };

        let importer = Self::fetch_module(&self.module_by_id, importer_id).expect_norm();

        let explicit_exported_names_of_importer = importer
          .linked_exports
          .keys()
          .cloned()
          .collect::<FxHashSet<_>>();

        importee_of_being_re_exported_all
          .iter()
          .for_each(|importee_id| {
            // It seems meaningless to re-export all from itself
            if importee_id == importer_id {
              return;
            }

            let [importer, importee] = self
              .module_by_id
              .get_many_mut([importer_id, importee_id])
              .unwrap();
            let importer = importer.expect_norm_mut();

            if let NormOrExt::Normal(importee) = importee {
              importee.re_export_all.iter().for_each(|id| {
                importer.re_export_all.get_or_insert(id.clone());
              });
            }

            match importee {
              NormOrExt::Normal(importee) => {
                importee
                  .linked_exports
                  .clone()
                  .into_iter()
                  .filter(|(name, _)| {
                    // export * from ... does not re-export `default`
                    let is_default_export = name == "default";
                    !is_default_export
                  })
                  .filter(|(name, _)| {
                    // explicit named export has higher priority than names from re-export-all
                    let is_already_exported_explicitly =
                      explicit_exported_names_of_importer.contains(name);

                    !is_already_exported_explicitly
                  })
                  .filter(|(exported_as, _spec)| {
                    // Conflicted names should be hidden
                    non_conflicted_names.contains(exported_as)
                  })
                  .for_each(|(exported_as, spec)| {
                    importer.add_to_linked_exports(exported_as, spec);
                  });

                // Handle case
                // ```ts
                // // index.ts
                // export * from "./foo.ts";
                // // foo.ts
                // export * from "./bar.ts";
                // export * from "external";
                // ```
                importee.re_export_all.iter().for_each(|id| {
                  importer.re_export_all.get_or_insert(id.clone());
                });
              }
              NormOrExt::External(_importee) => {
                // Handle case
                // ```js
                // // index.js
                // import * as foo from './foo'
                // console.log(foo)
                // // foo.js
                // export * from 'external'
                // ```
                // will be transformed to
                // ```js
                // // foo.js
                // import * as external from 'external'
                // const foo = _mergeNamespace({ __proto__: null}, [external])
                // // index.js
                // console.log(foo)
                // ```
                // TODO: We might need to check if the importer is a already import star from importee
                importer
                  .external_modules_of_re_export_all
                  .get_or_insert(importee_id.clone());
              }
            }
          });
        Ok(())
      })
  }

  /// two things
  /// 1. Union symbol
  /// 2. Generate real ImportedSpecifier for each import and add to `linked_imports`
  fn link_imports(&mut self, order_modules: &[ModuleId]) -> BuildResult<()> {
    order_modules
      .iter()
      .filter(|importer_id| !importer_id.is_external())
      .try_for_each(|importer_id| -> BuildResult<()> {
        tracing::trace!("link_imports for importer {}", importer_id);
        let importee_and_specifiers = Self::fetch_normal_module(&self.module_by_id, importer_id)
          .imports
          .clone()
          .into_iter()
          .collect::<Vec<_>>();

        importee_and_specifiers.into_iter().try_for_each(
          |(importee_id, specs)| -> BuildResult<()> {
            tracing::trace!("link_imports for importee {}", importee_id);
            if importer_id == &importee_id {
              // Handle self import
              let importee = Self::fetch_normal_module_mut(&mut self.module_by_id, &importee_id);

              for imported_spec in specs {
                if &imported_spec.imported == "*" {
                  importee.mark_namespace_id_referenced();
                }
                importee.suggest_name(&imported_spec.imported, imported_spec.imported_as.name());

                if let Some(exported_spec) =
                  importee.find_exported(&imported_spec.imported).cloned()
                {
                  self
                    .uf
                    .union(&imported_spec.imported_as, &exported_spec.local_id);

                  // The importee is also the importer
                  importee
                    .linked_imports
                    .entry(exported_spec.owner.clone())
                    .or_default()
                    .insert(ImportedSpecifier {
                      imported_as: imported_spec.imported_as.clone(),
                      imported: exported_spec.exported_as.clone(),
                    });
                } else {
                  return Err(BuildError::missing_export(
                    &imported_spec.imported,
                    importer_id.as_ref(),
                    importee_id.as_ref(),
                  ));
                }
              }
              return Ok(());
            }
            let [importer, importee] = self
              .module_by_id
              .get_many_mut([importer_id, &importee_id])
              .unwrap();
            let importer = importer.expect_norm_mut();

            for imported_spec in specs {
              match importee {
                NormOrExt::Normal(importee) => {
                  if &imported_spec.imported == "*" {
                    importee.mark_namespace_id_referenced();
                  }
                  importee.suggest_name(&imported_spec.imported, imported_spec.imported_as.name());
                  if let Some(exported_spec) =
                    importee.find_exported(&imported_spec.imported).cloned()
                  {
                    tracing::trace!(
                      "union alias:{:?}, original:{:?}",
                      imported_spec.imported_as,
                      exported_spec
                    );
                    self
                      .uf
                      .union(&imported_spec.imported_as, &exported_spec.local_id);

                    importer
                      .linked_imports
                      // Redirect to the owner of the exported symbol
                      .entry(exported_spec.owner.clone())
                      .or_default()
                      .insert(ImportedSpecifier {
                        imported_as: imported_spec.imported_as.clone(),
                        imported: exported_spec.exported_as.clone(),
                      });
                  } else if let Some(first_external_id) = importee
                    .external_modules_of_re_export_all
                    .iter()
                    .next()
                    .cloned()
                  {
                    if importee.external_modules_of_re_export_all.len() > 1 {
                      self
                        .warnings
                        .push(BuildError::ambiguous_external_namespaces(
                          imported_spec.imported_as.name().to_string(),
                          importer_id.to_string(),
                          first_external_id.to_string(),
                          importee
                            .external_modules_of_re_export_all
                            .iter()
                            .map(|id| id.to_string())
                            .collect_vec(),
                        ))
                    }

                    let symbol_in_importee =
                      importee.create_top_level_symbol(imported_spec.imported_as.name());
                    importee
                      .linked_imports
                      .entry(first_external_id.clone())
                      .or_default()
                      .insert(ImportedSpecifier {
                        imported: imported_spec.imported.clone(),
                        imported_as: symbol_in_importee.clone(),
                      });

                    importee.add_to_linked_exports(
                      imported_spec.imported.clone(),
                      ExportedSpecifier {
                        exported_as: imported_spec.imported.clone(),
                        local_id: symbol_in_importee.clone(),
                        owner: importee_id.clone(),
                      },
                    );
                    importer
                      .linked_imports
                      .entry(importee_id.clone())
                      .or_default()
                      .insert(ImportedSpecifier {
                        imported: imported_spec.imported.clone(),
                        imported_as: imported_spec.imported_as.clone(),
                      });

                    self
                      .uf
                      .union(&imported_spec.imported_as, &symbol_in_importee);
                  } else {
                    return Err(BuildError::missing_export(
                      &imported_spec.imported,
                      importer_id.as_ref(),
                      importee_id.as_ref(),
                    ));
                  };
                }
                NormOrExt::External(importee) => {
                  importer
                    .linked_imports
                    .entry(importee_id.clone())
                    .or_default()
                    .insert(imported_spec.clone());
                  let exported_symbol_of_importee =
                    importee.find_exported_symbol(&imported_spec.imported);

                  self
                    .uf
                    .union(&imported_spec.imported_as, exported_symbol_of_importee);
                }
              }
            }

            Ok(())
          },
        )
      })
  }

  /// In the function, we will:
  /// 1. TODO: More delicate analysis of import/export star for cross-module namespace export
  /// Only after linking, we can know which imported symbol is "namespace symbol" or declared by user.
  /// 2. Generate actual namespace export AST for each module whose namespace is referenced
  fn patch(&mut self) {
    use rayon::prelude::*;
    self
      .module_by_id
      .values_mut()
      .par_bridge()
      .for_each(|module| {
        if let NormOrExt::Normal(module) = module {
          module.generate_namespace_export();
        }
      });
  }

  pub(crate) async fn build(&mut self, input_opts: &InputOptions) -> BuildResult<()> {
    let resolver = Arc::new(Resolver::with_cwd(input_opts.cwd.clone()));

    ModuleLoader::new(self, resolver, self.build_plugin_driver.clone(), input_opts)
      .fetch_all_modules(input_opts)
      .await?;

    self.sort_modules();
    CWD.set(&input_opts.cwd, || self.link())?;
    tracing::debug!("link done, graph: {:#?}", self);
    self.patch();

    if input_opts.treeshake {
      self.treeshake()?;
    } else {
      self
        .module_by_id
        .values_mut()
        .par_bridge()
        .for_each(|module| {
          match module {
            NormOrExt::Normal(module) => {
              // Because of scope hoisting, we need to remove export/import
              rolldown_swc_visitors::remove_export_and_import(&mut module.ast);
            }
            NormOrExt::External(_ext) => {}
          };
        });
    }
    Ok(())
  }
}
