use rolldown_common::ModuleId;
use rolldown_plugin::ResolveArgs;
use rolldown_resolver::Resolver;
use sugar_path::AsPath;

use crate::{BuildResult, SharedBuildPluginDriver};

pub(crate) async fn resolve_id(
  resolver: &Resolver,
  args: ResolveArgs<'_>,
  plugin_driver: &SharedBuildPluginDriver,
) -> BuildResult<Option<ModuleId>> {
  let importer = args.importer.map(|id| id.as_ref());
  let specifier = args.specifier;

  let plugin_result = plugin_driver.read().await.resolve(args).await?;

  if plugin_result.is_some() {
    return Ok(
      plugin_result.map(|plugin_result| ModuleId::new(plugin_result.id, plugin_result.external)),
    );
  }

  // external modules (non-entry modules that start with neither '.' or '/')
  // are skipped at this stage.
  if importer.is_some() && !specifier.as_path().is_absolute() && !specifier.starts_with('.') {
    return Ok(None);
  }

  let resolved = resolver.resolve(importer, specifier)?;

  Ok(Some(ModuleId::new(resolved, false)))
}
