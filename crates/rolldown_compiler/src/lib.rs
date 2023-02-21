use std::sync::Arc;

use ast::EsVersion;
use sugar_path::AsPath;
use swc_common::{
  comments::Comments,
  errors::{ColorConfig, Handler},
  FileName, SourceMap,
};
use swc_core::{
  common::{self as swc_common, SourceFile},
  ecma::{
    ast, codegen as swc_ecma_codegen,
    parser::{self as swc_ecma_parser, PResult},
    visit as swc_ecma_visit,
  },
};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::{VisitMut, VisitMutWith};

#[derive(Default)]
pub struct Compiler {
  pub cm: Arc<SourceMap>,
}

impl Compiler {
  pub fn with_cm(cm: Arc<SourceMap>) -> Self {
    Self { cm }
  }

  pub fn print(
    &self,
    ast: &ast::Module,
    comments: Option<&dyn Comments>,
  ) -> anyhow::Result<String> {
    let mut output = Vec::new();

    let mut emitter = swc_ecma_codegen::Emitter {
      cfg: swc_ecma_codegen::Config {
        ..Default::default()
      },
      cm: self.cm.clone(),
      comments: Some(&comments),
      wr: Box::new(JsWriter::new(self.cm.clone(), "\n", &mut output, None)),
    };

    emitter.emit_module(ast)?;
    String::from_utf8(output).map_err(Into::into)
  }

  pub fn print_module_item(
    &self,
    ast: &ast::ModuleItem,
    comments: Option<&dyn Comments>,
  ) -> anyhow::Result<String> {
    let mut output = Vec::new();

    let mut emitter = swc_ecma_codegen::Emitter {
      cfg: swc_ecma_codegen::Config {
        ..Default::default()
      },
      cm: self.cm.clone(),
      comments: Some(&comments),
      wr: Box::new(JsWriter::new(self.cm.clone(), "\n", &mut output, None)),
    };

    emitter.emit_module_item(ast)?;
    String::from_utf8(output).map_err(Into::into)
  }

  pub fn debug_print(
    &self,
    ast: &ast::Module,
    comments: Option<&dyn Comments>,
  ) -> anyhow::Result<String> {
    let mut ast = ast.clone();
    ast.visit_mut_with(&mut SyntaxContextVisualizer);
    let ast = &ast;
    let mut output = Vec::new();

    let mut emitter = swc_ecma_codegen::Emitter {
      cfg: Default::default(),
      cm: self.cm.clone(),
      comments: Some(&comments),
      wr: Box::new(JsWriter::new(self.cm.clone(), "\n", &mut output, None)),
    };

    emitter.emit_module(ast)?;
    String::from_utf8(output).map_err(Into::into)
  }

  pub fn parse(
    &self,
    source_code: String,
    filename: &str,
  ) -> (Arc<SourceFile>, PResult<ast::Module>) {
    self.parse_with_comments(source_code, filename, None)
  }

  pub fn parse_with_comments(
    &self,
    source_code: String,
    filename: &str,
    comments: Option<&dyn Comments>,
  ) -> (Arc<SourceFile>, PResult<ast::Module>) {
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(self.cm.clone()));
    let p = filename.as_path();
    let fm = self
      .cm
      .new_source_file(FileName::Custom(filename.to_owned()), source_code);
    let ext = p.extension().and_then(|ext| ext.to_str()).unwrap_or("js");
    let syntax = if ext == "ts" || ext == "tsx" {
      Syntax::Typescript(TsConfig {
        decorators: false,
        tsx: ext == "tsx",
        ..Default::default()
      })
    } else {
      Syntax::Es(Default::default())
    };

    let lexer = Lexer::new(
      syntax,
      EsVersion::latest(),
      StringInput::from(fm.as_ref()),
      comments,
    );
    let mut parser = Parser::new_from(lexer);
    parser.take_errors().into_iter().for_each(|e| {
      e.into_diagnostic(&handler).emit();
    });
    // To be clear, rolldown will always assume the input is a module
    (fm.clone(), parser.parse_module())
  }
}

struct SyntaxContextVisualizer;

impl VisitMut for SyntaxContextVisualizer {
  fn visit_mut_ident(&mut self, ident: &mut ast::Ident) {
    if ident.span.ctxt.as_u32() != 0 {
      ident.sym = format!("{}#{:?}", ident.sym, ident.span.ctxt.as_u32()).into();
    }
  }
}