mod typescript;
use derivative::Derivative;
pub use typescript::*;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct BuiltinsOptions {
  /// None means disable the builtin
  pub tsconfig: TsConfig,
  // TODO: Should come up with a better name before exposing this option.
  pub detect_loader_by_ext: bool,
}

impl Default for BuiltinsOptions {
  fn default() -> Self {
    Self {
      tsconfig: Default::default(),
      detect_loader_by_ext: true,
    }
  }
}
