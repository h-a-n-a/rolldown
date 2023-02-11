use std::collections::HashMap;

use rolldown_core::{Bundler, InputOptions, OutputOptions};

#[tokio::main]
async fn main() {
  let mut bundler = Bundler::new(InputOptions {
    input: HashMap::from([(
      "main.js".to_string(),
      "/./rolldown/examples/basic/src/index".to_string(),
    )]),
    ..Default::default()
  });

  let assets = bundler
    .build(OutputOptions {
      ..Default::default()
    })
    .await;

  tracing::debug!("assets {assets:#?}")
}
