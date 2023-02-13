mod error;
pub use error::*;
mod error_kind;
pub use error_kind::*;

pub type Result<T> = std::result::Result<T, Error>;
