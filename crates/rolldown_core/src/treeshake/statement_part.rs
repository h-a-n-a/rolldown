use rolldown_common::Symbol;
use rolldown_swc_visitors::StatementPart;
use rustc_hash::FxHashSet;

use super::{TreeshakeContext, TreeshakeNormalModule};

pub(super) trait Include {
  fn include(&self, ctx: &TreeshakeContext, module: &TreeshakeNormalModule) -> FxHashSet<Symbol>;
}

impl Include for StatementPart {
  fn include(&self, ctx: &TreeshakeContext, module: &TreeshakeNormalModule) -> FxHashSet<Symbol> {
    if !self
      .is_included
      .swap(true, std::sync::atomic::Ordering::SeqCst)
    {
      self
        .declared
        .iter()
        .chain(self.referenced.iter())
        .cloned()
        .chain(
          self
            .referenced
            .iter()
            .flat_map(|id| module.define_by_top_level_id(ctx, id)),
        )
        .collect()
    } else {
      Default::default()
    }
  }
}
