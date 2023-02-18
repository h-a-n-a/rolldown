use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::impl_serde_default;

fn input_default() -> HashMap<String, String> {
  HashMap::from([("main".to_string(), "./main.js".to_string())])
}

fn true_by_default() -> bool {
  true
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputOptions {
  #[serde(default = "input_default")]
  pub input: HashMap<String, String>,

  #[serde(default)]
  pub external: Vec<String>,

  #[serde(default = "true_by_default")]
  pub treeshake: bool,
}

impl_serde_default!(InputOptions);
