use petgraph::dot::Dot;
use petgraph::prelude::*;
use std::borrow::BorrowMut;
use std::collections::HashMap;

use dashmap::DashMap;
use ena::unify::{
  InPlace, InPlaceUnificationTable, UnificationStoreBase, UnificationTable, UnifyKey,
};
use once_cell::sync::Lazy;
use petgraph::graph::{EdgeReference, NodeIndex};
use petgraph::Graph;
use swc_atoms::JsWord;
use swc_common::sync::{Lock, Lrc};
use swc_common::{Globals, SourceMap, SyntaxContext, GLOBALS};
use swc_ecma_visit::VisitMutWith;

use crate::external_module::ExternalModule;
use crate::module::Module;
use crate::scanner::rel::{DynImportDesc, ImportDesc, ReExportDesc};
use crate::scanner::Scanner;
use crate::types::ResolvedId;
use crate::utils::resolve_id::resolve_id;

pub(crate) static SOURCE_MAP: Lazy<Lrc<SourceMap>> = Lazy::new(Default::default);

#[derive(Debug, Hash, PartialEq, Eq, Clone)]

pub enum DepNode {
  Mod(Module),
  Ext(ExternalModule),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Rel {
  Import(ImportDesc),
  DynImport(DynImportDesc),
  ReExport(ReExportDesc),
  ReExportAll,
}

pub type DepGraph = Graph<DepNode, Rel>;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Ctxt(pub SyntaxContext, u32);

impl From<SyntaxContext> for Ctxt {
  fn from(ctxt: SyntaxContext) -> Self {
    Ctxt::new(ctxt)
  }
}

impl AsRef<SyntaxContext> for Ctxt {
  fn as_ref(&self) -> &SyntaxContext {
    &self.0
  }
}

// static map: DashMap<u32, SyntaxContext> = DashMap::new();
static ctxt_to_idx: Lazy<DashMap<SyntaxContext, u32>> = Lazy::new(|| Default::default());
static idx_to_ctxt_map: Lazy<DashMap<u32, Ctxt>> = Lazy::new(|| Default::default());

impl Ctxt {
  pub fn new(ctxt: SyntaxContext) -> Self {
    println!("Ctxt");
    let default_id = ctxt_to_idx.len() as u32;
    let next_id = ctxt_to_idx.entry(ctxt.clone()).or_insert(default_id);
    println!("Ctxt2");
    idx_to_ctxt_map
      .entry(*next_id)
      .or_insert_with(|| Self(ctxt, *next_id))
      .clone()
  }

  pub fn ctxt(&self) -> SyntaxContext {
    self.0
  }
}

impl UnifyKey for Ctxt {
  type Value = ();
  fn index(&self) -> u32 {
    self.1
  }
  fn from_index(u: u32) -> Self {
    idx_to_ctxt_map.get(&u).unwrap().clone()
  }
  fn tag() -> &'static str {
    "tag"
  }
}

#[non_exhaustive]
pub struct GraphContainer {
  pub entry_path: String,
  pub graph: DepGraph,
  pub entries: Vec<NodeIndex>,
  pub ordered_modules: Vec<NodeIndex>,
  // pub asserted_globals: HashMap<JsWord, bool>,
  pub symbol_rel: InPlaceUnificationTable<Ctxt>,
  // pub globals: Globals,
}

impl GraphContainer {
  pub fn new(entry: String) -> Self {
    env_logger::init();

    let graph = Graph::default();

    let s = Self {
      entry_path: entry,
      graph,
      entries: Default::default(),
      ordered_modules: Default::default(),
      // asserted_globals: Default::default(),
      symbol_rel: Default::default(),
    };
    s
  }

  // build dependency graph via entry modules.
  fn generate_module_graph(&mut self) {
    let entry_module = Module::new(self.entry_path.clone(), true);
    let mut module_id_to_node_idx_map = Default::default();
    let mut ctx = AnalyseContext {
      graph: &mut self.graph,
      module_id_to_node_idx_map: &mut module_id_to_node_idx_map,
    };
    let entry = analyse_module(&mut ctx, entry_module, None, Rel::ReExportAll);
    self.entries.push(entry)
  }

  pub fn build(&mut self) {
    let globals = Globals::new();
    GLOBALS.set(&globals, || {
      self.generate_module_graph();

      self.sort_modules();

      self.link_modules();

      self.include_statements();
    });

    println!("entry_modules {:?}", Dot::new(&self.graph))
  }

  fn include_statements(&mut self) {
    // TODO: tree-shaking
    self.graph.node_indices().into_iter().for_each(|idx| {
      if let DepNode::Mod(m) = &mut self.graph[idx] {
        m.include_all();
      }
    });
  }
  //
  // fn link_import_name(&mut self) {}
  //
  // fn link_module(&mut self, curr_module: &Module, edge: EdgeReference<Rel>) {
  //   let target_node = &mut self.graph[edge.target()];
  //
  //   match edge.weight() {
  //     Rel::Import(import_desc) => {
  //       match target_node {
  //         DepNode::Mod(target_module) => {
  //           let current_ctxt = curr_module
  //             .definitions
  //             .get(&import_desc.local_name.clone())
  //             .unwrap();
  //
  //           // We have no idea whether the name is defined in the target module, if not we need to recursively find the name in re-exported modules of target module.
  //
  //           loop {
  //             if let Some(export_desc) = target_module
  //               .scanner
  //               .as_ref()
  //               .unwrap()
  //               .exports
  //               .get(&import_desc.local_name.clone())
  //             {
  //               self
  //                 .symbol_rel
  //                 .union(Symbol(current_ctxt.clone()), Symbol(export_desc.ctxt));
  //               break;
  //             } else {
  //               target_module
  //                 .scanner
  //                 .as_ref()
  //                 .unwrap()
  //                 .export_all_sources
  //                 .iter()
  //                 .for_each(|src: &JsWord| {})
  //             }
  //           }
  //
  //           if target_module
  //             .suggested_names
  //             .get(&import_desc.name.as_ref().to_owned())
  //             .is_none()
  //           {
  //             target_module.suggested_names.insert(
  //               import_desc.name.as_ref().to_owned(),
  //               import_desc.local_name.as_ref().to_owned(),
  //             );
  //           }
  //         }
  //         _ => {}
  //       };
  //     }
  //     Rel::ReExport(reexport_desc) => {}
  //     Rel::DynImport(_) => {}
  //     Rel::ReExportAll => {}
  //   }
  // }

  fn link_each(
    curr_module: &Module,
    target_module: &Module,
    import_desc: &ImportDesc,
    symbol_rel: &mut InPlaceUnificationTable<Ctxt>,
    graph: &DepGraph,
  ) {
    let current_ctxt: Ctxt = curr_module
      .definitions
      .get(&import_desc.local_name)
      .unwrap()
      .clone()
      .into();

    if let Some(export_desc) = target_module
      .scanner
      .as_ref()
      .unwrap()
      .exports
      .get(&import_desc.local_name)
    {
      match export_desc.identifier.as_ref() {
        Some(ident) => {
          let decl_ctxt: Ctxt = target_module.definitions.get(ident).unwrap().clone().into();
          symbol_rel.union(current_ctxt, decl_ctxt);
        }
        _ => (),
      }

      match target_module.definitions.get(&export_desc.local_name) {
        Some(ctxt) => {
          let ctxt: Ctxt = ctxt.clone().into();
          symbol_rel.union(current_ctxt, ctxt);
        }
        None => (),
      }
    } else {
      target_module
        .scanner
        .as_ref()
        .unwrap()
        .export_all_sources
        .iter()
        .for_each(|src: &JsWord| {
          if let Some(module) =
            graph
              .node_indices()
              .into_iter()
              .find_map(|node_index| -> Option<&Module> {
                let node = &graph[node_index];

                if let DepNode::Mod(module) = node {
                  if module.id.as_str() == src.as_ref() {
                    return Some(module);
                  }
                }

                None
              })
          {
            Self::link_each(curr_module, module, import_desc, symbol_rel, graph)
          }
        })
    }
  }

  fn link_modules(&mut self) {
    self
      .graph
      .node_indices()
      .into_iter()
      .for_each(|node_index| {
        let graph = &self.graph[node_index];

        if let DepNode::Mod(curr_module) = graph {
          let edge = self.graph.edges_directed(node_index, Direction::Outgoing);
          edge.for_each(|e| {
            let target_node = &self.graph[e.target()];

            if let Rel::Import(import_desc) = e.weight() {
              if let DepNode::Mod(target_module) = target_node {
                // We have no idea whether the name is defined in the target module, if not we need to recursively find the name in re-exported modules of target module.
                Self::link_each(
                  curr_module,
                  target_module,
                  import_desc,
                  &mut self.symbol_rel,
                  &self.graph,
                );
              }
            }
          })
        }
      })
  }

  fn sort_modules(&mut self) {
    let mut dfs = DfsPostOrder::new(&self.graph, self.entries[0]);
    let mut ordered_modules = vec![];
    // FIXME: is this correct?
    while let Some(node) = dfs.next(&self.graph) {
      ordered_modules.push(node);
    }
    self.ordered_modules = ordered_modules;
  }
}

fn analyse_module(
  ctx: &mut AnalyseContext,
  mut module: Module,
  parent: Option<NodeIndex>,
  rel: Rel,
) -> NodeIndex {
  let source = std::fs::read_to_string(&module.id).unwrap();
  let scanner = module.set_source(source.clone());
  let module_id = module.id.clone();

  let node_idx;
  let has_seen;
  if let Some(idx) = ctx.module_id_to_node_idx_map.get(&module_id) {
    has_seen = true;
    node_idx = idx.clone();
  } else {
    has_seen = false;
    node_idx = ctx.graph.add_node(module.into());
    ctx
      .module_id_to_node_idx_map
      .insert(module_id.clone(), node_idx.clone());
  }

  if let Some(parent) = parent {
    ctx.graph.add_edge(parent, node_idx.clone(), rel);
  }

  if !has_seen {
    analyse_dep(ctx, scanner, &module_id, node_idx);
  }

  node_idx
}

struct AnalyseContext<'me> {
  pub graph: &'me mut DepGraph,
  pub module_id_to_node_idx_map: &'me mut HashMap<String, NodeIndex>,
}

fn analyse_external_module(
  ctx: &mut AnalyseContext,
  module: ExternalModule,
  parent: NodeIndex,
  rel: Rel,
) {
  let node_idx = ctx.graph.add_node(module.into());
  ctx.graph.add_edge(parent, node_idx, rel);
}

fn analyse_dep(ctx: &mut AnalyseContext, scanner: Scanner, module_id: &str, parent: NodeIndex) {
  scanner.imports.into_values().into_iter().for_each(|imp| {
    let unresolved_id = &imp.source;
    let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
    let mod_or_ext = resolve_module_by_resolved_id(resolved_id);
    analyse_mod_or_ext(ctx, mod_or_ext, parent, Rel::Import(imp));
  });

  scanner.dynamic_imports.into_iter().for_each(|dyn_imp| {
    let unresolved_id = &dyn_imp.argument;
    let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
    let mod_or_ext = resolve_module_by_resolved_id(resolved_id);
    analyse_mod_or_ext(ctx, mod_or_ext, parent, Rel::DynImport(dyn_imp));
  });

  scanner
    .re_exports
    .into_values()
    .into_iter()
    .for_each(|re_expr| {
      let unresolved_id = &re_expr.source;
      let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
      let mod_or_ext = resolve_module_by_resolved_id(resolved_id);
      analyse_mod_or_ext(ctx, mod_or_ext, parent, Rel::ReExport(re_expr));
    });

  scanner.export_all_sources.into_iter().for_each(|source| {
    let unresolved_id = &source;
    let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
    let mod_or_ext = resolve_module_by_resolved_id(resolved_id);
    analyse_mod_or_ext(ctx, mod_or_ext, parent, Rel::ReExportAll);
  });
}

fn analyse_mod_or_ext(ctx: &mut AnalyseContext, mod_or_ext: ModOrExt, parent: NodeIndex, rel: Rel) {
  match mod_or_ext {
    ModOrExt::Ext(ext) => analyse_external_module(ctx, ext, parent, rel),
    ModOrExt::Mod(m) => {
      analyse_module(ctx, m, Some(parent), rel);
    }
  }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ModOrExt {
  Mod(Module),
  Ext(ExternalModule),
}

fn resolve_module_by_resolved_id(resolved: ResolvedId) -> ModOrExt {
  if resolved.external {
    ModOrExt::Ext(ExternalModule::new(resolved.id))
  } else {
    ModOrExt::Mod(Module::new(resolved.id, false))
  }
}
