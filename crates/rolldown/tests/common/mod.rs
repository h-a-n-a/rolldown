use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use rolldown::Bundler;
use rolldown::{Asset, BuildResult, ExportMode, ModuleFormat, OutputOptions};
use rolldown_test_utils::tester::Tester;

pub struct CompiledFixture {
  pub tester: Tester,
  pub bundler: Bundler,
  pub output: BuildResult<Vec<Asset>>,
  pub dir_name: String,
  pub fixture_path: PathBuf,
}

impl CompiledFixture {
  pub fn output_friendly_to_snapshot(&self) -> String {
    let mut assets = self.output.as_ref().unwrap().iter().collect::<Vec<_>>();
    assets.sort_by_key(|c| &c.filename);
    assets
      .iter()
      .flat_map(|asset| {
        [
          format!("---------- {} ----------", asset.filename),
          asset.content.trim().to_string(),
        ]
      })
      .chain(if self.tester.warnings.lock().unwrap().is_empty() {
        vec![]
      } else {
        let mut warnings = self.tester.warnings.lock().unwrap();
        warnings.sort();
        vec![
          format!("---------- WARNINGS ----------"),
          warnings
            .iter()
            .map(|w| {
              format!(
                "{}: {}",
                w.kind.code(),
                w.kind.to_readable_string(&self.fixture_path)
              )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        ]
      })
      .collect::<Vec<_>>()
      .join("\n")
  }
}

pub async fn compile_fixture(test_config_path: &Path) -> CompiledFixture {
  let fixture_path = test_config_path.parent().unwrap();

  let tester = Tester::from_config_path(test_config_path);

  let mut bundler = Bundler::new(tester.input_options(fixture_path.to_path_buf()));

  if fixture_path.join("dist").is_dir() {
    std::fs::remove_dir_all(fixture_path.join("dist")).unwrap();
  }

  let output = bundler
    .generate(OutputOptions {
      // dir: Some(fixture_path.join("dist").to_string_lossy().to_string()),
      format: ModuleFormat::from_str(&tester.config.output.format).unwrap(),
      export_mode: ExportMode::from_str(&tester.config.output.export_mode).unwrap(),
      ..Default::default()
    })
    .await;
  let fixture_name = fixture_path
    .file_name()
    .unwrap()
    .to_string_lossy()
    .to_string();

  CompiledFixture {
    tester,
    bundler,
    output,
    dir_name: fixture_name,
    fixture_path: fixture_path.to_path_buf(),
  }
}

pub fn run_test(test_config_path: &Path) {
  // compile the fixture folder
  let compiled_fx = tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(crate::common::compile_fixture(test_config_path));

  // If the test config has an expected error, assert that the error matches
  if let Some(expected_error) = compiled_fx.tester.config.expected_error {
    let errors = compiled_fx
      .output
      .expect_err("Expected error but got success")
      .into_vec();
    assert_eq!(errors.len(), 1);
    let error = errors.into_iter().next().unwrap();
    assert_eq!(error.kind.code(), expected_error.code);
    assert_eq!(
      error.kind.to_readable_string(&compiled_fx.fixture_path),
      expected_error.message
    );
    return;
  }

  // Otherwise, assert that the output matches the snapshot

  // Configure insta to use the test config path as the snapshot path
  let fixture_folder = test_config_path.parent().unwrap();
  let mut settings = insta::Settings::clone_current();
  settings.set_snapshot_path(fixture_folder);
  settings.set_prepend_module_to_snapshot(false);
  settings.set_input_file(fixture_folder);
  settings.bind(|| {
    insta::assert_snapshot!("output", compiled_fx.output_friendly_to_snapshot());
  });
}
