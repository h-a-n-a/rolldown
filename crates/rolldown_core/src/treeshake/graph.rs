use rayon::prelude::{ParallelBridge, ParallelIterator};
use rolldown_common::Symbol;
use rustc_hash::FxHashSet;
use swc_core::common::GLOBALS;

use super::TreeshakeContext;
use crate::{treeshake::TreeshakeNormalModule, Graph, UnaryBuildResult, COMPILER, SWC_GLOBALS};

impl Graph {
  pub(crate) fn treeshake(&mut self) -> UnaryBuildResult<()> {
    let used_ids = self
      .collect_all_used_ids()?
      .into_iter()
      .map(|id| id.to_id())
      .collect();

    tracing::debug!("used_ids: {:#?}", used_ids);
    self
      .module_by_id
      .values_mut()
      .par_bridge()
      .filter_map(|m| m.as_norm_mut())
      .for_each(|module| {
        GLOBALS.set(&SWC_GLOBALS, || {
          tracing::debug!(
            "[before treeshake]module: {},code: \n{}",
            module.id,
            COMPILER.debug_print(&module.ast, None).unwrap()
          );

          rolldown_swc_visitors::treeshake(
            &mut module.ast,
            self.unresolved_mark,
            &used_ids,
            module.top_level_ctxt,
            GLOBALS.set(&SWC_GLOBALS, || module.top_level_ctxt.outer()),
            COMPILER.cm.clone(),
            &module.comments,
          );
          tracing::debug!(
            "[after treeshake]module: {},code: \n{}",
            module.id,
            COMPILER.debug_print(&module.ast, None).unwrap()
          );
          // We don't need `export`, because of scope hoisting.
          rolldown_swc_visitors::remove_export_and_import(&mut module.ast);
        });

        module
          .linked_imports
          .values_mut()
          .par_bridge()
          .for_each(|specs| specs.retain(|spec| used_ids.contains(spec.imported_as.as_id())));

        module
          .linked_exports
          .retain(|_exported_name, spec| used_ids.contains(spec.local_id.as_id()));

        module.parts.parts.iter_mut().for_each(|part| {
          // If the Symbol is unused, delete it.
          // So, in deconflicting, it won't take up a name meaninglessly.
          part
            .declared
            .retain(|symbol| used_ids.contains(symbol.as_id()));
        });
      });
    Ok(())
  }

  pub(crate) fn collect_all_used_ids(&mut self) -> UnaryBuildResult<FxHashSet<Symbol>> {
    let ctx = TreeshakeContext {
      id_to_module: self
        .module_by_id
        .iter()
        .filter_map(|(id, m)| m.as_norm().map(|m| (id, TreeshakeNormalModule::new(m))))
        .collect(),
      errors: Default::default(),
      warnings: Default::default(),
    };
    tracing::debug!("ctx: {:#?}", ctx);
    let used_ids = ctx
      .id_to_module
      .values()
      .par_bridge()
      .map(|m| m.include(&ctx))
      .flatten()
      .collect::<FxHashSet<_>>();
    let errors = ctx.errors.into_inner().unwrap();
    let warnings = ctx.warnings.into_inner().unwrap();
    self.warnings.extend(warnings);
    if !errors.is_empty() {
      return Err(errors.into_iter().next().unwrap());
    }
    Ok(used_ids)
  }
}
