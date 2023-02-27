mod error;
use std::path::PathBuf;

pub use error::*;
mod error_kind;
pub use anyhow;
pub use anyhow::format_err;
pub use error_kind::*;
mod utils;

mod errors;

pub type Result<T> = std::result::Result<T, Error>;
pub type ResultWithErrors<T> = std::result::Result<T, Errors>;
pub use errors::Errors;

scoped_tls::scoped_thread_local!(
  /// Current working directory.
  pub static CWD: PathBuf
);
