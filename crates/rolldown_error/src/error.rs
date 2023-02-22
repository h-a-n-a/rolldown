use std::{fmt::Display, path::Path, sync::Arc};

use rolldown_common::CWD;
use sugar_path::SugarPath;
use swc_core::common::SourceFile;

use crate::ErrorKind;

#[derive(Debug)]
pub struct Error {
  contexts: Vec<String>,
  pub kind: ErrorKind,
}

impl Error {
  fn with_kind(kind: ErrorKind) -> Self {
    Self {
      contexts: vec![],
      kind,
    }
  }

  pub fn context(mut self, context: String) -> Self {
    self.contexts.push(context);
    self
  }

  // --- Aligned with rollup
  pub fn entry_cannot_be_external(unresolved_id: impl AsRef<Path>) -> Self {
    CWD.with(|cwd| {
      let unresolved_id = unresolved_id.as_ref();
      let id = if unresolved_id.is_absolute() {
        unresolved_id.relative(cwd)
      } else {
        unresolved_id.to_path_buf()
      };
      Self::with_kind(ErrorKind::ExternalEntry { id })
    })
  }

  pub fn ambiguous_external_namespaces(
    binding: String,
    reexporting_module: String,
    used_module: String,
    sources: Vec<String>,
  ) -> Self {
    Self::with_kind(ErrorKind::AmbiguousExternalNamespaces {
      reexporting_module,
      used_module,
      binding,
      sources,
    })
  }

  pub fn unresolved_entry(unresolved_id: impl AsRef<Path>) -> Self {
    Self::with_kind(ErrorKind::UnresolvedEntry {
      unresolved_id: unresolved_id.as_ref().to_path_buf(),
    })
  }

  pub fn missing_export(
    missing_export: &str,
    importer: impl AsRef<Path>,
    importee: impl AsRef<Path>,
  ) -> Self {
    CWD.with(|cwd| {
      let importer = importer.as_ref().relative(cwd);
      let importee = importee.as_ref().relative(cwd);
      Self::with_kind(ErrorKind::MissingExport {
        importer,
        importee,
        missing_export: missing_export.to_string(),
      })
    })
  }

  pub fn circular_dependency(circular_path: Vec<String>) -> Self {
    Self::with_kind(ErrorKind::CircularDependency(circular_path))
  }

  pub fn invalid_export_option_value(value: String) -> Self {
    Self::with_kind(ErrorKind::InvalidExportOptionValue(value))
  }

  pub fn incompatible_export_option_value(
    option_value: &'static str,
    exported_keys: Vec<String>,
    entry_module: impl AsRef<Path>,
  ) -> Self {
    CWD.with(|cwd| {
      let entry_module = entry_module.as_ref().relative(cwd);
      Self::with_kind(ErrorKind::IncompatibleExportOptionValue {
        option_value,
        exported_keys,
        entry_module,
      })
    })
  }

  // --- rolldown special

  pub fn parse_js_failed(
    fm: Arc<SourceFile>,
    source: swc_core::ecma::parser::error::Error,
  ) -> Self {
    Self::with_kind(ErrorKind::ParseJsFailed {
      source_file: fm,
      source,
    })
  }

  // --- TODO: we should remove following errors

  pub fn napi_error(status: String, reason: String) -> Self {
    Self::with_kind(ErrorKind::Napi { status, reason })
  }

  pub fn panic(msg: String) -> Self {
    anyhow::format_err!(msg).into()
  }
}

impl std::convert::From<anyhow::Error> for Error {
  fn from(value: anyhow::Error) -> Self {
    Self::with_kind(ErrorKind::Panic { source: value })
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match &self.kind {
      ErrorKind::Panic { source, .. } => Some(source.as_ref()),
      _ => None,
    }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for ctx in self.contexts.iter().rev() {
      writeln!(f, "{}: {}", ansi_term::Color::Yellow.paint("context"), ctx)?;
    }

    self.kind.fmt(f)
  }
}
