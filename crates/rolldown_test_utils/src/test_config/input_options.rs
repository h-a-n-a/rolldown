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

  #[serde(default)]
  pub shim_missing_exports: bool,

  #[serde(default)]
  pub builtins: Builtins,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputItem {
  pub name: String,
  pub import: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Builtins {
  #[serde(default)]
  pub tsconfig: TsConfig,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TsConfig {
  #[serde(default)]
  pub use_define_for_class_fields: bool,
}

impl_serde_default!(InputOptions);
impl_serde_default!(InputItem);
impl_serde_default!(Builtins);
impl_serde_default!(TsConfig);
