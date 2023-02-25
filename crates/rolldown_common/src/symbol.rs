use std::fmt::Debug;

use swc_core::{
  common::SyntaxContext,
  ecma::{ast, atoms::JsWord},
};

#[derive(Hash, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Symbol(ast::Id);

impl Symbol {
  pub fn new(name: JsWord, ctxt: SyntaxContext) -> Self {
    Self((name, ctxt))
  }

  pub fn name(&self) -> &JsWord {
    &self.0 .0
  }

  pub fn ctxt(&self) -> SyntaxContext {
    self.0 .1
  }

  pub fn to_id(self) -> ast::Id {
    self.into()
  }

  pub fn as_id(&self) -> &ast::Id {
    &self.0
  }
}

impl From<ast::Id> for Symbol {
  fn from(id: ast::Id) -> Self {
    Self(id)
  }
}

impl From<Symbol> for ast::Id {
  fn from(s: Symbol) -> Self {
    s.0
  }
}

impl Debug for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let name = self.0 .0.as_ref();
    let syntax_context = self.0 .1.as_u32();
    f.debug_tuple(&format!("Symbol({name}#{syntax_context})"))
      .finish()
  }
}

impl AsRef<ast::Id> for Symbol {
  fn as_ref(&self) -> &ast::Id {
    &self.0
  }
}
