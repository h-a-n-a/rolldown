use swc_core::{
  common::{chain, util::take::Take, Mark, GLOBALS},
  ecma::{
    ast::Module,
    transforms::{
      base::{
        fixer::fixer,
        helpers::{inject_helpers, HELPERS},
        hygiene::hygiene,
        resolver,
      },
      proposal::decorators,
      typescript::{self, strip_with_config},
    },
    visit::FoldWith,
  },
};
pub use typescript::Config as TsConfig;

pub fn ts_to_js(ast: &mut Module, config: TsConfig) {
  // It's ok to use a new GLOBALS here.
  GLOBALS.set(&Default::default(), || {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    // Optionally transforms decorators here before the resolver pass
    // as it might produce runtime declarations.
    let mut chained = chain!(
      // TODO: should we transpile the decorators in this phase?
      // decorators::decorators(DecoratorConfig {
      //   use_define_for_class_fields: config.use_define_for_class_fields,
      // TODO: whats the correct default value?
      //   legacy: false,
      //   ..Default::default()
      // }),
      // Conduct identifier scope analysis
      resolver(unresolved_mark, top_level_mark, true),
      // Remove typescript types
      strip_with_config(config, top_level_mark),
      // Fix up any identifiers with the same name, but different contexts
      // Notice the resolved SyntaxContext is cleared by hygiene,
      // So we don't need to clear again.
      hygiene(),
      // Ensure that we have enough parenthesis.
      fixer(None),
      inject_helpers(unresolved_mark)
    );
    HELPERS.set(&Default::default(), || {
      *ast = ast.take().fold_with(&mut chained);
    })
  });
}
