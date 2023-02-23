use std::collections::HashSet;

use hashlink::LinkedHashSet;
use itertools::Itertools;
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rolldown_common::{ChunkId, ExportedSpecifier, ImportedSpecifier, ModuleId, Symbol, UnionFind};
use rolldown_runtime_helpers::RuntimeHelpers;
use rolldown_swc_visitors::FinalizeContext;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::{
  common::{comments::SingleThreadedComments, util::take::Take, Mark, SyntaxContext, GLOBALS},
  ecma::{
    ast::{self, Id, Ident},
    atoms::{js_word, JsWord},
    utils::{quote_ident, quote_str},
    visit::VisitMutWith,
  },
};

use crate::{
  file_name, norm_or_ext::NormOrExt, preset_of_used_names, BuildError, BuildResult, ExportMode,
  Graph, InputOptions, MergedExports, ModuleById, ModuleRefMutById, OutputOptions,
  SplitPointIdToChunkId, COMPILER,
};

pub struct Chunk {
  pub(crate) export_mode: ExportMode,
  pub(crate) id: ChunkId,
  pub(crate) filename: Option<String>,
  pub(crate) entry: ModuleId,
  pub(crate) modules: HashSet<ModuleId>,
  pub(crate) before_module_items: Vec<ast::ModuleItem>,
  pub(crate) after_module_items: Vec<ast::ModuleItem>,
  pub(crate) runtime_helpers: RuntimeHelpers,
  pub(crate) is_user_defined_entry: bool,
}

impl Chunk {
  pub fn new(id: impl Into<ChunkId>, entry: ModuleId, is_user_defined_entry: bool) -> Self {
    Self {
      export_mode: ExportMode::Named,
      id: id.into(),
      modules: Default::default(),
      entry,
      before_module_items: Default::default(),
      after_module_items: Default::default(),
      filename: None,
      runtime_helpers: Default::default(),
      is_user_defined_entry,
    }
  }

  pub(crate) fn gen_file_name(&mut self, output_options: &OutputOptions) {
    self.filename = Some(
      output_options
        .entry_file_names
        .render(file_name::RenderOptions {
          name: Some(self.id.as_ref()),
        }),
    )
  }

  fn ordered_modules<'m>(&self, module_by_id: &'m ModuleById) -> Vec<&'m NormOrExt> {
    let mut modules = self
      .modules
      .iter()
      .map(|id| module_by_id.get(id).unwrap())
      .collect::<Vec<_>>();

    modules.sort_by_key(|m| m.exec_order());

    modules
  }

  pub(crate) fn render(
    &self,
    ctx: RenderContext,
    graph: &Graph,
    input_options: &InputOptions,
    output_options: &OutputOptions,
  ) -> BuildResult<String> {
    let mut runtime_code = self.runtime_helpers.generate_helpers().join("\n");
    runtime_code.push('\n');

    let before_code = self
      .before_module_items
      .iter()
      .map(|item| COMPILER.print_module_item(item, None).unwrap())
      .join("\n");

    let after_code = self
      .after_module_items
      .iter()
      .map(|item| COMPILER.print_module_item(item, None).unwrap())
      .join("\n");

    let code = self
      .ordered_modules(&graph.module_by_id)
      .iter()
      .filter_map(|m| m.as_norm())
      .filter(|m| m.is_included())
      .map(|module| module.render(&ctx, input_options))
      .collect::<Vec<_>>()
      .join("\n");

    let mut code = before_code + &runtime_code + &code + &after_code;

    if output_options.format.is_cjs() {
      let filename = format!("{}.js", self.id.value());
      // Workaround for cjs output
      let comments = SingleThreadedComments::default();
      let mut program = COMPILER
        .parse_with_comments(code.clone(), &filename, Some(&comments))
        .1
        .map_err(|_| {
          BuildError::panic(format!(
            "Failed to parse generated code \n{}\n for {}",
            code, &self.entry
          ))
        })?;

      program = GLOBALS.set(&Default::default(), || {
        rolldown_swc_visitors::to_cjs(
          program,
          Mark::new(),
          &comments,
          self.export_mode.is_default() && self.is_user_defined_entry,
        )
      });

      code = COMPILER.print(&program, Some(&comments)).unwrap()
    }
    Ok(code)
  }

  /// Deconflicting is to rename identifiers to avoid conflicts.
  ///
  pub(crate) fn deconflict(&mut self, ctx: &mut FinalizeBundleContext) -> FxHashMap<Id, JsWord> {
    let mut ordered_modules = {
      let mut modules = ctx.modules.values_mut().collect::<Vec<_>>();
      modules.sort_by_key(|m| m.exec_order());
      modules
    };

    let uf = ctx.uf;

    let mut used_names = ordered_modules
      .iter()
      .filter_map(|m| m.as_norm().map(|m| m.visited_global_names.clone()))
      .flatten()
      .collect::<FxHashSet<_>>();

    used_names.extend(preset_of_used_names(&ctx.output_options.format));

    let mut id_to_name = FxHashMap::default();
    let mut root_id_to_name = FxHashMap::default();

    let mut create_conflictless_name = |original: JsWord| {
      let mut name = original.clone();
      let mut count = 1;
      while used_names.contains(&name) {
        name = format!("{}${}", &original, &count).into();
        count += 1;
      }
      used_names.insert(name.clone());
      name
    };

    let mut finalize_name_of_id = |id: Symbol, suggest_name: Option<JsWord>| {
      let root_id = uf
        .find_root_par(&id)
        .unwrap_or_else(|| panic!("Failed to find root of {:?}", id));
      let final_name = root_id_to_name.entry(root_id.to_id()).or_insert_with(|| {
        create_conflictless_name(suggest_name.unwrap_or_else(|| id.name().clone()))
      });

      debug_assert!(
        Ident::verify_symbol(final_name).is_ok(),
        "\"{final_name:?}\" is invalid",
      );
      final_name.clone()
    };

    // De-conflict from the entry module to keep namings as simple as possible
    ordered_modules.iter().rev().for_each(|norm_or_ext| {
      match norm_or_ext {
        NormOrExt::Normal(module) => {
          let declared_ids = module.parts.declared_ids();
          declared_ids.into_iter().for_each(|declared_id| {
            let suggested_name = module.suggested_name_for(declared_id.name());
            let final_name = finalize_name_of_id(declared_id.clone(), suggested_name);
            id_to_name.insert(declared_id.clone().to_id(), final_name);
          });

          module
            .linked_imports
            .iter()
            .filter(|(importee_id, _)| importee_id.is_external())
            .flat_map(|(_, specs)| specs.iter())
            .for_each(|spec| {
              let suggested_name = module.suggested_name_for(spec.imported_as.name());
              let final_name = finalize_name_of_id(spec.imported_as.clone(), suggested_name);
              id_to_name.insert(spec.imported_as.clone().to_id(), final_name);
            });
        }
        NormOrExt::External(_ext_module) => {
          // We don't care external module
        }
      }
    });

    let get_final_name = |id: &Symbol| {
      let root_id = uf.find_root_par(id).unwrap();
      let final_name = root_id_to_name.get(root_id.as_id());
      final_name
    };

    // Create a map of id -> name. So we could get a name of id with o(1) cost.
    ordered_modules
      .iter_mut()
      .for_each(|mod_or_ext| match mod_or_ext {
        NormOrExt::Normal(module) => {
          id_to_name.extend(
            module
              .imports
              .values()
              .flatten()
              .map(|spec_id| &spec_id.imported_as)
              .chain(module.local_exports.values().map(|spec| &spec.local_id))
              .chain(
                module
                  .is_facade_namespace_id_referenced
                  .then_some(&module.facade_id_for_namespace.local_id),
              )
              .filter_map(|declared_id| {
                get_final_name(declared_id)
                  .map(|final_name| (declared_id.clone().to_id(), final_name.clone()))
              }),
          );
        }
        NormOrExt::External(_ext_module) => {}
      });

    id_to_name
  }

  pub(crate) fn finalize(&mut self, mut ctx: FinalizeBundleContext) -> BuildResult<()> {
    self.generate_cross_chunk_links(&mut ctx)?;
    let ordered_modules = {
      let mut modules = ctx.modules.values_mut().collect::<Vec<_>>();
      modules.sort_by_key(|m| m.exec_order());
      modules
    };

    let top_level_ctxt_set = ordered_modules
      .iter()
      .map(|m| m.top_level_ctxt())
      .collect::<FxHashSet<_>>();

    let declared_scoped_names = ordered_modules
      .iter()
      .filter_map(|m| m.as_norm().map(|m| m.declared_scoped_names.clone()))
      .flatten()
      .collect::<FxHashSet<_>>();

    let id_to_name = self.deconflict(&mut ctx);

    tracing::debug!("id_to_name: {:#?}", id_to_name);

    let ordered_modules = {
      let mut modules = ctx.modules.values_mut().collect::<Vec<_>>();
      modules.sort_by_key(|m| m.exec_order());
      modules
    };

    {
      // Finalize module items in chunk
      let finalize_ctx = FinalizeContext {
        chunk_filename_by_id: ctx.chunk_filename_by_id,
        // Since there's no dynamic import expressions to rewrite, we can use empty set.
        resolved_ids: &Default::default(),
        // No scoped names to rewrite
        declared_scoped_names: &Default::default(),
        unresolved_ctxt: ctx.unresolved_ctxt,
        top_level_ctxt_set: &top_level_ctxt_set,
      };

      self
        .before_module_items
        .visit_mut_with(&mut rolldown_swc_visitors::finalizer(
          &id_to_name,
          ctx.split_point_id_to_chunk_id,
          finalize_ctx,
        ));
      let finalize_ctx = FinalizeContext {
        chunk_filename_by_id: ctx.chunk_filename_by_id,
        // Since there's no dynamic import expressions to rewrite, we can use empty set.
        resolved_ids: &Default::default(),
        // No scoped names to rewrite
        declared_scoped_names: &Default::default(),
        unresolved_ctxt: ctx.unresolved_ctxt,
        top_level_ctxt_set: &top_level_ctxt_set,
      };
      self
        .after_module_items
        .visit_mut_with(&mut rolldown_swc_visitors::finalizer(
          &id_to_name,
          ctx.split_point_id_to_chunk_id,
          finalize_ctx,
        ));
    }

    ordered_modules
      .into_par_iter()
      .filter_map(|m| m.as_norm_mut())
      .for_each(|m| {
        let finalize_ctx = FinalizeContext {
          chunk_filename_by_id: ctx.chunk_filename_by_id,
          resolved_ids: &m.resolved_module_ids,
          declared_scoped_names: &declared_scoped_names,
          unresolved_ctxt: ctx.unresolved_ctxt,
          top_level_ctxt_set: &top_level_ctxt_set,
        };

        m.ast.visit_mut_with(&mut rolldown_swc_visitors::finalizer(
          &id_to_name,
          ctx.split_point_id_to_chunk_id,
          finalize_ctx,
        ));
      });
    Ok(())
  }

  /// We only care about modules out of the chunk.
  /// - ExternalModule are considered out of the chunk.
  /// - NormalModule in other chunks are considered out of the chunk.
  fn depended_modules<'m>(&self, ordered_modules: &[&'m &'m mut NormOrExt]) -> Vec<&'m ModuleId> {
    let dependencies_of_chunk = {
      let mut deps = LinkedHashSet::new();
      let is_out_of_chunk = |id: &ModuleId| id.is_external() || !self.modules.contains(id);
      ordered_modules.iter().rev().for_each(|m| match m {
        NormOrExt::Normal(m) => {
          m.dependencies
            .iter()
            .filter(|id| is_out_of_chunk(id))
            .for_each(|dep| {
              if !deps.contains(dep) {
                deps.insert(dep);
              }
            });
        }
        NormOrExt::External(m) => {
          if is_out_of_chunk(&m.id) && !deps.contains(&m.id) {
            deps.insert(&m.id);
          }
        }
      });
      deps.into_iter().collect_vec()
    };
    dependencies_of_chunk
  }

  /// For generating correct exports in the chunk, we only need to care about the `linked_exports` in
  /// entry module. In linking phase, all needed exports are merged to `linked_exports` of entry module.
  pub(crate) fn generate_cross_chunk_links(
    &mut self,
    ctx: &mut FinalizeBundleContext,
  ) -> BuildResult<()> {
    let ordered_modules = {
      let mut modules = ctx.modules.values().collect::<Vec<_>>();
      modules.sort_by_key(|m| m.exec_order());
      modules
    };
    tracing::trace!(
      "ordered_modules: {:#?}",
      ordered_modules.iter().map(|m| m.id()).collect::<Vec<_>>()
    );

    let depended_modules = self.depended_modules(&ordered_modules);

    tracing::trace!("dependencies_of_chunk: {:#?}", depended_modules);

    // Merge imports coming from the same module.
    let mut imports_map: FxHashMap<&ModuleId, HashSet<&ImportedSpecifier>> = FxHashMap::default();
    ordered_modules
      .into_iter()
      .for_each(|norm_or_ext| match norm_or_ext {
        NormOrExt::Normal(module) => {
          module
            .linked_imports
            .iter()
            .for_each(|(importee, specifiers)| {
              imports_map
                .entry(importee)
                .or_default()
                .extend(specifiers.iter());
            });
        }
        NormOrExt::External(_) => {}
      });

    let entry_module = ctx.modules.get(&self.entry).unwrap().as_norm().unwrap();

    // If the owner of ExportedSpecifier isn't in the chunk, the export is considered in scope.
    let mut exports_in_scope: MergedExports = FxHashMap::default();
    let mut exports_out_scope: FxHashMap<&ModuleId, FxHashMap<&JsWord, &ExportedSpecifier>> =
      FxHashMap::default();
    let re_export_all: FxHashSet<&ModuleId> = entry_module
      .re_export_all
      .iter()
      .filter(|id| id.is_external())
      .collect();

    entry_module
      .linked_exports
      .iter()
      .for_each(|(exported_name, spec)| {
        if self.modules.contains(&spec.owner) && !spec.owner.is_external() {
          exports_in_scope.insert(exported_name.clone(), spec.clone());
        } else {
          exports_out_scope
            .entry(&spec.owner)
            .or_default()
            .insert(exported_name, spec);
        }
      });

    // imports and re-exports
    let module_items = depended_modules
      .par_iter()
      .flat_map(|chunk_dep_id| {
        let mut imported = false;
        let mut module_items = vec![];
        let src = if chunk_dep_id.is_external() {
          box quote_str!(chunk_dep_id.id())
        } else {
          let dep_chunk_id = ctx
            .split_point_id_to_chunk_id
            .get(chunk_dep_id)
            .unwrap_or_else(|| {
              panic!(
                "Cannot find chunk for split-point {} in Chunk({:?})",
                chunk_dep_id.id(),
                self.id
              )
            });
          let imported_chunk_filename = ctx.chunk_filename_by_id.get(dep_chunk_id).unwrap();
          box quote_str!(format!("./{imported_chunk_filename}"))
        };
        if let Some(specifiers) = imports_map.get(chunk_dep_id) {
          let mut specifiers = specifiers
            .iter()
            .map(|spec| (&spec.imported, spec))
            .collect::<FxHashMap<_, _>>();

          if let Some(star_specifier) = specifiers.remove(&js_word!("*")) {
            imported = true;
            module_items.push(ast::ModuleItem::ModuleDecl(ast::ModuleDecl::Import(
              ast::ImportDecl {
                src: src.clone(),
                specifiers: vec![ast::ImportSpecifier::Namespace(
                  ast::ImportStarAsSpecifier {
                    local: Ident::from(star_specifier.imported_as.clone().to_id()),
                    span: Default::default(),
                  },
                )],
                ..ast::ImportDecl::dummy()
              },
            )));
          }

          if !specifiers.is_empty() {
            imported = true;
            module_items.push(ast::ModuleItem::ModuleDecl(ast::ModuleDecl::Import(
              ast::ImportDecl {
                src: src.clone(),
                specifiers: specifiers
                  .into_values()
                  .sorted_by_key(|spec| &spec.imported)
                  .map(|spec| {
                    if spec.imported == js_word!("default") {
                      ast::ImportSpecifier::Default(ast::ImportDefaultSpecifier {
                        local: spec.imported_as.clone().to_id().into(),
                        span: Default::default(),
                      })
                    } else {
                      ast::ImportSpecifier::Named(ast::ImportNamedSpecifier {
                        local: Ident::from(spec.imported_as.clone().to_id()),
                        imported: Some(quote_ident!(spec.imported.clone()).into()),
                        span: Default::default(),
                        is_type_only: false,
                      })
                    }
                  })
                  .collect(),
                ..ast::ImportDecl::dummy()
              },
            )))
          }
        }
        if let Some(specifiers) = exports_out_scope.get(chunk_dep_id) {
          imported = true;
          module_items.push(ast::ModuleItem::ModuleDecl(ast::ModuleDecl::ExportNamed(
            ast::NamedExport {
              src: Some(src.clone()),
              span: Default::default(),
              specifiers: specifiers
                .iter()
                .sorted_by_key(|(exported_name, _)| *exported_name)
                .map(|(exported_name, spec_id)| {
                  if exported_name == &"*" {
                    ast::ExportSpecifier::Namespace(ast::ExportNamespaceSpecifier {
                      span: Default::default(),
                      name: quote_ident!(spec_id.local_id.name()).into(),
                    })
                  } else {
                    ast::ExportSpecifier::Named(ast::ExportNamedSpecifier {
                      span: Default::default(),
                      orig: ast::ModuleExportName::Ident(spec_id.local_id.clone().to_id().into()),
                      exported: (*exported_name != spec_id.local_id.name())
                        .then(|| quote_ident!((*exported_name).clone()).into()),
                      is_type_only: false,
                    })
                  }
                })
                .collect(),
              type_only: false,
              asserts: None,
            },
          )))
        }
        if re_export_all.contains(chunk_dep_id) {
          imported = true;
          module_items.push(ast::ModuleItem::ModuleDecl(ast::ModuleDecl::ExportAll(
            ast::ExportAll {
              src: src.clone(),
              span: Default::default(),
              asserts: None,
            },
          )))
        }

        if !imported {
          module_items.push(ast::ModuleItem::ModuleDecl(ast::ModuleDecl::Import(
            ast::ImportDecl {
              src,
              specifiers: vec![],
              ..ast::ImportDecl::dummy()
            },
          )))
        }

        module_items
      })
      .collect::<Vec<_>>();

    self.before_module_items = module_items;

    if self.is_user_defined_entry {
      self.validate_export_mode(ctx.output_options, &exports_in_scope)?;
    }

    if !exports_in_scope.is_empty() {
      let exports = rolldown_ast_template::build_exports_stmt(
        exports_in_scope
          .iter()
          .map(|(exported_name, spec)| (exported_name.clone(), spec.local_id.clone().to_id()))
          .collect(),
      );
      self.after_module_items.push(exports);
    }
    Ok(())
  }

  fn validate_export_mode(
    &mut self,
    output_options: &OutputOptions,
    exports: &FxHashMap<JsWord, ExportedSpecifier>,
  ) -> BuildResult<()> {
    // validate export mode
    if output_options.format.is_cjs() {
      match output_options.export_mode {
        ExportMode::Default => {
          if !exports.contains_key(&js_word!("default")) || exports.len() != 1 {
            return Err(BuildError::incompatible_export_option_value(
              "default",
              exports.keys().map(|s| s.to_string()).collect(),
              self.entry.as_ref(),
            ));
          }
        }
        ExportMode::None => {
          if !exports.is_empty() {
            return Err(BuildError::incompatible_export_option_value(
              "none",
              exports.keys().map(|s| s.to_string()).collect(),
              self.entry.as_ref(),
            ));
          }
        }
        ExportMode::Auto => {
          if exports.is_empty() {
            self.export_mode = ExportMode::None;
          } else if exports.len() == 1 && exports.contains_key(&js_word!("default")) {
            self.export_mode = ExportMode::Default;
          } else {
            if !output_options.format.is_es() && exports.contains_key(&js_word!("default")) {
              // TODO: warn about MIXED_EXPORTS
            }
            self.export_mode = ExportMode::Named;
          }
        }
        ExportMode::Named => {
          // Don't need to do anything
        }
      }
    };
    Ok(())
  }
}

#[derive(Debug)]
pub(crate) struct RenderContext {}

pub(crate) struct FinalizeBundleContext<'me> {
  pub modules: ModuleRefMutById<'me>,
  pub split_point_id_to_chunk_id: &'me SplitPointIdToChunkId,
  pub chunk_filename_by_id: &'me FxHashMap<ChunkId, String>,
  pub uf: &'me UnionFind<Symbol>,
  // pub unresolved_mark: Mark,
  pub unresolved_ctxt: SyntaxContext,
  pub output_options: &'me OutputOptions,
}
