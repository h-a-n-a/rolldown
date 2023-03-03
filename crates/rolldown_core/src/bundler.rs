use std::sync::Arc;

use rolldown_plugin::BuildPlugin;
use tracing::instrument;

use crate::{
  BuildInputOptions, BuildOutputOptions, BuildPluginDriver, BuildResult, Bundle, Graph,
  SharedBuildInputOptions, SharedBuildPluginDriver,
};

pub struct BundlerCore {
  input_options: SharedBuildInputOptions,
  plugin_driver: SharedBuildPluginDriver,
}

#[derive(Debug)]
pub struct Asset {
  pub filename: String,
  pub content: String,
}

impl BundlerCore {
  pub fn new(input_opts: BuildInputOptions) -> Self {
    Self::with_plugins(input_opts, vec![])
  }

  pub fn with_plugins(input_opts: BuildInputOptions, plugins: Vec<Box<dyn BuildPlugin>>) -> Self {
    rolldown_tracing::enable_tracing_on_demand();
    Self {
      input_options: Arc::new(input_opts),
      plugin_driver: BuildPluginDriver::new(plugins).into_shared(),
    }
  }

  #[instrument(skip_all)]
  pub async fn build(&mut self, output_opts: BuildOutputOptions) -> BuildResult<Vec<Asset>> {
    tracing::debug!("{:#?}", self.input_options);
    tracing::debug!("{:#?}", output_opts);
    let mut graph = Graph::new(self.plugin_driver.clone(), self.input_options.clone());
    graph.generate_module_graph(&self.input_options).await?;
    let mut bundle = Bundle::new(&self.input_options, &output_opts, &mut graph);
    let assets = bundle.generate()?;
    Ok(assets)
  }
}
