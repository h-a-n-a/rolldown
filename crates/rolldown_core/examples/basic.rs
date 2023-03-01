use std::path::PathBuf;

use rolldown_core::{Bundler, InputItem, InputOptions, OutputOptions};
use rolldown_tracing::enable_tracing_on_demand;

#[tokio::main]
async fn main() {
  let _guard = enable_tracing_on_demand();
  let root = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let fixture_path = root.join("../../../three.js/");
  let dist_dir = root.join("examples/dist");

  let mut bundler = Bundler::new(InputOptions {
    input: vec![InputItem {
      name: "threejs".to_string(),
      import: "./src/Three.js".to_string(),
    }],
    cwd: fixture_path,
    ..Default::default()
  });

  let assets = bundler
    .write(OutputOptions {
      dir: Some(dist_dir.to_string_lossy().to_string()),
      ..Default::default()
    })
    .await
    .unwrap();
}
