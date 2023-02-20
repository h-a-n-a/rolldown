// Tests are copied from https://github.com/evanw/esbuild/tree/main/internal/bundler_tests

use std::path::PathBuf;

use testing_macros::fixture;
mod common;
use common::run_test;

#[fixture("./tests/esbuild/**/test.config.json")]
fn test(path: PathBuf) {
  run_test(&path)
}
