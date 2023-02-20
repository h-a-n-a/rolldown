use swc_core::{
  common::Mark,
  ecma::{ast, transforms::base::resolver, visit::VisitMutWith},
};

pub fn resolve(ast: &mut ast::Module, unresolved_mark: Mark, top_level_mark: Mark) {
  ast.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));
}
