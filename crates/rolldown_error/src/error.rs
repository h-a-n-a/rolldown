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
      Self::with_kind(ErrorKind::UnresolvedEntry(format!(
        "Entry module \"{}\" cannot be external.",
        id.display()
      )))
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

  // align with rollup

  pub fn unresolved_entry(unresolved_id: impl AsRef<Path>) -> Self {
    Self::with_kind(ErrorKind::UnresolvedEntry(format!(
      "Could not resolve entry module \"{}\".",
      unresolved_id.as_ref().display()
    )))
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
      // Self::with_kind(ErrorKind::MissingExport(format!(
      //   r#""{missing_exported_name}" is not exported by "{}", imported by "{}"."#,
      //   importee.display(),
      //   importer.display()
      // )))
    })
  }

  pub fn circular_dependency(circular_path: Vec<String>) -> Self {
    Self::with_kind(ErrorKind::CircularDependency(circular_path))
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

  pub fn read_file_failed(filename: String, source: std::io::Error) -> Self {
    Self::with_kind(ErrorKind::ReadFileFailed { filename, source })
  }

  pub fn throw(msg: String) -> Self {
    Self::with_kind(ErrorKind::Throw(msg))
  }
  pub fn panic(msg: &str) -> Self {
    Self::with_kind(ErrorKind::Panic(msg.to_string()))
  }
}

impl std::convert::From<anyhow::Error> for Error {
  fn from(value: anyhow::Error) -> Self {
    Self::with_kind(ErrorKind::Anyhow { source: value })
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match &self.kind {
      ErrorKind::Anyhow { source, .. } => Some(source.as_ref()),
      ErrorKind::ReadFileFailed { source, .. } => Some(source),
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
