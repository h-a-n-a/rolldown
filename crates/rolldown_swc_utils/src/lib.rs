use swc_core::ecma::{ast, atoms::JsWord};

pub trait ModuleExportNameExt {
  fn expect_ident(&self) -> &ast::Ident;
}

impl ModuleExportNameExt for ast::ModuleExportName {
  fn expect_ident(&self) -> &ast::Ident {
    match self {
      ast::ModuleExportName::Ident(ident) => ident,
      _ => panic!("Expected ident, but got {self:#?}"),
    }
  }
}

pub trait ImportNamedSpecifierExt {
  fn imported_name(&self) -> &JsWord;

  fn imported_as_ident(&self) -> &ast::Ident;
}

impl ImportNamedSpecifierExt for ast::ImportNamedSpecifier {
  fn imported_name(&self) -> &JsWord {
    match &self.imported {
      Some(imported) => &imported.expect_ident().sym,
      None => &self.local.sym,
    }
  }

  fn imported_as_ident(&self) -> &ast::Ident {
    &self.local
  }
}

pub trait ExportNamedSpecifierExt {
  fn local_ident(&self) -> &ast::Ident;
  fn exported_as_name(&self) -> &JsWord;
}

impl ExportNamedSpecifierExt for ast::ExportNamedSpecifier {
  fn local_ident(&self) -> &ast::Ident {
    self.orig.expect_ident()
  }

  fn exported_as_name(&self) -> &JsWord {
    match &self.exported {
      Some(exported) => &exported.expect_ident().sym,
      None => &self.orig.expect_ident().sym,
    }
  }
}
