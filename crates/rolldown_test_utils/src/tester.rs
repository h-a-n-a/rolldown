use std::{
  collections::HashSet,
  path::{Path, PathBuf},
  sync::Arc,
};

use futures::FutureExt;
use rolldown_core::BuildError;

use crate::test_config::TestConfig;

pub struct Tester {
  pub config: TestConfig,
  pub warnings: Vec<BuildError>,
}

impl Tester {
  pub fn from_config_path(filepath: &Path) -> Self {
    let test_config = TestConfig::from_config_path(filepath);
    Self {
      config: test_config,
      warnings: Default::default(),
    }
  }

  pub fn input_options(&self, cwd: PathBuf) -> rolldown_core::InputOptions {
    rolldown_core::InputOptions {
      input: self.config.input.input.clone(),
      cwd,
      treeshake: self.config.input.treeshake,
      is_external: {
        let external = self
          .config
          .input
          .external
          .clone()
          .into_iter()
          .collect::<HashSet<_>>();
        Arc::new(move |specifier, _importer, _| {
          let external = external.clone();
          futures::future::ready(Ok(external.contains(specifier))).boxed()
        })
      },
      ..Default::default()
    }
  }
}
