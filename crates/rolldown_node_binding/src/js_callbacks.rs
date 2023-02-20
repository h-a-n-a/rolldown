use crate::{options::ResolveIdResult, utils::JsCallback};

pub type IsExternalCallback = JsCallback<(String, Option<String>, bool), bool>;

// Build hooks
pub type ResolveIdCallback = JsCallback<(String, Option<String>), Option<ResolveIdResult>>;
pub type TransformCallback = JsCallback<(String, String), Option<String>>;
