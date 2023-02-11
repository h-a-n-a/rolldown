use rolldown_common::ModuleId;

#[derive(Debug, Clone)]
pub struct ResolveArgs<'a> {
  pub importer: Option<&'a ModuleId>,
  pub specifier: &'a str,
}

pub struct TransformArgs<'a> {
  pub id: &'a ModuleId,
  pub code: &'a String,
}
