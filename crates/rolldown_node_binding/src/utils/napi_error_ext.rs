use rolldown_core::BundleError;

pub(crate) trait NapiErrorExt {
  fn into_bundle_error(self) -> BundleError;
}

impl NapiErrorExt for napi::Error {
  fn into_bundle_error(self) -> BundleError {
    BundleError::napi_error(self.status.to_string(), self.reason.clone())
  }
}
