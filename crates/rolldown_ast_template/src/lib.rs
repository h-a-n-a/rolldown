use swc_core::{
  common::util::take::Take,
  ecma::{ast, atoms::JsWord, utils::quote_ident},
};

pub fn build_exports_stmt(mut exports: Vec<(JsWord, ast::Id)>) -> ast::ModuleItem {
  use ast::{ExportNamedSpecifier, ExportSpecifier, ModuleDecl, ModuleExportName, NamedExport};
  exports.sort_by(|a, b| a.0.cmp(&b.0));

  ast::ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
    span: Default::default(),
    specifiers: exports
      .into_iter()
      .filter(|(name, _)| name != "*")
      .map(|(name, id)| {
        ExportSpecifier::Named(ExportNamedSpecifier {
          span: Default::default(),
          orig: ModuleExportName::Ident(id.into()),
          exported: Some(quote_ident!(name).into()),
          is_type_only: false,
        })
      })
      .collect::<Vec<_>>(),
    src: None,
    type_only: false,
    asserts: None,
  }))
}

pub fn build_namespace_export_stmt(
  var_name: ast::Id,
  exports: Vec<(JsWord, ast::Id)>,
  external_module_ids: Vec<ast::Id>,
) -> ast::ModuleItem {
  use ast::*;
  let mut exported_name_and_local_id_list = exports.into_iter().collect::<Vec<_>>();
  exported_name_and_local_id_list.sort_by(|a, b| a.0.cmp(&b.0));
  let exports_props = [PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
    key: quote_ident!("__proto__").into(),
    value: Box::new(Expr::Lit(Lit::Null(Null::dummy()))),
  })))]
  .into_iter()
  .chain(
    &mut exported_name_and_local_id_list
      .into_iter()
      .map(|(exported_name, local_id)| {
        PropOrSpread::Prop(Box::new(Prop::Getter(GetterProp {
          span: Default::default(),
          key: Ident::verify_symbol(&exported_name)
            .map(|_| PropName::Ident(quote_ident!(exported_name.clone())))
            .unwrap_or_else(|_| PropName::Str(exported_name.into())),
          type_ann: None,
          body: Some(BlockStmt {
            span: Default::default(),
            stmts: vec![ast::Stmt::Return(ReturnStmt {
              span: Default::default(),
              arg: Some(local_id.into()),
            })],
          }),
        })))
      }),
  )
  .collect::<Vec<_>>();

  if external_module_ids.is_empty() {
    ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDecl(ast::ExportDecl {
      span: Default::default(),
      decl: ast::Decl::Var(Box::new(VarDecl {
        span: Default::default(),
        kind: VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
          span: Default::default(),
          definite: false,
          name: var_name.into(),
          init: Some(Box::new(Expr::Call(CallExpr {
            callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
              obj: quote_ident!("Object").into(),
              prop: quote_ident!("freeze").into(),
              ..MemberExpr::dummy()
            }))),
            args: vec![ExprOrSpread {
              expr: Box::new(Expr::Object(ObjectLit {
                span: Default::default(),
                props: exports_props,
              })),
              spread: None,
            }],
            ..CallExpr::dummy()
          }))),
        }],
      })),
    }))
  } else {
    let merge_namespace_call = Expr::Call(CallExpr {
      callee: Callee::Expr(quote_ident!("_mergeNamespaces").into()),
      args: vec![
        ExprOrSpread {
          expr: Box::new(Expr::Object(ObjectLit {
            span: Default::default(),
            props: exports_props,
          })),
          spread: None,
        },
        ExprOrSpread {
          expr: Box::new(ast::Expr::Array(ast::ArrayLit {
            span: Default::default(),
            elems: external_module_ids
              .into_iter()
              .map(|id| {
                Some(ExprOrSpread {
                  expr: id.into(),
                  spread: None,
                })
              })
              .collect(),
          })),
          spread: None,
        },
      ],
      ..CallExpr::dummy()
    });
    ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDecl(ast::ExportDecl {
      span: Default::default(),
      decl: Decl::Var(Box::new(VarDecl {
        span: Default::default(),
        kind: VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
          span: Default::default(),
          definite: false,
          name: var_name.into(),
          init: Some(Box::new(merge_namespace_call)),
        }],
      })),
    }))
  }
}
