use derivative::Derivative;
use futures::future::join_all;
use rolldown_common::ModuleId;
use rolldown_plugin::ResolveArgs;
use rolldown_resolver::Resolver;
use rolldown_swc_visitors::ScanResult;
use swc_core::common::{Mark, SyntaxContext, GLOBALS};
use swc_core::ecma::ast;
use swc_node_comments::SwcComments;

use super::Msg;
use crate::{
  resolve_id, BuildError, IsExternal, ResolvedModuleIds, SharedBuildPluginDriver, SharedResolver,
  UnaryBuildResult, COMPILER, SWC_GLOBALS,
};

pub(crate) struct ModuleTask {
  pub(crate) id: ModuleId,
  pub(crate) is_user_defined_entry: bool,
  pub(crate) tx: tokio::sync::mpsc::UnboundedSender<Msg>,
  pub(crate) top_level_mark: Mark,
  pub(crate) top_level_ctxt: SyntaxContext,
  pub(crate) unresolved_mark: Mark,
  pub(crate) unresolved_ctxt: SyntaxContext,
  pub(crate) resolver: SharedResolver,
  pub(crate) plugin_driver: SharedBuildPluginDriver,
  pub(crate) is_external: IsExternal,
}

impl ModuleTask {
  pub(crate) async fn resolve_id(
    resolver: &Resolver,
    importer: &ModuleId,
    specifier: &str,
    plugin_driver: &SharedBuildPluginDriver,
    is_external: &IsExternal,
  ) -> UnaryBuildResult<ModuleId> {
    let inner_ret = {
      let is_external = is_external(specifier, Some(importer.id()), false).await?;
      if is_external {
        None
      } else {
        resolve_id(
          resolver,
          ResolveArgs {
            importer: Some(importer),
            specifier,
          },
          plugin_driver,
        )
        .await?
      }
    };

    // getNormalizedResolvedIdWithoutDefaults
    if let Some(id) = inner_ret {
      let external = id.is_external() || is_external(id.id(), Some(importer.id()), true).await?;
      Ok(ModuleId::new(id.id().clone(), external))
    } else {
      // TODO: Align with rollup
      Ok(ModuleId::new(specifier, true))
    }
  }

  pub(crate) async fn run(self) {
    let tx = self.tx.clone();
    match self.run_inner().await {
      Ok(result) => {
        tx.send(Msg::Scanned(result)).unwrap();
      }
      Err(err) => {
        tx.send(Msg::Error(err)).unwrap();
      }
    }
  }

  async fn run_inner(self) -> UnaryBuildResult<TaskResult> {
    // load hook
    let code = tokio::fs::read_to_string(self.id.as_ref())
      .await
      .map_err(rolldown_error::anyhow::Error::from)
      .map_err(|e| e.context(format!("Read file: {}", self.id.as_ref())))?;

    let code = self
      .plugin_driver
      .read()
      .await
      .transform(&self.id, code)
      .await?;

    let comments = SwcComments::default();
    let (fm, ast) = COMPILER.parse_with_comments(code, self.id.as_ref(), Some(&comments));
    let mut ast = ast.map_err(|e| BuildError::parse_js_failed(fm, e))?;

    GLOBALS.set(&SWC_GLOBALS, || {
      rolldown_swc_visitors::resolve(&mut ast, self.unresolved_mark, self.top_level_mark);
    });

    let result = rolldown_swc_visitors::scan(
      &mut ast,
      self.top_level_ctxt,
      self.unresolved_ctxt,
      self.id.clone(),
    );

    let resolved_ids = join_all(
      result
        .dependencies
        .iter()
        .chain(result.dyn_dependencies.iter())
        // .cloned()
        .map(|specifier| {
          let resolver = self.resolver.clone();
          let plugin_driver = self.plugin_driver.clone();
          let importer = self.id.clone();
          let is_external = self.is_external.clone();
          async move {
            Self::resolve_id(
              &resolver,
              &importer,
              specifier,
              &plugin_driver,
              &is_external,
            )
            .await
            .map(|id| (specifier.clone(), id))
          }
        }),
    )
    .await;

    let resolved_ids = resolved_ids.into_iter().try_collect()?;

    Ok(TaskResult {
      module_id: self.id,
      ast,
      top_level_mark: self.top_level_mark,
      top_level_ctxt: self.top_level_ctxt,
      scan_result: result,
      resolved_ids,
      comments,
      is_user_defined_entry: self.is_user_defined_entry,
    })
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct TaskResult {
  pub module_id: ModuleId,
  pub ast: ast::Module,
  pub top_level_mark: Mark,
  pub top_level_ctxt: SyntaxContext,
  pub scan_result: ScanResult,
  pub resolved_ids: ResolvedModuleIds,
  #[derivative(Debug = "ignore")]
  pub comments: SwcComments,
  pub is_user_defined_entry: bool,
}
