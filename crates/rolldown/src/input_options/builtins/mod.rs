mod node_resolve;
use derivative::Derivative;
pub use node_resolve::*;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct BuiltinsOptions {
  /// None means disable the builtin
  pub node_resolve: Option<NodeResolveOptions>,
}
