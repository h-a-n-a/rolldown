use derivative::Derivative;
use serde::Deserialize;

mod node_resolve;
pub use node_resolve::*;
mod tsconfig;
pub use tsconfig::*;

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct BuiltinsOptions {
  /// None means disable the behaviors
  pub node_resolve: Option<NodeResolveOptions>,
  pub tsconfig: Option<TsConfigOptions>,
}
