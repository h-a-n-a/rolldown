use std::path::PathBuf;

use common::snapshot;
use testing_macros::fixture;

mod common;

#[fixture("./tests/fixtures/**/test.config.json")]
fn test(path: PathBuf) {
  snapshot(&path)
}
