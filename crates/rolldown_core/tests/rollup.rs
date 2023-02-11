use std::path::PathBuf;

use testing_macros::fixture;
mod common;
use common::snapshot;

#[fixture("./tests/rollup/**/test.config.json")]
fn test(path: PathBuf) {
  snapshot(&path)
}
