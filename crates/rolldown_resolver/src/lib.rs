use std::path::PathBuf;

use sugar_path::{AsPath, SugarPathBuf};

#[derive(Debug)]
pub struct Resolver {
  cwd: PathBuf,
}

impl Resolver {
  pub fn with_cwd(cwd: PathBuf) -> Self {
    Self { cwd }
  }

  pub fn cwd(&self) -> &PathBuf {
    &self.cwd
  }
}

impl Default for Resolver {
  fn default() -> Self {
    Self {
      cwd: std::env::current_dir().unwrap(),
    }
  }
}

impl Resolver {
  pub fn resolve(&self, importer: Option<&str>, specifier: &str) -> rolldown_error::Result<String> {
    let mut path = if specifier.as_path().is_absolute() {
      specifier.as_path().to_path_buf()
    } else if let Some(importer) = importer {
      importer
        .as_path()
        .parent()
        .unwrap()
        .join(specifier)
        .into_absolutize()
    } else {
      self.cwd.as_path().join(specifier).into_absolutize()
    };

    add_js_extension(&mut path);
    let id = path.to_string_lossy().to_string();
    Ok(id)
  }
}

fn add_js_extension(path: &mut std::path::PathBuf) {
  if path.extension().is_none() {
    path.set_extension("js");
  }
}
