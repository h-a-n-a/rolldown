use std::{path::PathBuf, sync::Arc};

use derivative::Derivative;
use futures::{future, FutureExt};
pub use rolldown_core::{InputItem, IsExternal, WarningHandler};
mod builtins;
pub use builtins::*;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct InputOptions {
  pub input: Vec<InputItem>,
  pub preserve_symlinks: bool,
  pub treeshake: bool,
  pub cwd: PathBuf,
  #[derivative(Debug = "ignore")]
  pub is_external: IsExternal,
  #[derivative(Debug = "ignore")]
  pub on_warn: WarningHandler,
  pub shim_missing_exports: bool,
  pub builtins: BuiltinsOptions,
}

pub fn default_warning_handler() -> WarningHandler {
  Arc::new(|err| {
    eprintln!("{}", err);
  })
}

impl Default for InputOptions {
  fn default() -> Self {
    Self {
      input: Default::default(),
      preserve_symlinks: true,
      treeshake: true,
      cwd: std::env::current_dir().unwrap(),
      is_external: Arc::new(|_, _, _| future::ready(Ok(false)).boxed()),
      on_warn: default_warning_handler(),
      shim_missing_exports: false,
      builtins: Default::default(),
    }
  }
}
