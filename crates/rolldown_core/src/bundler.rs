use rolldown_plugin::BuildPlugin;
use sugar_path::AsPath;

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
    rolldown_tracing::init();
    Self {
      input_options: input_opts,
      plugin_driver: Default::default(),
    }
  }

  pub fn with_plugins(input_opts: InputOptions, plugins: Vec<Box<dyn BuildPlugin>>) -> Self {
    Self {
      input_options: input_opts,
      plugin_driver: BuildPluginDriver::new(plugins).into_shared(),
    }
  }

  async fn build(&mut self, output_opts: OutputOptions) -> BuildResult<Vec<Asset>> {
    tracing::debug!("InputOptions {:#?}", self.input_options);
    tracing::debug!("start bundling with OutputOptions: {:#?}", output_opts);
    let mut graph = Graph::new(self.plugin_driver.clone());
    graph.build(&self.input_options).await?;
    tracing::trace!("graph: {:#?}", graph);
    // TODO: Better warning handling
    if !graph.warnings.is_empty() {
      graph.warnings.iter().for_each(|w| {
        println!("{w}");
      });
    }
    let mut bundle = Bundle::new(&self.input_options, &output_opts, &mut graph);
    let assets = bundle.generate()?;
    Ok(assets)
  }

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
        "Could not create directory for output chunks: {:?}",
        dir.as_path()
      )
    });
    for chunk in &output {
      std::fs::write(dir.as_path().join(&chunk.filename), &chunk.content).unwrap_or_else(|_| {
        panic!(
          "Failed to write file in {:?}",
          dir.as_path().join(&chunk.filename)
        )
      });
    }
    Ok(output)
  }

  pub async fn generate(&mut self, output_options: OutputOptions) -> BuildResult<Vec<Asset>> {
    let output = self.build(output_options).await?;

    Ok(output)
  }
}
