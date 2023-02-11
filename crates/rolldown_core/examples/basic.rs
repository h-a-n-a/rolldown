use std::{collections::HashMap, path::PathBuf};

use rolldown_core::{Bundler, InputOptions, OutputOptions};

#[tokio::main]
async fn main() {
  let root = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let fixture_path = root.join("tests/esbuild/import_star/import_star_unused");
  let dist_dir = root.join("examples/dist");

  let mut bundler = Bundler::new(InputOptions {
    input: HashMap::from([("main.js".to_string(), "./entry".to_string())]),
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

  println!("assets {assets:#?}")
}
