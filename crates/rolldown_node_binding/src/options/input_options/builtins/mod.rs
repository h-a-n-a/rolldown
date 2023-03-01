use derivative::Derivative;
use serde::Deserialize;

mod node_resolve;
pub use node_resolve::*;

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct BuiltinsOption {
  /// None means disable the builtin
  pub node_resolve: Option<BuiltinNodeResolveOption>,
}
