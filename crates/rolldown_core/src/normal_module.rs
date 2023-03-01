use derivative::Derivative;
use hashlink::LinkedHashSet;
use itertools::Itertools;
use rolldown_common::{
  ExportedSpecifier, ImportedSpecifier, ModuleId, ReExportedSpecifier, Symbol,
};
use rolldown_runtime_helpers::RuntimeHelpers;
use rolldown_swc_visitors::StatementPart;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use sugar_path::{AsPath, SugarPath};
use swc_core::{
  common::{
    comments::{Comment, CommentKind, Comments, SingleThreadedComments},
    Spanned, SyntaxContext,
  },
  ecma::{
    ast::{self, Ident},
    atoms::{js_word, JsWord},
  },
};
use swc_node_comments::SwcComments;
use tracing::instrument;

use crate::{make_legal, InputOptions, MergedExports, RenderContext, ResolvedModuleIds, COMPILER};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NormalModule {
  /// execution order
  pub(crate) exec_order: usize,
  pub(crate) id: ModuleId,
  /// `dependencies` is a set of module ids that this module imported statically.
  /// The order of dependencies infers the order of execution.
  pub(crate) dependencies: Vec<ModuleId>,
  /// crated by `import()`
  /// Notice: It's ok to use `Vec` here instead of `HashSet`, because we won't mutate it after it's created.
  pub(crate) dyn_dependencies: Vec<ModuleId>,
  #[derivative(Debug = "ignore")]
  pub(crate) ast: ast::Module,
  pub(crate) top_level_ctxt: SyntaxContext,
  pub(crate) imports: HashMap<ModuleId, HashSet<ImportedSpecifier>>,
  pub(crate) linked_imports: HashMap<ModuleId, HashSet<ImportedSpecifier>>,
  /// These symbols are created by `create_top_level_symbol` and are not declared in any statement.
  pub(crate) extra_top_level_symbols: HashSet<Symbol>,
  // local_exports only contains local export like export const a = 1;
  // It does not contain info about re-exports.
  pub(crate) local_exports: HashMap<JsWord, ExportedSpecifier>,
  pub(crate) re_exported_ids: HashMap<ModuleId, HashSet<ReExportedSpecifier>>,
  pub(crate) re_export_all: LinkedHashSet<ModuleId>,
  pub(crate) declared_scoped_names: HashSet<JsWord>,

  pub(crate) resolved_module_ids: ResolvedModuleIds,
  pub(crate) is_user_defined_entry: bool,
  pub(crate) suggested_names: HashMap<JsWord, JsWord>,

  pub(crate) linked_exports: MergedExports,

  pub(crate) facade_id_for_namespace: ExportedSpecifier,
  pub(crate) is_facade_namespace_id_referenced: bool,

  // If a id/name is only referenced not declared, it will be added to this set.
  pub(crate) visited_global_names: HashSet<JsWord>,
  // (ModuleId, ImportStarId)
  pub(crate) external_modules_of_re_export_all: LinkedHashSet<ModuleId>,

  pub(crate) runtime_helpers: RuntimeHelpers,

  // -- Used to treeshake
  pub(crate) parts: StatementParts,

  // is imported dynamically
  pub(crate) is_dynamic_entry: bool,

  /// Comments of the source code
  #[derivative(Debug = "ignore")]
  pub(crate) comments: SwcComments,
}

impl NormalModule {
  /// We only need suggested names for following cases. We need a suggested names for
  /// those non-named variable.
  /// - non-named default export.
  /// ```js
  /// // index.js
  /// import foo from './foo.js';
  /// // foo.js
  /// export default 1;
  /// ```
  /// We need to generate a name for value `1` in `foo.js` and use `foo` as the name of it.
  ///
  /// - namespace import/namespace export.
  /// ```js
  /// // index.js
  /// import * as foo from './foo.js';
  /// export * as bar from './foo.js';
  /// // foo.js
  /// export const a = 1;
  /// ```
  /// We need to generate namespace export in `foo.js` and use `foo.a` as the name of it.
  pub(crate) fn suggest_name(&mut self, name: &JsWord, suggested: &JsWord) {
    // Only accept suggested name for non-named default export and namespace import/export.
    if name == &js_word!("default") || name == &js_word!("*") {
      // Fast path: In practice, only case `export { default } from './foo'` will give a
      // useless suggested name `default`.
      if suggested == &js_word!("default") {
        return;
      }

      self
        .suggested_names
        .raw_entry_mut()
        .from_key(name)
        .or_insert_with(|| (name.clone(), suggested.clone()));
    }
  }

  pub(crate) fn mark_namespace_id_referenced(&mut self) {
    self.is_facade_namespace_id_referenced = true;
  }

  pub(crate) fn find_exported(&self, exported_name: &JsWord) -> Option<&ExportedSpecifier> {
    if exported_name == "*" {
      Some(&self.facade_id_for_namespace)
    } else {
      self.linked_exports.get(exported_name)
    }
  }

  pub(crate) fn add_to_linked_exports(&mut self, name: JsWord, spec: ExportedSpecifier) {
    debug_assert!(&name != "*");
    debug_assert!(self
      .linked_exports
      .get(&name)
      .map_or(true, |founded| founded == &spec));

    self.linked_exports.insert(name, spec);
  }

  pub fn add_to_linked_imports(&mut self, importee: &ModuleId, spec: ImportedSpecifier) {
    self
      .linked_imports
      .raw_entry_mut()
      .from_key(importee)
      .or_insert_with(|| (importee.clone(), Default::default()))
      .1
      .insert(spec);
  }

  pub(crate) fn add_statement_part(&mut self, part: StatementPart) {
    self.parts.add(part);
  }

  pub(crate) fn contains_top_level_name(&self, name: &JsWord) -> bool {
    let top_level_symbol = Symbol::new(name.clone(), self.top_level_ctxt);
    self
      .parts
      .declared_id_to_statement_parts
      .contains_key(&top_level_symbol)
      || self.extra_top_level_symbols.contains(&top_level_symbol)
  }

  /// Create a top level symbol. If a symbol with the same name already exists, a new symbol will be created.
  pub(crate) fn create_top_level_symbol(&mut self, hint: &JsWord) -> Symbol {
    // First make sure the name is valid.
    let mut name = Ident::verify_symbol(hint)
      .map(|_| hint.clone())
      .unwrap_or_else(|suggested| suggested.into());

    let mut i = 0;
    while self.contains_top_level_name(&name) {
      i += 1;
      name = format!("{name}${i}").into();
    }
    let sym = Symbol::new(name, self.top_level_ctxt);
    self.extra_top_level_symbols.insert(sym.clone());
    sym
  }

  pub(crate) fn generate_namespace_export(&mut self) {
    if self.is_facade_namespace_id_referenced {
      if !self.external_modules_of_re_export_all.is_empty() {
        self.runtime_helpers.merge_namespaces();
      };

      let external_modules_and_star_symbol = self
        .external_modules_of_re_export_all
        .clone()
        .iter()
        .map(|external_id| {
          let import_star_as_symbol = self.create_top_level_symbol(external_id.id());
          // TODO: We might need to check if the importer is a already got import star from importee
          // And we could reuse that Symbol

          self.add_to_linked_imports(
            external_id,
            ImportedSpecifier {
              imported_as: import_star_as_symbol.clone(),
              imported: js_word!("*"),
            },
          );

          (external_id.clone(), import_star_as_symbol)
        })
        .collect_vec();

      self.add_statement_part(StatementPart {
        declared: HashSet::from_iter([self.facade_id_for_namespace.local_id.clone()]),
        referenced: self
          .linked_exports
          .values()
          .map(|spec| &spec.local_id)
          .chain(
            external_modules_and_star_symbol
              .iter()
              .map(|(_, star_symbol)| star_symbol),
          )
          .cloned()
          .collect(),
        is_included: false.into(),
        side_effect: false,
      });

      // Make sure all exports are imported.
      // Treeshake rely on this.
      self
        .linked_exports
        .clone()
        .into_iter()
        .for_each(|(_, spec)| {
          // Forbid cycle
          if spec.owner == self.id {
            return;
          }
          self.add_to_linked_imports(
            &spec.owner,
            ImportedSpecifier {
              imported_as: spec.local_id,
              imported: spec.exported_as,
            },
          );
        });

      let namespace_export = rolldown_ast_template::build_namespace_export_stmt(
        self.facade_id_for_namespace.local_id.clone().to_id(),
        self
          .linked_exports
          .iter()
          .map(|(name, spec)| (name.clone(), spec.local_id.clone().to_id()))
          .collect(),
        external_modules_and_star_symbol
          .iter()
          .map(|(_, id)| id.clone().to_id())
          .collect(),
      );

      self.ast.body.push(namespace_export);
    }
  }

  #[instrument(skip_all)]
  pub(crate) fn render(&self, _ctx: &RenderContext, options: &InputOptions) -> String {
    let comments = SingleThreadedComments::default();

    let mut text = String::new();
    text.push(' ');
    text.push_str(&self.id.as_path().relative(&options.cwd).to_string_lossy());
    comments.add_leading(
      self.ast.span_lo(),
      Comment {
        kind: CommentKind::Line,
        span: self.ast.span(),
        text: text.into(),
      },
    );

    COMPILER.print(&self.ast, Some(&comments)).unwrap()
  }

  pub(crate) fn suggested_name_for(&self, sym: &JsWord) -> Option<JsWord> {
    let ret = self
      .suggested_names
      .get(sym)
      .cloned()
      .map(|s| make_legal(&s).into());

    if ret.as_ref().is_none() && sym == "default" {
      return Some(
        make_legal(
          &self
            .id
            .as_path()
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap(),
        )
        .into(),
      );
    }

    ret
  }

  pub(crate) fn is_included(&self) -> bool {
    !self.ast.body.is_empty()
  }
}

#[derive(Debug)]
pub(crate) struct StatementParts {
  pub(crate) parts: Vec<StatementPart>,
  /// This value is a HashSet. Consider case
  /// ```js
  /// var baz = foo;
  /// var baz = bar;
  /// ```
  pub(crate) declared_id_to_statement_parts: HashMap<Symbol, HashSet<usize>>,
}

impl StatementParts {
  pub(crate) fn from_parts(parts: Vec<StatementPart>) -> Self {
    let mut declared_id_to_statement_parts: HashMap<Symbol, HashSet<usize>> = HashMap::default();
    parts.iter().enumerate().for_each(|(i, part)| {
      part.declared.iter().for_each(|id| {
        declared_id_to_statement_parts
          .entry(id.clone())
          .or_default()
          .insert(i);
      })
    });

    Self {
      parts,
      declared_id_to_statement_parts,
    }
  }

  pub(crate) fn add(&mut self, part: StatementPart) {
    let idx = self.parts.len();
    part.declared.iter().for_each(|id| {
      self
        .declared_id_to_statement_parts
        .entry(id.clone())
        .or_default()
        .insert(idx);
    });
    self.parts.push(part);
  }

  /// Notice
  /// - A symbol could be declared in multiple statements.
  /// Consider case
  /// ```js
  /// var baz = '1';
  /// var baz = '2';
  /// ```
  pub(crate) fn find_parts_where_symbol_declared(
    &self,
    symbol: &Symbol,
  ) -> Option<impl Iterator<Item = &StatementPart>> {
    self
      .declared_id_to_statement_parts
      .get(symbol)
      .map(|idx_list| idx_list.iter().map(|idx| &self.parts[*idx]))
  }

  pub(crate) fn declared_ids(&self) -> impl Iterator<Item = &Symbol> {
    self.parts.iter().flat_map(|part| part.declared.iter())
  }
}
