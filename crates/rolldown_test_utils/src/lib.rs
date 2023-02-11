use std::{
  collections::{HashMap, HashSet},
  path::{Path, PathBuf},
  sync::Arc,
};

use futures::FutureExt;
use rolldown_core::InputOptions;
use serde::{Deserialize, Serialize};

fn true_by_default() -> bool {
  true
}

fn esm_by_default() -> String {
  "esm".to_string()
}

fn input_default() -> HashMap<String, String> {
  HashMap::from([("main".to_string(), "./main.js".to_string())])
}
#[derive(Serialize, Deserialize)]
pub struct TestConfig {
  #[serde(default = "input_default")]
  pub input: HashMap<String, String>,
  #[serde(default)]
  pub external: Vec<String>,
  #[serde(default = "true_by_default")]
  pub treeshake: bool,
  #[serde(default = "esm_by_default")]
  pub format: String,
}

impl TestConfig {
  pub fn from_config_path(filepath: &Path) -> Self {
    let test_config: TestConfig =
      serde_json::from_str(&std::fs::read_to_string(filepath).unwrap_or_else(|_| "{}".to_string()))
        .unwrap();
    test_config
  }

  pub fn input_options(&self, cwd: String) -> InputOptions {
    let test_config = self;
    InputOptions {
      input: test_config.input.clone().into_iter().collect(),
      cwd: PathBuf::from(cwd),
      treeshake: self.treeshake,
      is_external: {
        let external = test_config
          .external
          .clone()
          .into_iter()
          .collect::<HashSet<_>>();
        Arc::new(move |specifier, _importer, _| {
          let external = external.clone();
          futures::future::ready(Ok(external.contains(specifier))).boxed()
        })
      },
    }
  }
}
