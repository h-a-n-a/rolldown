



use napi_derive::*;

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
  // exports: 'default' | 'named' | 'none' | 'auto';
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

pub fn resolve_output_options(opts: OutputOptions) -> napi::Result<rolldown_core::OutputOptions> {
  let mut defaults = rolldown_core::OutputOptions::default();

  opts
    .entry_file_names
    .inspect(|entry_file_names| defaults.entry_file_names = entry_file_names.clone().into());

  opts
    .chunk_file_names
    .map(|chunk_file_names| defaults.chunk_file_names = chunk_file_names.into());

  opts
    .format
    .map(|format| defaults.format = format.as_str().try_into().unwrap());

  defaults.dir = opts.dir;

  Ok(defaults)
}
