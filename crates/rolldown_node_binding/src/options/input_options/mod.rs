use std::{collections::HashMap, path::PathBuf};

use napi_derive::*;
use rolldown::default_warning_handler;
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
) -> napi::Result<(rolldown::InputOptions, Vec<Box<dyn BuildPlugin>>)> {
  let cwd = PathBuf::from(opts.cwd.clone());
  assert!(cwd != PathBuf::from("/"), "{:#?}", opts);

  let plugins = opts
    .plugins
    .into_iter()
    .map(JsBuildPlugin::new_boxed)
    .try_collect::<Vec<_>>()?;

  let is_external = resolve_external(opts.external)?;

  Ok((
    rolldown::InputOptions {
      input: opts
        .input
        .into_iter()
        .map(|(name, import)| rolldown::core::InputItem { name, import })
        .collect(),
      cwd,
      treeshake: opts.treeshake.unwrap_or(true),
      is_external,
      preserve_symlinks: opts.preserve_symlinks,
      builtins: rolldown::BuiltinsOptions {
        node_resolve: opts
          .builtins
          .node_resolve
          .map(|opts| rolldown::NodeResolveOptions {
            extensions: opts.extensions,
          }),
      },
      on_warn: default_warning_handler(),
      shim_missing_exports: false,
    },
    plugins,
  ))
}
