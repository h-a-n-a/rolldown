use std::path::PathBuf;

use rolldown::{Bundler, InputItem, InputOptions, OutputOptions};
use rolldown_tracing::enable_tracing_on_demand;

#[tokio::main]
async fn main() {
  let _guard = enable_tracing_on_demand();
  let project_root = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());

  let mut bundler = Bundler::new(InputOptions {
    input: vec![InputItem {
      name: "threejs-10x".to_string(),
      import: project_root
        .join("../../temp/threejs10x/main.js")
        .to_string_lossy()
        .to_string(),
    }],
    cwd: project_root.join("../../temp/threejs10x/"),
    ..Default::default()
  });
  bundler
    .write(OutputOptions {
      ..Default::default()
    })
    .await
    .unwrap();
}
