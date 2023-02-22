use std::{fmt::Display, path::PathBuf, sync::Arc};

use swc_core::common::SourceFile;

use crate::utils::format_quoted_strings;

pub mod error_code;

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
  InvalidExportOptionValue(String),
  IncompatibleExportOptionValue {
    option_value: &'static str,
    exported_keys: Vec<String>,
    entry_module: PathBuf,
  },

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
  /// We also use this to replace `panic!()` in the code for graceful shutdown.
  /// But this is not recommended.
  Panic {
    source: anyhow::Error,
  },

  // --- Custom
  Napi {
    status: String,
    reason: String,
  },
}

impl Display for ErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      // Aligned with rollup
      ErrorKind::UnresolvedEntry { unresolved_id } => write!(f, "Could not resolve entry module \"{}\"", unresolved_id.display()),
      ErrorKind::ExternalEntry { id } => write!(f, "Entry module \"{}\" cannot be external.", id.display()),
      ErrorKind::MissingExport { missing_export, importee, importer } => write!(
        f,
        r#""{missing_export}" is not exported by "{}", imported by "{}"."#,
        importee.display(),
        importer.display(),
      ),
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
      ErrorKind::InvalidExportOptionValue(value) =>  write!(f, r#""output.exports" must be "default", "named", "none", "auto", or left unspecified (defaults to "auto"), received "{value}"."#),
      ErrorKind::IncompatibleExportOptionValue { option_value, exported_keys, entry_module } => {
        let mut exported_keys = exported_keys.iter().collect::<Vec<_>>();
        exported_keys.sort();
        write!(f, r#""{option_value}" was specified for "output.exports", but entry module "{}" has the following exports: {}"#, entry_module.display(), format_quoted_strings(&exported_keys))
      }
      // Rolldown specific
      ErrorKind::Panic { source } => source.fmt(f),
      ErrorKind::Napi { status, reason } => write!(f, "Napi error: {} {}", status, reason),
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
      ErrorKind::InvalidExportOptionValue(_) => error_code::INVALID_EXPORT_OPTION,
      ErrorKind::IncompatibleExportOptionValue { .. } => error_code::INVALID_EXPORT_OPTION,
      // Rolldown specific
      ErrorKind::Panic { .. } => error_code::PANIC,
      ErrorKind::Napi {
        status: _,
        reason: _,
      } => todo!(),
      ErrorKind::ParseJsFailed {
        source_file: _,
        source: _,
      } => todo!(),
    }
  }
}
