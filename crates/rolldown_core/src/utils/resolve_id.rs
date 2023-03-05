use rolldown_common::ModuleId;
use rolldown_plugin::ResolveArgs;
use rolldown_resolver::Resolver;
use sugar_path::AsPath;

use crate::{SharedBuildPluginDriver, UnaryBuildResult};

pub(crate) async fn resolve_id(
  resolver: &Resolver,
  specifier: &str,
  importer: Option<&ModuleId>,
  _preserve_symlinks: bool,
  plugin_driver: &SharedBuildPluginDriver,
) -> UnaryBuildResult<Option<ModuleId>> {
  let plugin_result = plugin_driver
    .read()
    .await
    .resolve(ResolveArgs {
      importer,
      specifier,
    })
    .await?;

  if plugin_result.is_some() {
    return Ok(
      plugin_result.map(|plugin_result| ModuleId::new(plugin_result.id, plugin_result.external)),
    );
  }

  let importer = importer.map(|id| id.as_ref());
  // external modules (non-entry modules that start with neither '.' or '/')
  // are skipped at this stage.
  if importer.is_some() && !specifier.as_path().is_absolute() && !specifier.starts_with('.') {
    return Ok(None);
  }

  let resolved = resolver.resolve(importer, specifier)?;

  Ok(Some(ModuleId::new(resolved, false)))
}
