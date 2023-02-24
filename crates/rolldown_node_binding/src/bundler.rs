use napi::{tokio::sync::Mutex, Env};
use napi_derive::*;
use rolldown_core::{error::Errors, Bundler as BundlerCore};

use crate::{
  options::InputOptions,
  options::{resolve_input_options, resolve_output_options, OutputOptions},
  output_chunk::OutputChunk,
  NAPI_ENV,
};

#[napi]
pub struct Bundler {
  inner: Mutex<BundlerCore>,
}

#[napi]
impl Bundler {
  #[napi(constructor)]
  pub fn new(env: Env, input_opts: InputOptions) -> napi::Result<Self> {
    Self::new_impl(env, input_opts)
  }

  #[napi]
  pub async fn write(&self, opts: OutputOptions) -> napi::Result<Vec<OutputChunk>> {
    self.write_impl(opts).await
  }

  #[napi]
  pub async fn generate(&self, opts: OutputOptions) -> napi::Result<Vec<OutputChunk>> {
    self.generate_impl(opts).await
  }
}

impl Bundler {
  pub fn new_impl(env: Env, input_opts: InputOptions) -> napi::Result<Self> {
    rolldown_tracing::init();
    NAPI_ENV.set(&env, || {
      let (input_opts, plugins) = resolve_input_options(input_opts)?;
      Ok(Bundler {
        inner: Mutex::new(BundlerCore::with_plugins(input_opts, plugins)),
      })
    })
  }

  pub async fn write_impl(&self, opts: OutputOptions) -> napi::Result<Vec<OutputChunk>> {
    let mut bundler_core = self.inner.try_lock().map_err(|_| {
      napi::Error::from_reason("Failed to lock the bundler. Is another operation in progress?")
    })?;

    let binding_opts = resolve_output_options(opts)?;

    let outputs = bundler_core
      .write(binding_opts)
      .await
      .map_err(|err| self.handle_errors(err))?;

    let output_chunks = outputs
      .into_iter()
      .map(|asset| OutputChunk {
        code: asset.content,
        file_name: asset.filename,
      })
      .collect::<Vec<_>>();
    Ok(output_chunks)
  }

  pub async fn generate_impl(&self, opts: OutputOptions) -> napi::Result<Vec<OutputChunk>> {
    let mut bundler_core = self.inner.try_lock().map_err(|_| {
      napi::Error::from_reason("Failed to lock the bundler. Is another operation in progress?")
    })?;

    let binding_opts = resolve_output_options(opts)?;

    let outputs = bundler_core
      .generate(binding_opts)
      .await
      .map_err(|err| self.handle_errors(err))?;

    let output_chunks = outputs
      .into_iter()
      .map(|asset| OutputChunk {
        code: asset.content,
        file_name: asset.filename,
      })
      .collect::<Vec<_>>();
    Ok(output_chunks)
  }

  fn handle_errors(&self, errors: Errors) -> napi::Error {
    for error in errors.into_vec().into_iter() {
      eprintln!("{}", error);
    }
    napi::Error::from_reason("Build failed")
  }
}
