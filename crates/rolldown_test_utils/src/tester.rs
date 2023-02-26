use std::{
  collections::HashSet,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use futures::FutureExt;
use rolldown_core::BuildError;

use crate::test_config::TestConfig;

pub struct Tester {
  pub config: TestConfig,
  pub warnings: Arc<Mutex<Vec<BuildError>>>,
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
    let warning_collector = self.warnings.clone();
    rolldown_core::InputOptions {
      // TODO: the order should be preserved
      input: self
        .config
        .input
        .input
        .iter()
        .map(|item| rolldown_core::InputItem {
          name: item.name.clone(),
          import: item.import.clone(),
        })
        .collect(),
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
      on_warn: Arc::new(move |err| {
        warning_collector.lock().unwrap().push(err);
      }),
      ..Default::default()
    }
  }
}
