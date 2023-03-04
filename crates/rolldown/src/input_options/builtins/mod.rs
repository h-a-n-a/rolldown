mod node_resolve;
use derivative::Derivative;
pub use node_resolve::*;
pub use rolldown_core::TsConfig;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct BuiltinsOptions {
  /// None means disable the builtin
  pub node_resolve: Option<NodeResolveOptions>,
  /// None means default
  pub tsconfig: Option<TsConfig>,
}

impl Default for BuiltinsOptions {
  fn default() -> Self {
    Self {
      node_resolve: Some(Default::default()),
      tsconfig: Some(Default::default()),
    }
  }
}
