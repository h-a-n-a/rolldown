use std::sync::Arc;

use derivative::Derivative;
use futures::FutureExt;
use napi::JsFunction;
use rustc_hash::FxHashSet;
use serde::Deserialize;

use crate::{js_callbacks::IsExternalCallback, utils::NapiErrorExt};

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct ExternalOption {
  #[serde(skip_deserializing)]
  #[napi(
    ts_type = "(specifier: string, importer: string | undefined, isResolved: boolean) => boolean"
  )]
  #[derivative(Debug = "ignore")]
  pub function: Option<JsFunction>,
  pub string: Vec<String>,
}

pub fn resolve_external(is_external: ExternalOption) -> napi::Result<rolldown::core::IsExternal> {
  let is_external_cb = is_external
    .function
    .as_ref()
    .map(IsExternalCallback::new)
    .transpose()?;

  let string_pattern = Arc::new(is_external.string.into_iter().collect::<FxHashSet<_>>());

  Ok(Arc::new(move |specifier, importer, is_resolved| {
    let string_pattern = string_pattern.clone();
    let is_external_cb = is_external_cb.clone();

    let importer = importer.map(|s| s.to_string());
    let specifier = specifier.to_string();
    let is_resolved = is_resolved;
    async move {
      if string_pattern.contains(&specifier) {
        return Ok(true);
      }
      if let Some(cb) = is_external_cb.clone() {
        cb.call_async((specifier, importer, is_resolved))
          .await
          .map_err(|e| e.into_bundle_error())
      } else {
        Ok(false)
      }
    }
    .boxed()
  }))
}
