use schemars::JsonSchema;
use serde::Deserialize;

use crate::impl_serde_default;

fn input_default() -> Vec<InputItem> {
  vec![InputItem {
    name: "main".to_string(),
    import: "./main".to_string(),
  }]
}

fn true_by_default() -> bool {
  true
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputOptions {
  #[serde(default = "input_default")]
  pub input: Vec<InputItem>,

  #[serde(default)]
  pub external: Vec<String>,

  #[serde(default = "true_by_default")]
  pub treeshake: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputItem {
  pub name: String,
  pub import: String,
}

impl_serde_default!(InputOptions);
impl_serde_default!(InputItem);
