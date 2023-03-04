use std::collections::HashSet;

use futures::future::join_all;
use rolldown_common::{ExportedSpecifier, ModuleId};
use rolldown_error::Errors;
use rolldown_plugin::ResolveArgs;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::common::{Mark, SyntaxContext, GLOBALS};

pub(crate) mod module_task;

use module_task::{ModuleTask, TaskResult};
use swc_core::ecma::atoms::js_word;
use tracing::instrument;

use crate::{norm_or_ext::NormOrExt, BuildInputOptions, Graph, NormalModule, SWC_GLOBALS};
use crate::{
  resolve_id, BuildError, BuildResult, ExternalModule, SharedBuildInputOptions,
  SharedBuildPluginDriver, SharedResolver, StatementParts, UnaryBuildResult,
};

pub(crate) struct ModuleLoader<'a> {
  input_options: SharedBuildInputOptions,
  graph: &'a mut Graph,
  build_plugin_driver: SharedBuildPluginDriver,
  loaded_modules: HashSet<ModuleId>,
  remaining_tasks: usize,
  tx: tokio::sync::mpsc::UnboundedSender<Msg>,
  rx: tokio::sync::mpsc::UnboundedReceiver<Msg>,
  resolver: SharedResolver,
  errors: Vec<BuildError>,
  dynamic_imported_modules: FxHashSet<ModuleId>,
}

#[derive(Debug)]
pub(crate) enum Msg {
  Scanned(TaskResult),
  Error(BuildError),
}

impl<'a> ModuleLoader<'a> {
  pub(crate) fn new(
    graph: &'a mut Graph,
    resolver: SharedResolver,
    plugin_driver: SharedBuildPluginDriver,
    input_options: SharedBuildInputOptions,
  ) -> Self {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();
    Self {
      graph,
      loaded_modules: Default::default(),
      remaining_tasks: 0,
      tx,
      rx,
      resolver,
      errors: Default::default(),
      build_plugin_driver: plugin_driver,
      dynamic_imported_modules: Default::default(),
      input_options,
    }
  }

  #[instrument(skip_all)]
  async fn resolve_entries(
    &self,
    input_opts: &BuildInputOptions,
  ) -> Vec<UnaryBuildResult<ModuleId>> {
    join_all(input_opts.input.iter().cloned().map(|input_item| async {
      let id = resolve_id(
        &self.resolver,
        ResolveArgs {
          importer: None,
          specifier: &input_item.import,
        },
        &self.build_plugin_driver,
      )
      .await?;

      let Some(id) = id else {
          return Err(BuildError::unresolved_entry(input_item.import))
        };

      if id.is_external() {
        return Err(BuildError::entry_cannot_be_external(id.as_ref()));
      }
      UnaryBuildResult::Ok(id)
    }))
    .await
  }

  #[instrument(skip_all)]
  pub(crate) async fn fetch_all_modules(mut self) -> BuildResult<()> {
    if self.input_options.input.is_empty() {
      return Err(
        BuildError::panic("You must supply options.input to rolldown".to_string()).into(),
      );
    }

    let resolved_entries = self.resolve_entries(&self.input_options).await;

    resolved_entries
      .into_iter()
      .try_for_each(|entry| -> UnaryBuildResult<()> {
        let id = entry?;
        if id.is_external() {
          return Err(BuildError::entry_cannot_be_external(id.as_ref()));
        }
        self.loaded_modules.insert(id.clone());
        self.graph.entries.push(id.clone());
        self.spawn_new_module_task(id, true);
        Ok(())
      })?;

    while self.remaining_tasks > 0 {
      let msg = self.rx.recv().await.unwrap();
      match msg {
        Msg::Scanned(res) => {
          self.remaining_tasks -= 1;
          self.handle_msg_scanned(res);
        }
        Msg::Error(err) => {
          self.remaining_tasks -= 1;
          self.errors.push(err);
        }
      }
      tracing::trace!("remaining: {}", self.remaining_tasks);
    }

    self.mark_dynamic_imported_module();

    if self.errors.is_empty() {
      Ok(())
    } else {
      Err(Errors::from_vec(std::mem::take(&mut self.errors)))
    }
  }

  fn mark_dynamic_imported_module(&mut self) {
    self.dynamic_imported_modules.iter().for_each(|id| {
      let module =
        self.graph.module_by_id.get_mut(id).unwrap_or_else(|| {
          unreachable!("dynamic imported module should be in the graph: {}", id)
        });
      if let NormOrExt::Normal(module) = module {
        module.is_dynamic_entry = true;
      }
    });
  }

  fn spawn_new_module_task(&mut self, module_id: ModuleId, is_user_defined_entry: bool) {
    tracing::trace!("spawning new job for {}", module_id);
    self.remaining_tasks += 1;
    let (top_level_mark, top_level_ctxt) = GLOBALS.set(&SWC_GLOBALS, || {
      let mark = Mark::new();
      (mark, SyntaxContext::empty().apply_mark(mark))
    });
    let task = ModuleTask {
      id: module_id,
      tx: self.tx.clone(),
      top_level_mark,
      unresolved_mark: self.graph.unresolved_mark,
      top_level_ctxt,
      unresolved_ctxt: self.graph.unresolved_ctxt,
      is_user_defined_entry,
      resolver: self.resolver.clone(),
      plugin_driver: self.build_plugin_driver.clone(),
      is_external: self.input_options.is_external.clone(),
      input_options: self.input_options.clone(),
    };
    tokio::spawn(task.run());
  }

  #[instrument(skip_all)]
  fn handle_msg_scanned(&mut self, result: TaskResult) {
    let module_id = result.module_id;
    let scan_result = result.scan_result;
    let resolved_ids = result.resolved_ids;

    resolved_ids.values().for_each(|id| {
      if self.loaded_modules.contains(id) {
        return;
      }
      self.loaded_modules.insert(id.clone());
      let top_level_ctxt = GLOBALS.set(&SWC_GLOBALS, || {
        SyntaxContext::empty().apply_mark(Mark::new())
      });
      if id.is_external() {
        let external_module = ExternalModule {
          exec_order: usize::MAX,
          id: id.clone(),
          top_level_ctxt,
          runtime_helpers: Default::default(),
          exports: Default::default(),
        };
        self.graph.add_module(NormOrExt::External(external_module));
      } else {
        self.spawn_new_module_task(id.clone(), false);
      }
    });

    let dependencies = scan_result
      .dependencies
      .iter()
      .map(|id| resolved_ids[id].clone())
      .collect();

    let dyn_dependencies: Vec<ModuleId> = scan_result
      .dyn_dependencies
      .iter()
      .map(|id| resolved_ids[id].clone())
      .collect();
    self
      .dynamic_imported_modules
      .extend(dyn_dependencies.clone());

    let re_export_all = scan_result
      .re_export_all
      .iter()
      .map(|id| resolved_ids[id].clone())
      .collect();

    let re_exported_ids = scan_result
      .re_exported_ids
      .into_iter()
      .map(|(id, re_exported_ids)| {
        let id = resolved_ids[&id].clone();
        (id, re_exported_ids)
      })
      .collect();

    let imports = scan_result
      .imports
      .into_iter()
      .map(|(id, imported_ids)| {
        let id = resolved_ids[&id].clone();
        (id, imported_ids)
      })
      .collect::<FxHashMap<_, _>>();

    let top_level_ctxt = result.top_level_ctxt;

    let normal_module = NormalModule {
      dependencies,
      dyn_dependencies,
      exec_order: usize::MAX,
      top_level_ctxt,
      ast: result.ast,
      is_user_defined_entry: result.is_user_defined_entry,
      suggested_names: scan_result.suggested_names,
      facade_id_for_namespace: ExportedSpecifier {
        exported_as: js_word!("*"),
        local_id: ("*".to_string().into(), top_level_ctxt).into(),
        owner: module_id.clone(),
      },
      extra_top_level_symbols: Default::default(),
      is_facade_namespace_id_referenced: false,
      visited_global_names: scan_result.visited_global_names,
      external_modules_of_re_export_all: Default::default(),
      is_dynamic_entry: false,
      comments: result.comments,
      imports,
      linked_imports: Default::default(),
      local_exports: scan_result.local_exports.clone(),
      linked_exports: scan_result.local_exports,
      re_exported_ids,
      re_export_all,
      resolved_module_ids: resolved_ids,
      declared_scoped_names: scan_result.declared_scoped_names,
      id: module_id,
      runtime_helpers: Default::default(),
      parts: StatementParts::from_parts(scan_result.statement_parts),
      missing_exports: Default::default(),
    };
    self.graph.add_module(NormOrExt::Normal(normal_module));
  }
}
