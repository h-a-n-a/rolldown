use std::str::FromStr;

use napi_derive::*;
use rolldown::ModuleFormat;
use serde::Deserialize;

#[napi(object)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputOptions {
  // --- Options Rolldown doesn't need to be supported
  // /** @deprecated Use the "renderDynamicImport" plugin hook instead. */
  // dynamicImportFunction: string | undefined;
  pub entry_file_names: Option<String>,
  pub chunk_file_names: Option<String>,

  // amd: NormalizedAmdOptions;
  // assetFileNames: string | ((chunkInfo: PreRenderedAsset) => string);
  // banner: () => string | Promise<string>;
  // chunkFileNames: string | ((chunkInfo: PreRenderedChunk) => string);
  // compact: boolean;
  pub dir: Option<String>,
  // pub entry_file_names: String, // | ((chunkInfo: PreRenderedChunk) => string)
  // esModule: boolean;
  #[napi(ts_type = "'default' | 'named' | 'none' | 'auto'")]
  pub exports: Option<String>,
  // extend: boolean;
  // externalLiveBindings: boolean;
  // footer: () => string | Promise<string>;
  #[napi(ts_type = "'esm' | 'cjs'")]
  pub format: Option<String>,
  // freeze: boolean;
  // generatedCode: NormalizedGeneratedCodeOptions;
  // globals: GlobalsOption;
  // hoistTransitiveImports: boolean;
  // indent: true | string;
  // inlineDynamicImports: boolean;
  // interop: GetInterop;
  // intro: () => string | Promise<string>;
  // manualChunks: ManualChunksOption;
  // minifyInternalExports: boolean;
  // name: string | undefined;
  // namespaceToStringTag: boolean;
  // noConflict: boolean;
  // outro: () => string | Promise<string>;
  // paths: OptionsPaths;
  // plugins: OutputPlugin[];
  // preferConst: boolean;
  // preserveModules: boolean;
  // preserveModulesRoot: string | undefined;
  // sanitizeFileName: (fileName: string) => string;
  // sourcemap: boolean | 'inline' | 'hidden';
  // sourcemapExcludeSources: boolean;
  // sourcemapFile: string | undefined;
  // sourcemapPathTransform: SourcemapPathTransformOption | undefined;
  // strict: boolean;
  // systemNullSetters: boolean;
  // validate: boolean;
  // --- Enhanced options
  // pub minify: bool,
}

pub fn resolve_output_options(opts: OutputOptions) -> napi::Result<rolldown::OutputOptions> {
  let mut defaults = rolldown::OutputOptions::default();

  opts
    .entry_file_names
    .inspect(|entry_file_names| defaults.entry_file_names = entry_file_names.clone().into());

  if let Some(chunk_file_names) = opts.chunk_file_names {
    defaults.chunk_file_names = chunk_file_names.into()
  }
  if let Some(format) = opts.format {
    defaults.format = ModuleFormat::from_str(format.as_str()).map_err(|err| {
      napi::Error::new(
        napi::Status::InvalidArg,
        format!("Invalid module format {}", err),
      )
    })?;
  }

  defaults.dir = opts.dir;

  Ok(defaults)
}
