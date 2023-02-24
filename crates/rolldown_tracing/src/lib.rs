use std::sync::{atomic::AtomicBool, Arc};

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

#[derive(Debug, Default, Clone)]
pub struct ContextedTracer {
  context: Vec<Arc<String>>,
}

impl ContextedTracer {
  pub fn context(mut self, ctxt: String) -> Self {
    self.context.push(ctxt.into());
    self
  }

  pub fn emit_trace(&self, info: String) {
    for ctxt in &self.context {
      tracing::trace!("{}: {}", ansi_term::Color::Yellow.paint("context"), ctxt);
    }
    tracing::trace!(info)
  }
}
