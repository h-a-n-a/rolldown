use std::sync::atomic::AtomicBool;

use ast::{CallExpr, Callee, ExportSpecifier, Expr, Id, Ident, Lit, ModuleDecl, ModuleItem, Stmt};
use hashlink::LinkedHashSet;
use rolldown_common::{ExportedSpecifier, ImportedSpecifier, Symbol};
use rolldown_common::{ModuleId, ReExportedSpecifier};
use rolldown_swc_utils::{ExportNamedSpecifierExt, ImportNamedSpecifierExt, ModuleExportNameExt};
use rustc_hash::{FxHashMap as HashMap, FxHashMap, FxHashSet as HashSet, FxHashSet};
use swc_atoms::JsWord;
use swc_common::SyntaxContext;
use swc_core::{
  common::{self as swc_common, util::take::Take},
  ecma::{
    ast,
    atoms::{self as swc_atoms, js_word},
    utils::{class_has_side_effect, var::VarCollector, ExprCtx, ExprExt},
    visit as swc_ecma_visit,
  },
};
use swc_ecma_visit::{noop_visit_mut_type, Visit, VisitMut, VisitMutWith, VisitWith};

type LocalExports = HashMap<JsWord, ExportedSpecifier>;

pub fn scan(
  ast: &mut ast::Module,
  top_level_ctxt: SyntaxContext,
  unresolved_ctxt: SyntaxContext,
  module_id: ModuleId,
) -> ScanResult {
  let mut scanner = Scanner::new(top_level_ctxt, unresolved_ctxt, module_id);
  ast.visit_mut_with(&mut scanner);
  scanner.result
}

#[derive(Debug, Default)]
pub struct ScanResult {
  pub dependencies: LinkedHashSet<JsWord>,
  pub dyn_dependencies: HashSet<JsWord>,
  // pub imported_ids: HashMap<JsWord, HashSet<SpecifierInfo>>,
  // Representations of special cases
  // `export * from './src'        => alias: ("*", SyntaxContext) origin: "*"
  // `export * as foo from './src' => alias: ("foo", SyntaxContext) origin: "*"
  pub re_exported_ids: HashMap<JsWord, HashSet<ReExportedSpecifier>>,

  pub re_export_all: LinkedHashSet<JsWord>,

  // Representations of special cases
  // `export default 'hello'` : "default" => ("default", SyntaxContext(0)). We need generate a id for it.
  pub local_exports: LocalExports,
  // Record exported id to check if there are duplicated exports
  pub declared_scoped_names: HashSet<JsWord>,
  pub visited_global_names: HashSet<JsWord>,
  pub statement_parts: Vec<StatementPart>,
  pub imports: FxHashMap<JsWord, FxHashSet<ImportedSpecifier>>,
  pub suggested_names: FxHashMap<JsWord, JsWord>,
}

/// Notices
/// 1. Though,the pass is named scan, we will change some AST nodes in this pass.
struct Scanner {
  module_id: ModuleId,
  result: ScanResult,
  // Record exported id to check if there are duplicated exports
  exported_names: HashSet<JsWord>,
  unresolved_ctxt: SyntaxContext,
  top_level_ctxt: SyntaxContext,
  statement_part: StatementPart,
  imported_namespaces: HashMap<Symbol, NamespaceInfo>,
}

struct NamespaceInfo {
  source: JsWord,
  used_member_name: HashMap<JsWord, Symbol>,
  is_used_dynamically: bool,
}

impl Scanner {
  pub fn new(
    top_level_ctxt: SyntaxContext,
    unresolved_ctxt: SyntaxContext,
    module_id: ModuleId,
  ) -> Self {
    Self {
      module_id,
      result: Default::default(),
      // top_level_mark,
      // unresolved_mark,
      unresolved_ctxt,
      top_level_ctxt,
      exported_names: Default::default(),
      statement_part: Default::default(),
      imported_namespaces: Default::default(),
    }
  }

  fn add_declared_id(&mut self, id: Symbol) {
    self.statement_part.declared.insert(id);
  }

  fn add_declared_scope_id(&mut self, id: Symbol) {
    self.result.declared_scoped_names.insert(id.name().clone());
  }

  fn add_dependency(&mut self, specifier: JsWord) {
    self.result.dependencies.get_or_insert(specifier);
  }

  fn check_is_already_exported(&mut self, exported_name: &JsWord) {
    if self.exported_names.contains(exported_name) {
      panic!("SyntaxError: Duplicate export of '{:}'", exported_name)
    } else {
      self.exported_names.insert(exported_name.clone());
    }
  }

  fn add_local_export(&mut self, exported_as: JsWord, exported: Symbol) {
    self.check_is_already_exported(&exported_as);
    self.result.local_exports.insert(
      exported_as.clone(),
      ExportedSpecifier {
        exported_as,
        local_id: exported,
        owner: self.module_id.clone(),
      },
    );
  }

  fn add_re_export_all(&mut self, src: JsWord) {
    self.result.re_export_all.get_or_insert(src);
  }

  fn add_re_export(&mut self, module_id: JsWord, spec_id: ReExportedSpecifier) {
    let exported_name = &spec_id.exported_as;
    self.check_is_already_exported(exported_name);

    self
      .result
      .re_exported_ids
      .entry(module_id)
      .or_default()
      .insert(spec_id);
  }

  fn add_dynamic_import(&mut self, node: &CallExpr) {
    if let Callee::Import(_) = node.callee {
      if let Some(dyn_imported) = node.args.get(0) {
        if dyn_imported.spread.is_none() {
          if let Expr::Lit(Lit::Str(imported)) = dyn_imported.expr.as_ref() {
            self.result.dyn_dependencies.insert(imported.value.clone());
          }
        }
      }
    }
  }

  fn add_imported_specifier(
    &mut self,
    local_module_id: JsWord,
    imported_as: Symbol,
    imported: JsWord,
  ) {
    let specifier = ImportedSpecifier {
      imported_as,
      imported,
    };

    self
      .result
      .imports
      .entry(local_module_id)
      .or_insert_with(Default::default)
      .insert(specifier);
  }

  fn scan_import(&mut self, module_decl: &ModuleDecl) {
    if let ModuleDecl::Import(import_decl) = module_decl {
      let local_module_id = import_decl.src.value.clone();
      self.add_dependency(local_module_id.clone());
      import_decl.specifiers.iter().for_each(|specifier| {
        let (imported_name, imported_as) = match specifier {
          ast::ImportSpecifier::Named(s) => {
            let imported_name = s.imported_name().clone();
            let imported_as = s.imported_as_ident().to_id();
            (imported_name, imported_as)
          }
          ast::ImportSpecifier::Default(s) => (js_word!("default"), s.local.to_id()),
          ast::ImportSpecifier::Namespace(s) => {
            debug_assert!(!self
              .imported_namespaces
              .contains_key(&s.local.to_id().into()));
            self.imported_namespaces.insert(
              s.local.to_id().into(),
              NamespaceInfo {
                source: local_module_id.clone(),
                used_member_name: Default::default(),
                is_used_dynamically: false,
              },
            );
            (js_word!("*"), s.local.to_id())
          }
        };
        self.add_imported_specifier(local_module_id.clone(), imported_as.into(), imported_name);
      });
    }
  }

  fn scan_export(&mut self, module_decl: &ModuleDecl) {
    match module_decl {
      ModuleDecl::ExportNamed(node) => {
        let dep_id = node.src.as_ref().map(|s| s.value.clone());

        if let Some(source) = &dep_id {
          self.add_dependency(source.clone());

          node.specifiers.iter().for_each(|specifier| {
            match specifier {
              ExportSpecifier::Named(s) => {
                self.add_re_export(
                  source.clone(),
                  ReExportedSpecifier {
                    exported_as: s.exported_as_name().clone(),
                    imported: s.local_ident().sym.clone(),
                  },
                );
              }
              ExportSpecifier::Namespace(s) => {
                // export * as name from './other'
                self.add_re_export(
                  source.clone(),
                  ReExportedSpecifier {
                    exported_as: s.name.expect_ident().sym.clone(),
                    imported: js_word!("*"),
                  },
                )
              }
              ExportSpecifier::Default(_) => {
                // export v from 'mod';
                // Rollup doesn't support it.
                unreachable!("`export xxx from 'xxx'` is not supported")
              }
            };
          });
        } else {
          node.specifiers.iter().for_each(|specifier| {
            if let ExportSpecifier::Named(s) = specifier {
              // export { name }
              self.add_local_export(s.exported_as_name().clone(), s.local_ident().to_id().into());
            };
          });
        }
      }
      ModuleDecl::ExportDecl(decl) => match &decl.decl {
        ast::Decl::Class(decl) => {
          self.add_local_export(decl.ident.sym.clone(), decl.ident.to_id().into());
        }
        ast::Decl::Fn(decl) => {
          self.add_local_export(decl.ident.sym.clone(), decl.ident.to_id().into());
        }
        ast::Decl::Var(decl) => {
          let mut collected = vec![] as Vec<Ident>;
          let mut collector = VarCollector { to: &mut collected };
          decl.visit_with(&mut collector);

          collected
            .clone()
            .into_iter()
            .map(|i| i.to_id())
            .map(|id| (id.0.clone(), id))
            .for_each(|(name, id)| self.add_local_export(name, id.into()));
        }
        ast::Decl::TsInterface(_) => unreachable!(),
        ast::Decl::TsTypeAlias(_) => unreachable!(),
        ast::Decl::TsEnum(_) => unreachable!(),
        ast::Decl::TsModule(_) => unreachable!(),
      },
      ModuleDecl::ExportDefaultDecl(node) => match &node.decl {
        // We will make sure that the default export always has a name.
        ast::DefaultDecl::Class(node) => {
          self.add_local_export(
            "default".into(),
            node.ident.clone().map(|i| i.to_id()).unwrap().into(),
          );
        }
        ast::DefaultDecl::Fn(node) => {
          self.add_local_export(
            "default".into(),
            node.ident.clone().map(|i| i.to_id()).unwrap().into(),
          );
        }
        ast::DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
      },
      ModuleDecl::ExportDefaultExpr(node) => match node.expr.as_ref() {
        Expr::Ident(ident) => {
          self.add_local_export("default".into(), ident.to_id().into());
        }
        _ => {
          unreachable!()
        }
      },
      ModuleDecl::ExportAll(node) => {
        // export * from './other'
        let source = node.src.value.clone();
        self.add_re_export_all(source);

        self.add_dependency(node.src.value.clone());
      }
      _ => {}
    }
  }

  fn collect_declared_id_of_top_level(&mut self, module_item: &ModuleItem) {
    match module_item {
      ModuleItem::Stmt(Stmt::Decl(decl)) => match decl {
        ast::Decl::Class(decl) => self.add_declared_id(decl.ident.to_id().into()),
        ast::Decl::Fn(_decl) => {
          // same as following
        }
        ast::Decl::Var(_decl) => {
          // We are not going collected declared ids here
          // Because we might missing some top level variable declarations
          // which is in the block statement.
        }
        ast::Decl::TsInterface(_) => unreachable!(),
        ast::Decl::TsTypeAlias(_) => unreachable!(),
        ast::Decl::TsEnum(_) => unreachable!(),
        ast::Decl::TsModule(_) => unreachable!(),
      },
      ModuleItem::ModuleDecl(module_decl) => match module_decl {
        ModuleDecl::ExportDecl(decl) => match &decl.decl {
          ast::Decl::Class(decl) => {
            self.add_declared_id(decl.ident.to_id().into());
          }
          ast::Decl::Fn(decl) => {
            self.add_declared_id(decl.ident.to_id().into());
          }
          ast::Decl::Var(decl) => {
            let mut collected = vec![] as Vec<Ident>;
            let mut collector = VarCollector { to: &mut collected };
            decl.visit_with(&mut collector);
            collected
              .into_iter()
              .map(|i| i.to_id())
              .for_each(|id| self.add_declared_id(id.into()));
          }
          ast::Decl::TsInterface(_) => todo!(),
          ast::Decl::TsTypeAlias(_) => todo!(),
          ast::Decl::TsEnum(_) => todo!(),
          ast::Decl::TsModule(_) => todo!(),
        },
        ModuleDecl::ExportDefaultDecl(node) => match &node.decl {
          ast::DefaultDecl::Class(cls) => {
            if let Some(ident) = cls.ident.as_ref() {
              self.add_declared_id(ident.to_id().into())
            }
          }
          ast::DefaultDecl::Fn(func) => {
            if let Some(ident) = func.ident.as_ref() {
              self.add_declared_id(ident.to_id().into())
            } else {
              self.add_declared_id(self.facade_default_symbol())
            }
          }
          ast::DefaultDecl::TsInterfaceDecl(_) => todo!(),
        },
        _ => {}
      },
      _ => {}
    }
  }

  fn add_imported_namespaces_to_imports(&mut self) {
    std::mem::take(&mut self.imported_namespaces)
      .into_iter()
      .for_each(|(imported_namespace_id, info)| {
        if info.is_used_dynamically {
          self.add_imported_specifier(info.source.clone(), imported_namespace_id, "*".into());
        }
        info
          .used_member_name
          .into_iter()
          .for_each(|(name, member_id)| {
            self.add_imported_specifier(info.source.clone(), member_id, name);
          });
      });
  }

  /// Notice that we need to look ahead before `refer_variable()`
  /// Otherwise, the namespace will be marked as dynamic.
  fn rewrite_imported_namespace_visit(&mut self, expr: &mut ast::Expr) {
    match expr {
              ast::Expr::Member(ast::MemberExpr { obj: box ast::Expr::Ident(object), prop, .. }) if let Some(namespace_info) = self.imported_namespaces.get_mut(&object.to_id().into()) => {
                  match &prop {
                      ast::MemberProp::Ident(ast::Ident { sym: prop_sym, .. }) | ast::MemberProp::Computed(ast::ComputedPropName { expr: box ast::Expr::Lit(ast::Lit::Str( ast::Str { value: prop_sym, .. } )), .. }) => {
                          let prop_id = namespace_info
                              .used_member_name
                              .entry(prop_sym.clone())
                              .or_insert_with_key(|prop_name| {
                                  // Since we are using the same top_level_ctxt,
                                  // Add a invalid name to avoid conflicted with local name
                                  // They will be renamed valid in finalize phase.
                                  let ident_name: JsWord = format!("{}#{}", object.sym, prop_name).into();
                                  self.result.suggested_names.insert(ident_name.clone(), prop_sym.clone());
                                  (ident_name, self.top_level_ctxt).into()
                              });
                          *expr = Expr::Ident(prop_id.clone().to_id().into());
                      }
                      ast::MemberProp::Computed(_) => {
                          namespace_info.is_used_dynamically = true;
                      }
                      ast::MemberProp::PrivateName(_) => {
                          unreachable!("Private field must be declared in an enclosing class")
                      }
                  }
              },
              _ => {}
          }
  }

  fn refer_variable(&mut self, variable: &Ident) {
    let var_id: Symbol = variable.to_id().into();
    if let Some(info) = self.imported_namespaces.get_mut(&var_id) {
      // TODO: currently we will mark useless visit of namespace as dynamic
      info.is_used_dynamically = true;
    }
    if variable.span.ctxt == self.unresolved_ctxt {
      self
        .result
        .visited_global_names
        .insert(variable.sym.clone());
    } else if variable.span.ctxt == self.top_level_ctxt {
      self.statement_part.referenced.insert(var_id);
    }
  }

  fn facade_default_symbol(&self) -> Symbol {
    (js_word!("default"), self.top_level_ctxt).into()
  }

  /// This method turn `export default 'hello, world'`
  /// to `var default = 'hello, world'; export default default;`
  /// We use `default` as a facade name to avoid conflict with local variable.
  /// In the finalize/deconflict phase, we will rename it to a valid name.
  ///
  /// Notices
  ///
  /// - `var` is used instead of `const`/`let` because the hoisting behavior of `export default`.
  fn name_anonymous_default_export(&self, node: &mut Vec<ModuleItem>) {
    let mut is_need_generate_export_default_ident = false;
    for module_item in node.iter_mut() {
      match module_item {
        ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDefaultExpr(expr)) => {
          is_need_generate_export_default_ident = true;
          *module_item =
            ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Var(Box::new(ast::VarDecl {
              span: Default::default(),
              declare: false,
              kind: ast::VarDeclKind::Var,
              decls: vec![ast::VarDeclarator {
                span: Default::default(),
                name: self.facade_default_symbol().to_id().into(),
                init: Some(expr.expr.take()),
                definite: false,
              }],
            }))));
          break;
        }
        ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDefaultDecl(decl)) => match &mut decl.decl {
          ast::DefaultDecl::Class(cls) => {
            if cls.ident.is_none() {
              cls.ident = Some(self.facade_default_symbol().to_id().into());
            }
            break;
          }
          ast::DefaultDecl::Fn(func) => {
            if func.ident.is_none() {
              func.ident = Some(self.facade_default_symbol().to_id().into());
            }
            break;
          }
          ast::DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
        },
        _ => {}
      };
    }
    if is_need_generate_export_default_ident {
      node.push(ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDefaultExpr(
        ast::ExportDefaultExpr {
          span: Default::default(),
          expr: Box::new(ast::Expr::Ident(
            self.facade_default_symbol().to_id().into(),
          )),
        },
      )))
    }
  }

  fn remove_unused_imported_specifier(&mut self) {
    let referenced = self
      .result
      .statement_parts
      .iter()
      .flat_map(|part| part.referenced.iter())
      .collect::<FxHashSet<_>>();

    self
      .result
      .imports
      .values_mut()
      .for_each(|specs| specs.retain(|spec| referenced.contains(&spec.imported_as)));
  }

  fn scan_ident(&mut self, ident: &Ident) {
    debug_assert!(
      ident.span.ctxt != SyntaxContext::empty(),
      "ident should have a context"
    );
    // TODO: Is the assumption correct?
    debug_assert!(
      ident.span.ctxt != SyntaxContext::empty(),
      "ident should have a context"
    );
    if ident.span.ctxt == self.top_level_ctxt {
      self.add_declared_id(ident.to_id().into());
    } else {
      self.add_declared_scope_id(ident.to_id().into());
    }
  }
}

impl VisitMut for Scanner {
  noop_visit_mut_type!();

  fn visit_mut_module_items(&mut self, node: &mut Vec<ModuleItem>) {
    self.name_anonymous_default_export(node);

    self.result.statement_parts = Vec::with_capacity(node.len());

    // We need scan import and export first.
    node
      .iter()
      .filter_map(|module_item| module_item.as_module_decl())
      .for_each(|module_decl| {
        self.scan_import(module_decl);
        self.scan_export(module_decl);
      });

    node.visit_mut_children_with(self);

    self.add_imported_namespaces_to_imports();
    self.remove_unused_imported_specifier();
  }

  fn visit_mut_module_item(&mut self, node: &mut ModuleItem) {
    self.statement_part.side_effect = match node {
      ModuleItem::ModuleDecl(_) => false,
      ModuleItem::Stmt(stmt) => stmt.may_have_side_effect(&ExprCtx {
        unresolved_ctxt: self.unresolved_ctxt,
        is_unresolved_ref_safe: false,
      }),
    };
    self.collect_declared_id_of_top_level(node);
    node.visit_mut_children_with(self);
    self
      .result
      .statement_parts
      .push(std::mem::take(&mut self.statement_part));
  }

  fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
    self.add_dynamic_import(node);
    node.visit_mut_children_with(self);
  }

  #[allow(clippy::single_match)]
  fn visit_mut_expr(&mut self, node: &mut Expr) {
    self.rewrite_imported_namespace_visit(node);
    match node {
      Expr::Ident(ident) => {
        self.refer_variable(ident);
      }
      _ => {}
    }
    node.visit_mut_children_with(self);
  }

  fn visit_mut_export_named_specifier(&mut self, node: &mut ast::ExportNamedSpecifier) {
    if let ast::ModuleExportName::Ident(local_id) = &node.orig {
      self.refer_variable(local_id);
    }
    node.visit_mut_children_with(self);
  }

  fn visit_mut_param(&mut self, node: &mut ast::Param) {
    let mut var_decl_collect = ParamsCollector::default();
    node.visit_children_with(&mut var_decl_collect);
    var_decl_collect
      .collected
      .into_values()
      .for_each(|id| self.scan_ident(&id.into()));

    node.visit_mut_children_with(self);
  }

  fn visit_mut_fn_expr(&mut self, node: &mut ast::FnExpr) {
    if let Some(ident) = &node.ident {
      self.scan_ident(ident)
    }
    node.visit_mut_children_with(self);
  }

  fn visit_mut_fn_decl(&mut self, node: &mut ast::FnDecl) {
    self.scan_ident(&node.ident);
    node.visit_mut_children_with(self);
  }

  fn visit_mut_var_decl(&mut self, node: &mut ast::VarDecl) {
    let mut collected = vec![] as Vec<Ident>;
    let mut collector = VarCollector { to: &mut collected };
    node.visit_with(&mut collector);
    collected
      .into_iter()
      .for_each(|ident| self.scan_ident(&ident));

    node.visit_mut_children_with(self);
  }
}

/// The concept of `StatementPart` is borrowed from Esbuild.
/// A `StatementPart` describe information which is helpful to do treeshake about a statement.
#[derive(Default, Debug)]
pub struct StatementPart {
  pub declared: HashSet<Symbol>,
  pub referenced: HashSet<Symbol>,
  // We could assume that every part has side effects.
  pub is_included: AtomicBool,
  pub side_effect: bool,
}

#[derive(Default, Debug)]
struct ParamsCollector {
  pub collected: HashMap<JsWord, Id>,
}

impl Visit for ParamsCollector {
  fn visit_binding_ident(&mut self, n: &ast::BindingIdent) {
    let id = n.id.to_id();
    self.collected.insert(id.0.clone(), id);
  }

  fn visit_assign_pat_prop(&mut self, n: &ast::AssignPatProp) {
    let id = n.key.to_id();
    self.collected.insert(id.0.clone(), id);
  }
}

trait StmtExt {
  fn may_have_side_effect(&self, ctx: &ExprCtx) -> bool;
}

impl StmtExt for ast::Stmt {
  fn may_have_side_effect(&self, ctx: &ExprCtx) -> bool {
    match self {
      Stmt::Block(stmt) => stmt.stmts.iter().any(|stmt| stmt.may_have_side_effect(ctx)),
      Stmt::Empty(_) | Stmt::Return(_) | Stmt::Labeled(_) | Stmt::Break(_) | Stmt::Continue(_) => {
        false
      }
      Stmt::Debugger(_) => true,

      Stmt::If(stmt) => {
        stmt.test.may_have_side_effects(ctx)
          || stmt.cons.may_have_side_effect(ctx)
          || stmt
            .alt
            .as_ref()
            .map(|alt| alt.may_have_side_effect(ctx))
            .unwrap_or(false)
      }
      Stmt::Switch(stmt) => {
        stmt.discriminant.may_have_side_effects(ctx)
          || stmt.cases.iter().any(|case| {
            case
              .test
              .as_ref()
              .map(|test| test.may_have_side_effects(ctx))
              .unwrap_or(false)
              || case.cons.iter().any(|stmt| stmt.may_have_side_effect(ctx))
          })
      }
      Stmt::Throw(stmt) => stmt.arg.may_have_side_effects(ctx),
      Stmt::While(stmt) => {
        stmt.test.may_have_side_effects(ctx) || stmt.body.may_have_side_effect(ctx)
      }
      Stmt::DoWhile(stmt) => {
        stmt.test.may_have_side_effects(ctx) || stmt.body.may_have_side_effect(ctx)
      }
      Stmt::Decl(stmt) => match stmt {
        ast::Decl::Class(decl) => class_has_side_effect(ctx, &decl.class),
        ast::Decl::Fn(_) => false,
        ast::Decl::Var(decl) => decl.decls.iter().any(|decl| {
          decl
            .init
            .as_ref()
            .map(|init| init.may_have_side_effects(ctx))
            .unwrap_or(false)
        }),
        ast::Decl::TsInterface(_) => false,
        ast::Decl::TsTypeAlias(_) => false,
        ast::Decl::TsEnum(_) => false,
        ast::Decl::TsModule(_) => false,
      },
      Stmt::Expr(stmt) => stmt.expr.may_have_side_effects(ctx),
      // Not decided yet.
      Stmt::With(_) => true,
      Stmt::Try(_) => true,
      Stmt::For(_) => true,
      Stmt::ForIn(_) => true,
      Stmt::ForOf(_) => true,
    }
  }
}
