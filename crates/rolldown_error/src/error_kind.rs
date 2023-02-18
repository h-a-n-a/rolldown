use std::{fmt::Display, sync::Arc};

use swc_core::common::SourceFile;

#[derive(Debug)]
pub enum ErrorKind {
  // --- Aligned with rollup
  UnresolvedEntry(String),
  MissingExport(String),
  AmbiguousExternalNamespaces {
    reexporting_module: String,
    used_module: String,
    binding: String,
    sources: Vec<String>,
  },
  CircularDependency(Vec<String>),

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
  ParseFailedOld(String, String),
  ParseJsFailed {
    source_file: Arc<SourceFile>,
    source: swc_core::ecma::parser::error::Error,
  },
}

impl Display for ErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorKind::UnresolvedEntry(entry) => write!(f, "Unresolved entry: {entry}"),
      ErrorKind::MissingExport(name) => write!(f, "Missing export: {name}"),
      ErrorKind::AmbiguousExternalNamespaces {
        binding,
        reexporting_module,
        used_module,
        sources,
      } => write!(
        f,
        "Ambiguous external namespace resolution: {reexporting_module} re-exports {binding} from one of the external modules {sources:?}, guessing {used_module}"
      ),
      ErrorKind::CircularDependency(path) => write!(f, "Circular dependency: {}", path.join(" -> ")),
      ErrorKind::Panic(msg) => write!(f, "panic!: {msg}"),
      ErrorKind::Throw(msg) => write!(f, "Throw {msg}"),
      ErrorKind::Anyhow { source } => write!(f, "Rolldown error: {source}"),
      ErrorKind::Napi { status, reason } => write!(f, "Napi error: {} {}", status, reason),
      ErrorKind::ReadFileFailed { filename, source } => {
        write!(f, "Read file failed: {} {}", filename, source)
      }
      ErrorKind::ParseFailedOld(code, msg) => write!(f, "Parse failed: {} {}", code, msg),
      ErrorKind::ParseJsFailed { source_file, .. } => {
        write!(f, "Parse failed: {}", source_file.name )
      }
    }
  }
}
