use std::{fmt::Display, path::PathBuf, sync::Arc};

use swc_core::common::SourceFile;

#[derive(Debug)]
pub enum ErrorKind {
  // --- Aligned with rollup
  UnresolvedEntry(String),
  MissingExport {
    importer: PathBuf,
    importee: PathBuf,
    missing_export: String,
  },
  AmbiguousExternalNamespaces {
    reexporting_module: String,
    used_module: String,
    binding: String,
    sources: Vec<String>,
  },
  CircularDependency(Vec<String>),

  // --- Rolldown specific
  ParseJsFailed {
    source_file: Arc<SourceFile>,
    source: swc_core::ecma::parser::error::Error,
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
}

impl Display for ErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorKind::UnresolvedEntry(entry) => write!(f, "Unresolved entry: {entry}"),
      ErrorKind::MissingExport { missing_export, importee, importer } => write!(f, 
        r#""{missing_export}" is not exported by "{}", imported by "{}"."#,
        importee.display(),
        importer.display()),
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
      ErrorKind::ParseJsFailed { source_file, .. } => {
        write!(f, "Parse failed: {}", source_file.name )
      }
    }
  }
}

impl ErrorKind {
  pub fn code(&self) -> &'static str {
    match self {
      ErrorKind::UnresolvedEntry(_) => todo!(),
      ErrorKind::MissingExport { .. } => "MISSING_EXPORT",
      ErrorKind::AmbiguousExternalNamespaces { .. } => todo!(),
      ErrorKind::CircularDependency(_) => "CIRCULAR_DEPENDENCY",
      ErrorKind::Panic(_) => todo!(),
      ErrorKind::Throw(_) => todo!(),
      ErrorKind::Anyhow { source } => todo!(),
      ErrorKind::Napi { status, reason } => todo!(),
      ErrorKind::ReadFileFailed { filename, source } => todo!(),
      ErrorKind::ParseJsFailed {
        source_file,
        source,
      } => todo!(),
    }
  }
}
