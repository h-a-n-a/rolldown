use std::sync::atomic::AtomicBool;

use tracing::Level;

static IS_INIT: AtomicBool = AtomicBool::new(false);
pub fn init() {
  use tracing_subscriber::{fmt, prelude::*, EnvFilter};
  if !IS_INIT.swap(true, std::sync::atomic::Ordering::SeqCst) {
    tracing_subscriber::registry()
      .with(fmt::layer())
      .with(EnvFilter::from_default_env())
      .with(
        tracing_subscriber::filter::Targets::new().with_targets(vec![("rolldown", Level::TRACE)]),
      )
      .init();
  }
}
