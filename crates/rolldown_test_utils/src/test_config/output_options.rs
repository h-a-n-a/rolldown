use schemars::JsonSchema;
use serde::Deserialize;

use crate::impl_serde_default;

fn esm_by_default() -> String {
  "esm".to_string()
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputOptions {
  #[serde(default = "esm_by_default")]
  pub format: String,
}

impl_serde_default!(OutputOptions);
