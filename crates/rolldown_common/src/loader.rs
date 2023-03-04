use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum Loader {
  Js,
  Jsx,
  Ts,
  Tsx,
  Json,
}

impl FromStr for Loader {
  type Err = rolldown_error::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "js" => Ok(Self::Js),
      "jsx" => Ok(Self::Jsx),
      "ts" => Ok(Self::Ts),
      "tsx" => Ok(Self::Tsx),
      _ => Err(rolldown_error::Error::panic(format!(
        "Unknown loader value \"{}\"",
        s
      ))),
    }
  }
}
