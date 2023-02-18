use std::{fmt::Display, path::PathBuf, sync::Arc};

use swc_core::common::SourceFile;

use crate::error_code;

#[derive(Debug)]
pub enum ErrorKind {
  // --- Aligned with rollup
  UnresolvedEntry {
    unresolved_id: PathBuf,
  },
  ExternalEntry {
    id: PathBuf,
  },
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

  
  /// This error means that rolldown panics because unrecoverable error happens.
  /// 
  /// This error is also used to emulate plain error `throw`ed by rollup.
  /// For `throw new Error("Errored")` in js, you can use `ErrorKind::anyhow(anyhow::format_err!("Errored"))`
  /// 
  /// We also use this to replace panic!() in the code for graceful shutdown.
  /// But this is not recommended.
  Panic {
    source: anyhow::Error,
  },

  // --- Custom

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
      // Aligned with rollup
      ErrorKind::UnresolvedEntry { unresolved_id } => write!(f, "Could not resolve entry module \"{}\"", unresolved_id.display()),
      ErrorKind::ExternalEntry { id } => write!(f, "Entry module \"{}\" cannot be external.", id.display()),
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
      // Rolldown specific
      ErrorKind::Panic { source } => source.fmt(f),
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
      // Aligned with rollup
      ErrorKind::UnresolvedEntry { .. } => error_code::UNRESOLVED_ENTRY,
      ErrorKind::ExternalEntry { .. } => error_code::UNRESOLVED_ENTRY,
      ErrorKind::MissingExport { .. } => error_code::MISSING_EXPORT,
      ErrorKind::AmbiguousExternalNamespaces { .. } => error_code::AMBIGUOUS_EXTERNAL_NAMESPACES,
      ErrorKind::CircularDependency(_) => error_code::CIRCULAR_DEPENDENCY,
      // Rolldown specific
      ErrorKind::Panic { .. } => error_code::PANIC,
      ErrorKind::Napi { status, reason } => todo!(),
      ErrorKind::ReadFileFailed { filename, source } => todo!(),
      ErrorKind::ParseJsFailed {
        source_file,
        source,
      } => todo!(),
    }
  }
}
