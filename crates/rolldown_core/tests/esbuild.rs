// Tests are copied from https://github.com/evanw/esbuild/tree/main/internal/bundler_tests

use std::path::PathBuf;

use testing_macros::fixture;
mod common;

use crate::common::snapshot;

#[fixture("./tests/esbuild/**/test.config.json")]
fn test(path: PathBuf) {
  snapshot(&path)
}
