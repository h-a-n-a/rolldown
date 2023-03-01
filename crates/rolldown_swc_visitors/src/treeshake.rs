use std::sync::Arc;

use ast::{Id, Ident, ModuleItem};
use rustc_hash::FxHashSet as HashSet;
use swc_atoms::js_word;
use swc_common::{util::take::Take, Mark, SourceMap, SyntaxContext, DUMMY_SP};
use swc_core::{
  common::{self as swc_common, comments::Comments},
  ecma::{
    ast, atoms as swc_atoms, minifier as swc_ecma_minifier,
    transforms::base::fixer,
    utils::{self as swc_ecma_utils, member_expr},
    visit as swc_ecma_visit,
  },
};
use swc_ecma_minifier::{
  optimize,
  option::{CompressOptions, ExtraOptions, MinifyOptions, TopLevelOptions},
};
use swc_ecma_utils::{quote_ident, var::VarCollector};
use swc_ecma_visit::{FoldWith, VisitMut, VisitMutWith, VisitWith};
use tracing::instrument;

/// The goal is to do tree shaking on the AST not minimize it.
#[instrument(skip_all, level = "trace")]
pub fn treeshake(
  ast: &mut ast::Module,
  unresolved_mark: Mark,
  unused: &HashSet<Id>,
  top_level_ctxt: SyntaxContext,
  top_level_mark: Mark,
  cm: Arc<SourceMap>,
  comments: &dyn Comments,
) {
  ast.visit_mut_with(&mut UnusedExportRemover::new(unused, top_level_ctxt));

  // *ast = take_program(ast).fold_with(&mut simplifier(unresolved_mark, Default::default()))

  let optimized = optimize(
    ast.take().into(),
    cm,
    Some(comments),
    None,
    &MinifyOptions {
      // Details see https://terser.org/docs/api-reference
      compress: Some(CompressOptions {
        ecma: ast::EsVersion::Es2022,
        // The maximum number of times to run compress. In some cases more than one pass leads to further compressed code. Keep in mind more passes will take more time.
        passes: 3,
        // --- Enabled
        // enable top level variable and function name mangling and to drop unused variables and functions.
        top_level: Some(TopLevelOptions { functions: true }),
        // Pass true to prevent the compressor from discarding function names. Pass a regular expression to only keep function names matching that regex. Useful for code relying on Function.prototype.name. See also: the keep_fnames
        keep_fnames: true,
        //  Prevents the compressor from discarding unused function arguments. You need this for code which relies on Function.length
        keep_fargs: true,
        // various optimizations for boolean context, for example !!a ? b : c → a ? b : c
        bools: false,
        // apply optimizations for if-s and conditional expressions
        conditionals: true,
        //  remove unreachable code
        dead_code: true,
        // attempt to evaluate constant expressions
        evaluate: true,
        // Pass true to not mangle class names.
        keep_classnames: true,
        // Pass true to prevent Infinity from being compressed into 1/0, which may cause performance issues on Chrome.
        keep_infinity: true,
        //  optimizations for do, while and for loops when we can statically determine the condition.
        loops: true,
        // If you pass true for this, Terser will assume that object property access (e.g. foo.bar or foo["bar"]) doesn't have any side effects. Specify "strict" to treat foo.bar as side-effect-free only when foo is certain to not throw, i.e. not null or undefined.
        pure_getters: swc_ecma_minifier::option::PureGetterOption::Strict,
        // Remove expressions which have no side effects and whose results aren't used.
        side_effects: true,
        // de-duplicate and remove unreachable switch branches
        switches: true,
        // drop unreferenced functions and variables (simple direct variable assignments do not count as references unless set to "keep_assign")
        unused: true,

        // --- Disabled
        const_to_let: false,
        // Disable inline
        inline: 0,
        // join consecutive simple statements using the comma operator.
        sequences: 0,
        // replace arguments[index] with function parameter name whenever possible.
        arguments: false,
        //  Class and object literal methods are converted will also be converted to arrow expressions if the resultant code is shorter: m(){return x} becomes m:()=>x.
        arrows: false,
        // Turn booleans into 0 and 1, also makes comparisons with booleans use == and != instead of === and !==.
        bools_as_ints: false,
        // Collapse single-use non-constant variables, side effects permitting.
        collapse_vars: false,
        // apply certain optimizations to binary nodes, e.g. !(a <= b) → a > b (only when unsafe_comps), attempts to negate binary nodes, e.g. a = !b && !c && !d && !e → a=!(b||c||d||e) etc.
        comparisons: false,
        // Transforms constant computed properties into regular ones: {["computed"]: 1} is converted to {computed: 1}.
        computed_props: false,
        // hoist function declarations
        hoist_fns: false,
        // hoist properties from constant object and array literals into regular variables subject to a set of constraints. For example: var o={p:1, q:2}; f(o.p, o.q); is converted to f(1, 2);. Note: hoist_props works best with mangle enabled, the compress option passes set to 2 or higher, and the compress option toplevel enabled
        hoist_props: false,
        //  hoist var declarations (this is false by default because it seems to increase the size of the output in general)
        hoist_vars: false,
        // set to true to support IE8.
        ie8: false,
        // optimizations for if/return and if/continue
        if_return: false,
        // join consecutive var statements
        join_vars: false,
        // negate "Immediately-Called Function Expressions" where the return value is discarded, to avoid the parens that the code generator would insert.
        negate_iife: false,
        // rewrite property access using the dot notation, for example foo["bar"] → foo.bar
        props: false,
        // WARN: The performance of pure_funcs is not good. With huge input, it may take a long time to finish.
        //  You can pass an array of names and Terser will assume that those functions do not produce side effects. DANGER: will not check if the name is redefined in scope. An example case here, for instance var q = Math.floor(a/b). If variable q is not used elsewhere, Terser will drop it, but will still keep the Math.floor(a/b), not knowing what it does. You can pass
        pure_funcs: vec![member_expr!(Default::default(), Object.freeze)],
        // (legacy option, safely ignored for backwards compatibility).
        reduce_fns: false,
        //  prevent specific toplevel functions and variables from unused removal (can be array, comma-separated, RegExp or function. Implies toplevel)
        top_retain: Vec::default(),
        // Transforms typeof foo == "undefined" into foo === void 0. Note: recommend to set this value to false for IE10 and earlier versions due to known issues.
        typeofs: false,
        // Pass true to preserve completion values from terminal statements without return, e.g. in bookmarklets.
        expr: false,
        // Unsafe assumptions
        unsafe_passes: false,
        unsafe_arrows: false,
        unsafe_comps: false,
        unsafe_function: false,
        unsafe_math: false,
        unsafe_symbols: false,
        unsafe_methods: false,
        unsafe_proto: false,
        unsafe_regexp: false,
        unsafe_undefined: false,

        drop_console: false,
        drop_debugger: false,

        // --- Pending
        // If you modified globals, set this to false.
        pristine_globals: false,
        // remove redundant or non-standard directives
        directives: false,
        //  Pass true when compressing an ES6 module. Strict mode is implied and the toplevel option as well.
        module: false,

        // Improve optimization on variables assigned with and used as constant values.
        // To enable this, we need
        // 1. Fix case https://github.com/swc-project/swc/issues/6419
        // 2. Unexpected inlining operation
        // 2.1 reduce_vars option will inline function call, while terser doesn't.
        // This could be a inconsistent behavior in SWC.
        reduce_vars: false,

        // TODO: we might need to use this to do some replacements
        global_defs: Default::default(),
      }),
      ..Default::default()
    },
    &ExtraOptions {
      unresolved_mark,
      top_level_mark,
    },
  )
  .fold_with(&mut fixer::fixer(None));

  *ast = optimized.module().unwrap();
}

struct UnusedExportRemover<'a> {
  used_ids: &'a HashSet<Id>,
  top_level_ctxt: SyntaxContext,
}

impl<'a> UnusedExportRemover<'a> {
  pub fn new(used_ids: &'a HashSet<Id>, top_level_ctxt: SyntaxContext) -> Self {
    Self {
      used_ids,
      top_level_ctxt,
    }
  }

  fn is_id_used(&self, id: &Id) -> bool {
    self.used_ids.contains(id)
  }
}

impl<'a> VisitMut for UnusedExportRemover<'a> {
  fn visit_mut_module_items(&mut self, module_items: &mut Vec<ModuleItem>) {
    module_items.visit_mut_children_with(self);
  }
  fn visit_mut_module_item(&mut self, module_item: &mut ModuleItem) {
    if let ast::ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDefaultExpr(
      ast::ExportDefaultExpr {
        expr: box ast::Expr::Ident(ident),
        ..
      },
    )) = module_item
    {
      // SWC minifier will compress code `var answer2 = 1;export default answer2`
      // to `export default 1`
      // This behavior will break the DeConflict and Finalizer of Rolldown, so we need to
      // transform `var answer2 = 1;export default answer2` to `var answer2 = 1;export { answer2 as default }` to avoid this behavior.
      *module_item = ModuleItem::ModuleDecl(ast::ModuleDecl::ExportNamed(ast::NamedExport {
        span: DUMMY_SP,
        specifiers: vec![ast::ExportSpecifier::Named(ast::ExportNamedSpecifier {
          span: DUMMY_SP,
          orig: ident.clone().into(),
          exported: Some(quote_ident!(js_word!("default")).into()),
          is_type_only: false,
        })],
        src: None,
        type_only: false,
        asserts: None,
      }));
    }

    match module_item {
      ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDecl(decl)) => match &mut decl.decl {
        ast::Decl::Class(node) => {
          if !self.is_id_used(&node.ident.to_id()) {
            *module_item = ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Class(node.clone())));
          }
        }
        ast::Decl::Fn(node) => {
          if !self.is_id_used(&node.ident.to_id()) {
            *module_item = ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Fn(node.clone())));
          }
        }
        ast::Decl::Var(var) => {
          let mut to: Vec<Ident> = Default::default();
          let mut collector = VarCollector { to: &mut to };
          var.visit_with(&mut collector);
          if !to.iter().any(|i| self.is_id_used(&i.to_id())) {
            *module_item = ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Var(var.clone())));
          }
        }
        _ => unreachable!(),
      },
      ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDefaultDecl(default_decl)) => {
        match &mut default_decl.decl {
          ast::DefaultDecl::Class(cls_decl) => {
            if let Some(ident) = &cls_decl.ident {
              if !self.is_id_used(&ident.to_id()) {
                *module_item = ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Class(ast::ClassDecl {
                  ident: ident.clone(),
                  declare: false,
                  class: cls_decl.class.take(),
                })))
              }
            } else if !self.is_id_used(&("default".into(), self.top_level_ctxt)) {
              *module_item = ModuleItem::Stmt(ast::Stmt::Expr(ast::ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(ast::Expr::Paren(ast::ParenExpr {
                  span: DUMMY_SP,
                  expr: Box::new(ast::Expr::Class(ast::ClassExpr {
                    ident: None,
                    class: cls_decl.class.take(),
                  })),
                })),
              }))
            }
          }
          ast::DefaultDecl::Fn(fn_decl) => {
            if let Some(ident) = &fn_decl.ident {
              if !self.is_id_used(&ident.to_id()) {
                *module_item = ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Fn(ast::FnDecl {
                  ident: ident.clone(),
                  declare: false,
                  function: fn_decl.function.take(),
                })));
              }
            } else if !self.is_id_used(&("default".into(), self.top_level_ctxt)) {
              *module_item = ModuleItem::Stmt(ast::Stmt::Expr(ast::ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(ast::Expr::Paren(ast::ParenExpr {
                  span: DUMMY_SP,
                  expr: Box::new(ast::Expr::Fn(ast::FnExpr {
                    ident: None,
                    function: fn_decl.function.take(),
                  })),
                })),
              }))
            }
          }
          ast::DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
        }
      }
      ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDefaultExpr(default_expr)) => {
        // Remove `export default foo`
        let default_ident_id = default_expr
          .expr
          .as_ident()
          .map(|i| i.to_id())
          .unwrap_or_else(|| ("default".into(), self.top_level_ctxt));
        if !self.is_id_used(&default_ident_id) {
          *module_item = ModuleItem::Stmt(ast::Stmt::Expr(ast::ExprStmt {
            span: DUMMY_SP,
            expr: default_expr.expr.take(),
          }))
        }
      }
      ModuleItem::ModuleDecl(ast::ModuleDecl::ExportNamed(export_named)) => {
        if export_named.src.is_none() {
          export_named.specifiers.retain(|specifier| match specifier {
            ast::ExportSpecifier::Named(named) => {
              if let ast::ModuleExportName::Ident(ident) = &named.orig {
                self.is_id_used(&ident.to_id())
              } else {
                true
              }
            }
            ast::ExportSpecifier::Default(_) => unreachable!(),
            ast::ExportSpecifier::Namespace(ns) => {
              if let ast::ModuleExportName::Ident(ident) = &ns.name {
                self.is_id_used(&ident.to_id())
              } else {
                true
              }
            }
          });
        }
      }
      // ModuleItem::ModuleDecl(ModuleDecl::ExportAll(_)) => vec![],
      _ => {}
    }
  }
}
