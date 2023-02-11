use derivative::Derivative;
use napi::JsFunction;
use serde::Deserialize;

mod resolve_id_result;
pub use resolve_id_result::*;

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct BuildPluginOption {
  pub name: String,

  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(id: string, code: string) => Promise<string | null | undefined>")]
  pub transform: Option<JsFunction>,

  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(
    ts_type = "(specifier: string, importer?: string) => Promise<string | null | ResolveIdResult>"
  )]
  pub resolve_id: Option<JsFunction>,
}
