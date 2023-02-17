use std::str::FromStr;

use derivative::Derivative;

use self::file_name::FileNameTemplate;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InternalModuleFormat {
  Esm,
  Cjs,
  // AMD,
  // UMD,
}

impl InternalModuleFormat {
  pub fn is_es(self) -> bool {
    self == InternalModuleFormat::Esm
  }

  pub fn is_cjs(self) -> bool {
    self == InternalModuleFormat::Cjs
  }
}

impl FromStr for InternalModuleFormat {
  type Err = String;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    match value {
      "esm" => Ok(InternalModuleFormat::Esm),
      "cjs" => Ok(InternalModuleFormat::Cjs),
      _ => Err(format!("Invalid module format: {value}")),
    }
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct OutputOptions {
  pub entry_file_names: FileNameTemplate,
  pub chunk_file_names: FileNameTemplate,
  pub dir: Option<String>,
  pub format: InternalModuleFormat,
}

impl Default for OutputOptions {
  fn default() -> Self {
    Self {
      entry_file_names: FileNameTemplate::from("[name].js".to_string()),
      chunk_file_names: FileNameTemplate::from("[name]-[hash].js".to_string()),
      dir: None,
      format: InternalModuleFormat::Esm,
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
