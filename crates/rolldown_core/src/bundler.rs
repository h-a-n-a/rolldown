use rolldown_plugin::BuildPlugin;
use sugar_path::AsPath;
use tracing::instrument;

use crate::{
  BuildPluginDriver, BuildResult, Bundle, Graph, InputOptions, OutputOptions,
  SharedBuildPluginDriver,
};

pub struct Bundler {
  input_options: InputOptions,
  plugin_driver: SharedBuildPluginDriver,
}

#[derive(Debug)]
pub struct Asset {
  pub filename: String,
  pub content: String,
}

impl Bundler {
  pub fn new(input_opts: InputOptions) -> Self {
    Self::with_plugins(input_opts, vec![])
  }

  pub fn with_plugins(input_opts: InputOptions, plugins: Vec<Box<dyn BuildPlugin>>) -> Self {
    rolldown_tracing::enable_tracing_on_demand();
    Self {
      input_options: input_opts,
      plugin_driver: BuildPluginDriver::new(plugins).into_shared(),
    }
  }

  #[instrument(skip_all)]
  async fn build(&mut self, output_opts: OutputOptions) -> BuildResult<Vec<Asset>> {
    tracing::debug!("{:#?}", self.input_options);
    tracing::debug!("{:#?}", output_opts);
    let mut graph = Graph::new(
      self.plugin_driver.clone(),
      self.input_options.on_warn.clone(),
    );
    graph.generate_module_graph(&self.input_options).await?;
    let mut bundle = Bundle::new(&self.input_options, &output_opts, &mut graph);
    let assets = bundle.generate()?;
    Ok(assets)
  }

  #[instrument(skip_all)]
  pub async fn write(&mut self, output_options: OutputOptions) -> BuildResult<Vec<Asset>> {
    let dir = output_options.dir.clone().unwrap_or_else(|| {
      self
        .input_options
        .cwd
        .as_path()
        .join("dist")
        .to_string_lossy()
        .to_string()
    });
    let output = self.build(output_options).await?;

    std::fs::create_dir_all(&dir).unwrap_or_else(|_| {
      panic!(
        "Could not create directory for output chunks: {:?} \ncwd: {}",
        dir.as_path(),
        self.input_options.cwd.display()
      )
    });
    for chunk in &output {
      let dest = dir.as_path().join(&chunk.filename);
      if let Some(p) = dest.parent() {
        if !p.exists() {
          std::fs::create_dir_all(p)?;
        }
      };
      std::fs::write(dest, &chunk.content).unwrap_or_else(|_| {
        panic!(
          "Failed to write file in {:?}",
          dir.as_path().join(&chunk.filename)
        )
      });
    }
    Ok(output)
  }

  #[instrument(skip_all)]
  pub async fn generate(&mut self, output_options: OutputOptions) -> BuildResult<Vec<Asset>> {
    let output = self.build(output_options).await?;

    Ok(output)
  }
}
