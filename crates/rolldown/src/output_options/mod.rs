use derivative::Derivative;
pub use rolldown_core::{file_name::FileNameTemplate, ExportMode, ModuleFormat};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct OutputOptions {
  pub dir: Option<String>,
  pub entry_file_names: FileNameTemplate,
  pub chunk_file_names: FileNameTemplate,
  pub format: ModuleFormat,
  pub export_mode: ExportMode,
}

impl Default for OutputOptions {
  fn default() -> Self {
    Self {
      entry_file_names: FileNameTemplate::from("[name].js".to_string()),
      chunk_file_names: FileNameTemplate::from("[name]-[hash].js".to_string()),
      dir: None,
      format: ModuleFormat::Esm,
      export_mode: ExportMode::Auto,
    }
  }
}
