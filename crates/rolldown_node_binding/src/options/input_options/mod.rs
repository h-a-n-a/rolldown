use std::{collections::HashMap, path::PathBuf};

use napi_derive::*;
use rolldown_core::InputItem;
use rolldown_plugin::BuildPlugin;
use serde::Deserialize;
mod external;
pub use external::*;
mod build_plugin;
pub use build_plugin::*;
mod builtins;
pub use builtins::*;

use crate::js_build_plugin::JsBuildPlugin;

#[napi(object)]
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct InputOptions {
  // Not going to be supported
  // @deprecated Use the "inlineDynamicImports" output option instead.
  // inlineDynamicImports?: boolean;

  // acorn?: Record<string, unknown>;
  // acornInjectPlugins?: (() => unknown)[] | (() => unknown);
  // cache?: false | RollupCache;
  // context?: string;sssssssssss
  // experimentalCacheExpiry?: number;
  pub external: ExternalOption,
  pub input: HashMap<String, String>,
  // makeAbsoluteExternalsRelative?: boolean | 'ifRelativeSource';
  // /** @deprecated Use the "manualChunks" output option instead. */
  // manualChunks?: ManualChunksOption;
  // maxParallelFileOps?: number;
  // /** @deprecated Use the "maxParallelFileOps" option instead. */
  // maxParallelFileReads?: number;
  // moduleContext?: ((id: string) => string | null | void) | { [id: string]: string };
  // onwarn?: WarningHandlerWithDefault;
  // perf?: boolean;
  pub plugins: Vec<BuildPluginOption>,
  // preserveEntrySignatures?: PreserveEntrySignaturesOption;
  // /** @deprecated Use the "preserveModules" output option instead. */
  // preserveModules?: boolean;
  pub preserve_symlinks: bool,
  // shimMissingExports?: boolean;
  // strictDeprecations?: boolean;
  pub treeshake: Option<bool>,
  // watch?: WatcherOptions | false;

  // extra
  pub cwd: String,
  pub builtins: BuiltinsOption,
}

pub fn resolve_input_options(
  opts: InputOptions,
) -> napi::Result<(rolldown_core::InputOptions, Vec<Box<dyn BuildPlugin>>)> {
  let cwd = PathBuf::from(opts.cwd.clone());
  assert!(cwd != PathBuf::from("/"), "{:#?}", opts);

  let mut plugins = opts
    .plugins
    .into_iter()
    .map(JsBuildPlugin::new_boxed)
    .try_collect::<Vec<_>>()?;

  let mut builtin_post_plugins = vec![];

  if let Some(node_resolve) = opts.builtins.node_resolve {
    builtin_post_plugins.push(rolldown_plugin_node_resolve::NodeResolvePlugin::new_boxed(
      rolldown_plugin_node_resolve::ResolverOptions {
        extensions: node_resolve.extensions,
        symlinks: !opts.preserve_symlinks,
        ..Default::default()
      },
      cwd.clone(),
    ))
  }

  plugins.extend(builtin_post_plugins);

  let is_external = resolve_external(opts.external)?;

  Ok((
    rolldown_core::InputOptions {
      input: opts
        .input
        .into_iter()
        .map(|(name, import)| InputItem { name, import })
        .collect(),
      cwd,
      treeshake: opts.treeshake.unwrap_or(true),
      is_external,
      ..Default::default()
    },
    plugins,
  ))
}
