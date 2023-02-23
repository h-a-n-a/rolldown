use std::{collections::HashMap, path::PathBuf, pin::Pin, sync::Arc};

use derivative::Derivative;
use futures::{future, Future, FutureExt};

use crate::{BuildError, BuildResult};

type PinFutureBox<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub type IsExternal =
  Arc<dyn Fn(&str, Option<&str>, bool) -> PinFutureBox<BuildResult<bool>> + Send + Sync>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct InputOptions {
  pub input: HashMap<String, String>,
  pub treeshake: bool,
  pub cwd: PathBuf,
  #[derivative(Debug = "ignore")]
  pub is_external: IsExternal,
  #[derivative(Debug = "ignore")]
  pub on_warn: Arc<dyn Fn(BuildError) + Send + Sync>,
}

impl Default for InputOptions {
  fn default() -> Self {
    Self {
      input: Default::default(),
      treeshake: true,
      cwd: std::env::current_dir().unwrap(),
      is_external: Arc::new(|_, _, _| future::ready(Ok(false)).boxed()),
      on_warn: Arc::new(|err| {
        eprintln!("{}", err);
      }),
    }
  }
}
