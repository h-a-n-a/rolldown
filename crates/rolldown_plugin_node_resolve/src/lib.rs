use std::path::Path;

use nodejs_resolver::{ResolveResult, Resolver};
use rolldown_plugin::{async_trait, BuildPlugin, Context, ResolveArgs, ResolveOutput, ResolvedId};

#[derive(Debug)]
pub struct NodeResolvePlugin {}

impl NodeResolvePlugin {
  pub fn new_boxed() -> Box<dyn BuildPlugin> {
    Box::new(Self {})
  }
}

#[async_trait::async_trait]
impl BuildPlugin for NodeResolvePlugin {
  fn name(&self) -> rolldown_plugin::PluginName {
    std::borrow::Cow::Borrowed("builtin:node-resolve")
  }

  async fn resolve(&self, _ctx: &mut Context, args: &mut ResolveArgs) -> ResolveOutput {
    let resolver = Resolver::new(nodejs_resolver::Options {
      extensions: vec![
        ".js".to_string(),
        ".jsx".to_string(),
        ".ts".to_string(),
        ".tsx".to_string(),
      ],
      ..Default::default()
    });
    if let Some(importer) = args.importer {
      let s = resolver.resolve(
        &Path::new(importer.as_ref())
          .canonicalize()
          .unwrap()
          .parent()
          .unwrap(),
        args.specifier,
      );
      if s.is_err() {
        // println!("{args:#?}");
        // println!(
        //   "importer: {:#?}",
        //   &Path::new(importer.as_ref())
        //     .canonicalize()
        //     .unwrap()
        //     .parent()
        //     .unwrap()
        //     .display()
        // );
        return Ok(None);
      }
      let s = s.unwrap();
      match s {
        ResolveResult::Info(info) => Ok(Some(ResolvedId {
          id: info.path().to_string_lossy().to_string(),
          external: false,
        })),
        ResolveResult::Ignored => Ok(None),
      }
    } else {
      Ok(None)
    }
  }
}
