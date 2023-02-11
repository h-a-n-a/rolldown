use std::{fmt::Display, path::Path};

use sugar_path::SugarPath;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  // --- Aligned with rollup
  UnresolvedEntry(String),
  MissingExport(String),
  AmbiguousExternalNamespaces {
    reexporting_module: String,
    used_module: String,
    binding: String,
    sources: Vec<String>,
  },

  // --- Custom
  /// Rolldown use this replace panic!() in the code.
  /// This makes Rolldown could gracefully shutdown.
  Panic(String),
  Throw(String),

  Anyhow {
    source: anyhow::Error,
  },
  Napi {
    status: String,
    reason: String,
  },
  ReadFileFailed {
    filename: String,
    source: std::io::Error,
  },
  ParseFailed(String, String),
}

impl Error {
  // --- Aligned with rollup
  pub fn entry_cannot_be_external(unresolved_id: impl AsRef<Path>, cwd: impl AsRef<Path>) -> Self {
    let unresolved_id = unresolved_id.as_ref();
    let cwd = cwd.as_ref();
    let id = if unresolved_id.is_absolute() {
      unresolved_id.relative(cwd)
    } else {
      unresolved_id.to_path_buf()
    };
    Self::UnresolvedEntry(format!(
      "Entry module \"{}\" cannot be external.",
      id.display()
    ))
  }

  pub fn ambiguous_external_namespaces(
    binding: String,
    reexporting_module: String,
    used_module: String,
    sources: Vec<String>,
  ) -> Self {
    Self::AmbiguousExternalNamespaces {
      reexporting_module,
      used_module,
      binding,
      sources,
    }
  }

  pub fn unresolved_entry(unresolved_id: impl AsRef<Path>) -> Self {
    Self::UnresolvedEntry(format!(
      "Could not resolve entry module \"{}\".",
      unresolved_id.as_ref().display()
    ))
  }

  pub fn missing_export(missing_exported_name: &str, importer: &str, importee: &str) -> Self {
    Self::MissingExport(format!(
      r#""{missing_exported_name}" is not exported by "{importee}", imported by "{importer}"."#,
    ))
  }

  // --- Custom

  pub fn throw(msg: String) -> Self {
    Self::Throw(msg)
  }
  pub fn panic(msg: &str) -> Self {
    Self::Panic(msg.to_string())
  }
}

impl std::convert::From<anyhow::Error> for Error {
  fn from(value: anyhow::Error) -> Self {
    Self::Anyhow { source: value }
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Error::Anyhow { source, .. } => Some(source.as_ref()),
      Error::ReadFileFailed { source, .. } => Some(source),
      _ => None,
    }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::UnresolvedEntry(msg) => msg.fmt(f),
      Error::MissingExport(msg) => msg.fmt(f),
      Error::AmbiguousExternalNamespaces {
        binding,
        reexporting_module,
        used_module,
        sources,
      } => write!(
        f,
        "Ambiguous external namespace resolution: {reexporting_module} re-exports {binding} from one of the external modules {sources:?}, guessing {used_module}"
      ),
      Error::Throw(msg) => write!(f, "Throw: {msg}"),
      Error::Panic(msg) => write!(f, "Panic: {msg}"),
      Error::Anyhow { source } => source.fmt(f),
      Error::Napi { status, reason } => write!(f, "Napi Error: {status} - {reason}"),
      Error::ReadFileFailed { source, filename } => {
        write!(f, "Read file failed: [{filename}] {source}")
      }
      Error::ParseFailed(reason, code) => write!(f, "Parse failed: {reason} - {code:#}"),
    }
  }
}
