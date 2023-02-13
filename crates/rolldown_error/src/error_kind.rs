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
  ParseFailed(String, String),
}
