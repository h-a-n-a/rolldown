use std::path::PathBuf;

use testing_macros::fixture;

mod common;
use common::run_test;

#[fixture("./tests/fixtures/**/test.config.json")]
fn test(path: PathBuf) {
  run_test(&path)
}
