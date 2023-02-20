use std::{sync::Arc};

use rolldown_common::ModuleId;
use rolldown_plugin::{
  BuildPlugin, Context, ResolveArgs, ResolveOutput, TransformArgs,
};
use tokio::sync::RwLock;

use crate::BundleResult;

pub(crate) type SharedBuildPluginDriver = Arc<RwLock<BuildPluginDriver>>;

#[derive(Debug, Default)]
pub(crate) struct BuildPluginDriver {
  pub plugins: Vec<Box<dyn BuildPlugin>>,
}

impl BuildPluginDriver {
  pub(crate) fn new(plugins: Vec<Box<dyn BuildPlugin>>) -> Self {
    Self { plugins }
  }

  pub(crate) fn into_shared(self) -> SharedBuildPluginDriver {
    Arc::new(RwLock::new(self))
  }

  pub(crate) async fn resolve(&self, mut args: ResolveArgs<'_>) -> ResolveOutput {
    for plugin in &self.plugins {
      let output = plugin.resolve(&mut Context::new(), &mut args).await?;
      if output.is_some() {
        return Ok(output);
      }
    }
    Ok(None)
  }

  pub(crate) async fn transform(&self, id: &ModuleId, code: String) -> BundleResult<String> {
    let mut code = code;
    for plugin in &self.plugins {
      let output = plugin
        .transform(&mut Context::new(), &mut TransformArgs { id, code: &code })
        .await?;
      if let Some(output) = output {
        code = output
      }
    }
    Ok(code)
  }
}
