use std::str::FromStr;

use crate::BundleError;

#[derive(Debug, Clone, Copy)]
pub enum ExportMode {
  Auto,
  Named,
  Default,
  None,
}

impl ExportMode {
  pub fn is_auto(&self) -> bool {
    matches!(self, ExportMode::Auto)
  }

  pub fn is_named(&self) -> bool {
    matches!(self, ExportMode::Named)
  }

  pub fn is_default(&self) -> bool {
    matches!(self, ExportMode::Default)
  }

  pub fn is_none(&self) -> bool {
    matches!(self, ExportMode::None)
  }
}

impl FromStr for ExportMode {
  type Err = BundleError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    match value {
      "auto" => Ok(ExportMode::Auto),
      "named" => Ok(ExportMode::Named),
      "default" => Ok(ExportMode::Default),
      "none" => Ok(ExportMode::None),
      _ => Err(BundleError::invalid_export_option_value(value.to_string())),
    }
  }
}
