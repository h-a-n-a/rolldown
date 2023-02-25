use ast::{ExportNamedSpecifier, Id, Ident, PropName};
use rolldown_common::{ChunkId, ModuleId};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_common::{util::take::Take, SyntaxContext, DUMMY_SP};
use swc_core::{
  common::{self as swc_common, Span},
  ecma::{
    ast::{self, BindingIdent, Stmt},
    atoms::{js_word, JsWord},
    utils::{self as swc_ecma_utils, quote_str},
    visit as swc_ecma_visit,
  },
};
use swc_ecma_utils::quote_ident;
use swc_ecma_visit::{VisitMut, VisitMutWith};

enum IdentType {
  TopLevel,
  Scoped,
  Unresolved,
  Dummy,
}

#[allow(unused)]
impl IdentType {
  pub(crate) fn is_top_level(&self) -> bool {
    matches!(self, IdentType::TopLevel)
  }

  pub(crate) fn is_scoped(&self) -> bool {
    matches!(self, IdentType::Scoped)
  }

  pub(crate) fn is_unresolved(&self) -> bool {
    matches!(self, IdentType::Unresolved)
  }

  pub(crate) fn is_dummy(&self) -> bool {
    matches!(self, IdentType::Dummy)
  }
}

#[derive(Debug, Clone)]
pub struct FinalizeContext<'me> {
  /// Used to rewrite dynamic import
  pub resolved_ids: &'me HashMap<JsWord, ModuleId>,
  /// All declared scoped names in this chunk
  pub declared_scoped_names: &'me HashSet<JsWord>,
  pub unresolved_ctxt: SyntaxContext,
  /// Used to rewrite dynamic import
  pub chunk_filename_by_id: &'me HashMap<ChunkId, String>,
  // All top_level_ctxt of modules belong to this chunk
  pub top_level_ctxt_set: &'me HashSet<SyntaxContext>,
}

pub fn finalizer<'a>(
  rename_map: &'a HashMap<Id, JsWord>,
  split_point_id_to_chunk_id: &'a HashMap<ModuleId, ChunkId>,
  ctx: FinalizeContext<'a>,
) -> impl VisitMut + 'a {
  Finalizer::new(rename_map, split_point_id_to_chunk_id, ctx)
}

#[derive(Debug)]
struct Finalizer<'a> {
  pub ctx: FinalizeContext<'a>,
  used_scoped_names: HashSet<JsWord>,
  renamed_scoped_ids: HashMap<Id, JsWord>,
  renamed_top_level_ids: &'a HashMap<Id, JsWord>,
  top_level_names: HashSet<JsWord>,
  // fix test case 'consistent-renaming-f'
  split_point_id_to_chunk_id: &'a HashMap<ModuleId, ChunkId>,
}

impl<'a> Finalizer<'a> {
  pub fn new(
    rename_map: &'a HashMap<Id, JsWord>,
    split_point_id_to_chunk_id: &'a HashMap<ModuleId, ChunkId>,
    ctx: FinalizeContext<'a>,
  ) -> Self {
    Self {
      renamed_top_level_ids: rename_map,
      top_level_names: rename_map.values().cloned().collect(),
      renamed_scoped_ids: Default::default(),
      split_point_id_to_chunk_id,
      ctx,
      used_scoped_names: Default::default(),
    }
  }
  fn rename_top_level_ident(&mut self, ident: &mut Ident) -> Option<()> {
    debug_assert!(self.ident_type(ident).is_top_level());
    let name = self.renamed_top_level_ids.get(&ident.to_id())?;
    // Renamed ident should be dummy.
    *ident = quote_ident!(name.clone());
    Some(())
  }

  fn rename_scoped_ident(&mut self, ident: &mut Ident) -> Option<()> {
    debug_assert!(self.ident_type(ident).is_scoped());
    let name = self.renamed_scoped_ids.get(&ident.to_id())?;
    // Renamed ident should be dummy.
    *ident = quote_ident!(name.clone());
    Some(())
  }

  fn ident_type(&self, ident: &Ident) -> IdentType {
    if self.ctx.top_level_ctxt_set.contains(&ident.span.ctxt) {
      IdentType::TopLevel
    } else if ident.span.ctxt == self.ctx.unresolved_ctxt {
      IdentType::Unresolved
    } else if ident.span.ctxt == SyntaxContext::empty() {
      IdentType::Dummy
    } else {
      IdentType::Scoped
    }
  }

  fn should_rename_the_ident(&self, ident: &Ident) -> bool {
    let id = ident.to_id();
    match self.ident_type(ident) {
      // If the final name is different from the original name, we should rename it
      IdentType::TopLevel => self
        .renamed_top_level_ids
        .get(&id)
        .map_or(false, |final_name| final_name != &id.0),

      IdentType::Scoped => {
        // If a scoped ident is conflict with top level name, the scoped ident will be renamed.
        // So, we don't need to check the final name of scoped ident whether is different from the original name.
        self.top_level_names.contains(&id.0)
      }
      _ => false,
    }
  }

  fn generate_conflictless_scoped_name(&mut self, ident: &Ident) {
    debug_assert!(self.ident_type(ident).is_scoped());

    let id = ident.to_id();

    if self.top_level_names.contains(&id.0) && !self.renamed_scoped_ids.contains_key(&id) {
      let mut count = 1;
      let mut new_name: JsWord = format!("{}${}", &id.0, count).into();
      while self.ctx.declared_scoped_names.contains(&new_name)
        || self.used_scoped_names.contains(&new_name)
        || self.top_level_names.contains(&new_name)
      {
        count += 1;
        new_name = format!("{}${}", &id.0, count).into();
      }
      self.used_scoped_names.insert(new_name.clone());
      self.renamed_scoped_ids.insert(id, new_name);
    }
  }

  fn rewrite_dynamic_import(&mut self, node: &mut ast::CallExpr) -> Option<()> {
    if node.callee.is_import() {
      let first_arg = node.args.get_mut(0)?.expr.as_mut_lit()?;
      if let ast::Lit::Str(ast::Str {
        value: local_module_id,
        raw,
        ..
      }) = first_arg
      {
        *raw = None;
        let module_id = self.resolve_module_id(local_module_id)?;
        let chunk_id = self.split_point_id_to_chunk_id.get(module_id)?;
        let filename = self.ctx.chunk_filename_by_id.get(chunk_id)?;
        *local_module_id = format!("./{}", filename.clone()).into();
      };
    }

    Some(())
  }

  fn resolve_module_id(&self, local_module_id: &JsWord) -> Option<&ModuleId> {
    let resolved_id = self.ctx.resolved_ids.get(local_module_id)?;
    Some(resolved_id)
  }

  fn make_exported_specifier_shorter(&mut self, node: &mut ast::ExportNamedSpecifier) {
    if let ExportNamedSpecifier {
            exported: Some(ast::ModuleExportName::Ident(ast::Ident { sym: exported_name, .. })),
            orig: ast::ModuleExportName::Ident(ast::Ident { sym: orig_name, .. }),
            ..
        } = node && exported_name == orig_name
        { node.exported = None }
  }

  /// Turn `obj = ({ a })` to `obj = ({ a: a })`
  /// After expanding, if the property `a` is renamed to `b`,
  /// the output would be `{ a: b }`. The expr `obj.a` is still valid.
  fn expand_shorthand(&self, prop: &mut ast::Prop) -> bool {
    if let ast::Prop::Shorthand(ident) = prop {
      *prop = ast::Prop::KeyValue(ast::KeyValueProp {
        key: quote_ident!(ident.sym.clone()).into(),
        value: Box::new(ast::Expr::Ident(ident.take())),
      });
      true
    } else {
      false
    }
  }

  fn undo_expand_shorthand(&self, prop: &mut ast::Prop, shorthanded: bool) {
    if shorthanded && let ast::Prop::KeyValue(ast::KeyValueProp { key: ast::PropName::Ident(key), value: box ast::Expr::Ident(value) }) = prop && key.sym == value.sym { *prop = ast::Prop::Shorthand(value.take()) }
  }

  /// If the property `foo` is renamed `foo2`, we need to keep the semantics of ObjectPatProp.
  /// turn `const { foo } = { foo }`
  /// into `const { foo: foo2 } = { foo }`
  /// instead `const { foo2 } = { foo }`
  fn keep_semantics_of_object_pat_prop(&self, node: &mut ast::ObjectPatProp) -> Option<Span> {
    if let ast::ObjectPatProp::Assign(prop) = node {
      let prop_span = prop.span;
      *node = ast::ObjectPatProp::KeyValue(ast::KeyValuePatProp {
        key: PropName::Ident(quote_ident!(prop.key.sym.clone())),
        value: prop
          .value
          .take()
          .map(|value| {
            // handle case `const { foo = 1 } = { foo }`
            Box::new(ast::Pat::Assign(ast::AssignPat {
              span: DUMMY_SP,
              left: Box::new(ast::Pat::Ident(ast::BindingIdent {
                id: prop.key.take(),
                type_ann: None,
              })),
              right: value,
              type_ann: None,
            }))
          })
          .unwrap_or_else(|| {
            Box::new(ast::Pat::Ident(ast::BindingIdent {
              id: prop.key.take(),
              type_ann: None,
            }))
          }),
      });
      Some(prop_span)
    } else {
      None
    }
  }

  fn undo_keep_semantics_of_object_pat_prop(
    &self,
    node: &mut ast::ObjectPatProp,
    changed: Option<Span>,
  ) {
    if let Some(assign_prop_span) = changed {
      if let ast::ObjectPatProp::KeyValue(ast::KeyValuePatProp {
        key: ast::PropName::Ident(ref key),
        value,
      }) = node
      {
        match value.as_mut() {
          ast::Pat::Ident(ast::BindingIdent { id: value, .. }) if key.sym == value.sym => {
            *node = ast::ObjectPatProp::Assign(ast::AssignPatProp {
              span: assign_prop_span,
              key: value.take(),
              value: None,
            })
          }
          ast::Pat::Assign(ast::AssignPat {
            left: box ast::Pat::Ident(ast::BindingIdent { id, .. }),
            right,
            ..
          }) if key.sym == id.sym => {
            *node = ast::ObjectPatProp::Assign(ast::AssignPatProp {
              span: assign_prop_span,
              key: id.take(),
              value: Some(right.take()),
            })
          }
          _ => {}
        }
      }
    }
  }

  /// https://github.com/rollup/rollup/pull/4674
  fn keep_class_name_if_needed(&mut self, stmt: &mut Stmt) {
    /// Inject `static name = "..."`
    ///
    /// TODO: We might need to inject name using the esbuild's way.
    /// Using static field require transpile while targeting es2015.
    /// https://github.com/evanw/esbuild/issues/510
    fn inject_name(class: &mut ast::Class, name: JsWord) {
      class.body.insert(
        0,
        ast::ClassMember::ClassProp(ast::ClassProp {
          key: PropName::Ident(quote_ident!("name")),
          value: Some(box ast::Expr::Lit(quote_str!(name).into())),
          span: DUMMY_SP,
          is_abstract: false,
          is_optional: false,
          is_override: false,
          readonly: false,
          declare: false,
          definite: false,
          type_ann: None,
          is_static: true,
          decorators: vec![],
          accessibility: None,
        }),
      )
    }
    match stmt {
      ast::Stmt::Decl(ast::Decl::Class(ast::ClassDecl { ident, class, .. }))
      // "default" is a special case, it's not a valid identifier
      // The "default" ident is generated by scanner, to give a default export anonymous class expression a name
        if self.should_rename_the_ident(ident) && ident.sym != js_word!("default") =>
      {
        let original_name = ident.sym.clone();
        inject_name(class, original_name);
      }
      ast::Stmt::Decl(ast::Decl::Var(box ast::VarDecl { decls, .. })) => {
        match decls.as_mut_slice() {
          [ast::VarDeclarator {
            name: ast::Pat::Ident(BindingIdent { id: decl_ident, .. }),
            init:
              Some(box ast::Expr::Class(ast::ClassExpr {
                ident: class_expr_ident,
                class,
                ..
              })),
            ..
          }] if class_expr_ident.is_none() && self.should_rename_the_ident(decl_ident) && decl_ident.sym != js_word!("default") => {
            let original_name = decl_ident.sym.clone();
            inject_name(class, original_name);
          }
          _ => {}
        }
      }
      _ => {}
    }
  }
}

impl<'a> VisitMut for Finalizer<'a> {
  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    match self.ident_type(ident) {
      IdentType::TopLevel => {
        self.rename_top_level_ident(ident);
      }
      IdentType::Scoped => {
        self.generate_conflictless_scoped_name(ident);
        self.rename_scoped_ident(ident);
      }
      IdentType::Dummy => {
        // tracing::trace!("Bailout finalize dummy ident: {:?}", ident.to_id());
      }
      IdentType::Unresolved => {
        // tracing::trace!("Bailout finalize for unresolved ident: {:?}", ident.to_id());
      }
    }
  }

  fn visit_mut_stmt(&mut self, stmt: &mut ast::Stmt) {
    self.keep_class_name_if_needed(stmt);
    stmt.visit_mut_children_with(self);
  }

  fn visit_mut_prop(&mut self, prop: &mut ast::Prop) {
    let shorthanded = self.expand_shorthand(prop);
    prop.visit_mut_children_with(self);
    self.undo_expand_shorthand(prop, shorthanded);
  }

  fn visit_mut_object_pat_prop(&mut self, node: &mut ast::ObjectPatProp) {
    let changed = self.keep_semantics_of_object_pat_prop(node);
    node.visit_mut_children_with(self);
    self.undo_keep_semantics_of_object_pat_prop(node, changed);
  }

  fn visit_mut_export_named_specifier(&mut self, node: &mut ExportNamedSpecifier) {
    node.visit_mut_children_with(self);
    self.make_exported_specifier_shorter(node)
  }

  fn visit_mut_import_named_specifier(&mut self, node: &mut ast::ImportNamedSpecifier) {
    node.visit_mut_children_with(self);
    if let Some(ast::ModuleExportName::Ident(imported)) = &node.imported {
      if imported.sym == node.local.sym {
        node.imported = None;
      }
    }
  }

  fn visit_mut_call_expr(&mut self, node: &mut ast::CallExpr) {
    self.rewrite_dynamic_import(node);
    node.visit_mut_children_with(self);
  }
}
