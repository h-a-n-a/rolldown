use swc_core::ecma::{ast, utils::member_expr, visit::VisitMut};

struct DefaultExportModeShimer;

impl VisitMut for DefaultExportModeShimer {
  fn visit_mut_module(&mut self, node: &mut ast::Module) {
    // apeend `module.exports = exports.default`
    let item = ast::ModuleItem::Stmt(ast::Stmt::Expr(ast::ExprStmt {
      span: Default::default(),
      expr: Box::new(ast::Expr::Assign(ast::AssignExpr {
        span: Default::default(),
        op: ast::AssignOp::Assign,
        left: ast::PatOrExpr::Expr(member_expr!(Default::default(), module.exports)),
        right: member_expr!(Default::default(), exports.default),
      })),
    }));
    node.body.push(item);
  }
}

pub fn default_export_mode_shimer() -> impl VisitMut {
  DefaultExportModeShimer
}
