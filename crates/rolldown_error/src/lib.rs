mod error;
pub use error::*;
mod error_kind;
pub use anyhow;
pub use anyhow::format_err;
pub use error_kind::*;
mod utils;

pub type Result<T> = std::result::Result<T, Error>;
