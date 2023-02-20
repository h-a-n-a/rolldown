use std::{collections::HashMap, path::PathBuf};

use napi_derive::*;
use rolldown_plugin::BuildPlugin;
use serde::Deserialize;
mod external;
pub use external::*;
mod build_plugin;
pub use build_plugin::*;

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
  // preserveSymlinks?: boolean;
  // shimMissingExports?: boolean;
  // strictDeprecations?: boolean;
  pub treeshake: Option<bool>,
  // watch?: WatcherOptions | false;

  // extra
  pub cwd: Option<String>,
}

pub fn resolve_input_options(
  opts: InputOptions,
) -> napi::Result<(rolldown_core::InputOptions, Vec<Box<dyn BuildPlugin>>)> {
  let plugins = opts
    .plugins
    .into_iter()
    .map(JsBuildPlugin::new_boxed)
    .try_collect::<Vec<_>>()?;

  let is_external = resolve_external(opts.external)?;

  Ok((
    rolldown_core::InputOptions {
      input: opts.input,
      cwd: opts
        .cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap()),
      treeshake: opts.treeshake.unwrap_or(true),
      is_external,
      ..Default::default()
    },
    plugins,
  ))
}
