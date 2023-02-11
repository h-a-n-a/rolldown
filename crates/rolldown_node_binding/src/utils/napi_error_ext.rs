use rolldown_core::BundleError;

pub(crate) trait NapiErrorExt {
  fn into_bundle_error(self) -> BundleError;
}

impl NapiErrorExt for napi::Error {
  fn into_bundle_error(self) -> BundleError {
    BundleError::Napi {
      status: self.status.to_string(),
      reason: self.reason.clone(),
    }
  }
}
