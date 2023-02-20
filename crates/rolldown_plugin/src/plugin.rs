use std::{borrow::Cow, fmt::Debug};

use crate::{Context, ResolveArgs, TransformArgs};

#[derive(Debug)]
pub struct ResolvedId {
  pub id: String,
  pub external: bool,
}

pub type ResolveOutput = rolldown_error::Result<Option<ResolvedId>>;
pub type TransformOutput = rolldown_error::Result<Option<String>>;
pub type PluginName<'a> = Cow<'a, str>;

#[async_trait::async_trait]
pub trait BuildPlugin: Debug + Send + Sync {
  fn name(&self) -> PluginName;

  async fn resolve(&self, _ctx: &mut Context, _args: &mut ResolveArgs) -> ResolveOutput {
    Ok(None)
  }

  async fn transform(&self, _ctx: &mut Context, _args: &mut TransformArgs) -> TransformOutput {
    Ok(None)
  }
}
