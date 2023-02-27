use swc_core::{
  common::{chain, util::take::Take, Mark, GLOBALS},
  ecma::{
    ast::Module,
    transforms::{
      base::{fixer::fixer, hygiene::hygiene, resolver},
      typescript::strip,
    },
    visit::FoldWith,
  },
};

pub fn ts_to_js(ast: &mut Module) {
  // It's ok to use a new GLOBALS here.
  GLOBALS.set(&Default::default(), || {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    // Optionally transforms decorators here before the resolver pass
    // as it might produce runtime declarations.

    let mut chained = chain!(
      // Conduct identifier scope analysis
      resolver(unresolved_mark, top_level_mark, true),
      // Remove typescript types
      strip(top_level_mark),
      // Fix up any identifiers with the same name, but different contexts
      // Notice the resolved SyntaxContext is cleared by hygiene,
      // So we don't need to clear again.
      hygiene(),
      // Ensure that we have enough parenthesis.
      fixer(None),
    );
    *ast = ast.take().fold_with(&mut chained);
  });
}
