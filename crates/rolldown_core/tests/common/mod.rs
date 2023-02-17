use std::{path::Path, str::FromStr};

use rolldown_core::{Asset, Bundler, InternalModuleFormat, OutputOptions};
use rolldown_test_utils::TestConfig;

pub struct CompiledFixture {
  pub bundler: Bundler,
  pub assets: Vec<Asset>,
  pub name: String,
}

impl CompiledFixture {
  pub fn output_friendly_to_snapshot(&self) -> String {
    let mut assets = self.assets.iter().collect::<Vec<_>>();
    assets.sort_by_key(|c| &c.filename);
    assets
      .iter()
      .flat_map(|asset| {
        [
          format!("---------- {} ----------", asset.filename),
          asset.content.trim().to_string(),
        ]
      })
      .collect::<Vec<_>>()
      .join("\n")
  }
}

pub async fn compile_fixture(test_config_path: &Path) -> CompiledFixture {
  let fixture_path = test_config_path.parent().unwrap();

  let test_config: TestConfig = TestConfig::from_config_path(test_config_path);

  let mut bundler =
    Bundler::new(test_config.input_options(fixture_path.to_string_lossy().to_string()));

  if fixture_path.join("dist").is_dir() {
    std::fs::remove_dir_all(fixture_path.join("dist")).unwrap();
  }

  let assets = bundler
    .generate(OutputOptions {
      // dir: Some(fixture_path.join("dist").to_string_lossy().to_string()),
      format: InternalModuleFormat::from_str(&test_config.format)
        .unwrap_or(InternalModuleFormat::Esm),
      ..Default::default()
    })
    .await
    .unwrap();
  let fixture_name = fixture_path
    .file_name()
    .unwrap()
    .to_string_lossy()
    .to_string();

  CompiledFixture {
    bundler,
    assets,
    name: fixture_name,
  }
}

pub fn snapshot(test_config_path: &Path) {
  let fixture_folder = test_config_path.parent().unwrap();
  let mut settings = insta::Settings::clone_current();
  settings.set_snapshot_path(fixture_folder);
  settings.set_prepend_module_to_snapshot(false);
  settings.set_input_file(fixture_folder);
  tokio::runtime::Runtime::new().unwrap().block_on(async {
    let res = crate::common::compile_fixture(test_config_path).await;
    settings.bind(|| {
      insta::assert_snapshot!("output", res.output_friendly_to_snapshot());
    });
  });
}
