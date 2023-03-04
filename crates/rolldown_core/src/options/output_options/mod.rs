use std::str::FromStr;

use derivative::Derivative;

mod export_mode;
pub use export_mode::*;

use self::file_name::FileNameTemplate;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModuleFormat {
  Esm,
  Cjs,
  // AMD,
  // UMD,
}

impl ModuleFormat {
  pub fn is_es(self) -> bool {
    self == ModuleFormat::Esm
  }

  pub fn is_cjs(self) -> bool {
    self == ModuleFormat::Cjs
  }
}

impl FromStr for ModuleFormat {
  type Err = String;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    match value {
      "esm" => Ok(ModuleFormat::Esm),
      "cjs" => Ok(ModuleFormat::Cjs),
      _ => Err(format!("Invalid module format: {value}")),
    }
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct BuildOutputOptions {
  pub entry_file_names: FileNameTemplate,
  pub chunk_file_names: FileNameTemplate,
  pub format: ModuleFormat,
  pub export_mode: ExportMode,
}

impl Default for BuildOutputOptions {
  fn default() -> Self {
    Self {
      entry_file_names: FileNameTemplate::from("[name].js".to_string()),
      chunk_file_names: FileNameTemplate::from("[name]-[hash].js".to_string()),
      format: ModuleFormat::Esm,
      export_mode: ExportMode::Auto,
    }
  }
}

pub mod file_name {
  #[derive(Debug)]
  pub struct FileNameTemplate {
    template: String,
  }

  impl FileNameTemplate {
    pub fn new(template: String) -> Self {
      Self { template }
    }
  }

  impl From<String> for FileNameTemplate {
    fn from(template: String) -> Self {
      Self { template }
    }
  }

  #[derive(Debug, Default)]
  pub struct RenderOptions<'me> {
    pub name: Option<&'me str>,
  }

  impl FileNameTemplate {
    pub fn render(&self, options: RenderOptions) -> String {
      let mut tmp = self.template.clone();
      if let Some(name) = options.name {
        tmp = tmp.replace("[name]", name);
      }
      tmp
    }
  }
}
