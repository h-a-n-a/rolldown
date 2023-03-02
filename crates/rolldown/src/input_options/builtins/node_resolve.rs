use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NodeResolveOptions {
  pub extensions: Vec<String>,
}

impl Default for NodeResolveOptions {
  fn default() -> Self {
    Self {
      extensions: vec![
        ".js".to_string(),
        ".jsx".to_string(),
        ".ts".to_string(),
        ".tsx".to_string(),
      ],
    }
  }
}
