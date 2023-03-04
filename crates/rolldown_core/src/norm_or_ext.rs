use rolldown_common::ModuleId;
use rolldown_runtime_helpers::RuntimeHelpers;
use swc_core::common::SyntaxContext;

use crate::{external_module::ExternalModule, normal_module::NormalModule};

#[derive(Debug)]
pub enum NormOrExt {
  Normal(NormalModule),
  External(ExternalModule),
}

impl NormOrExt {
  pub fn id(&self) -> &ModuleId {
    match self {
      NormOrExt::Normal(module) => &module.id,
      NormOrExt::External(module) => &module.id,
    }
  }

  pub fn dependencies(&self) -> &[ModuleId] {
    static DUMMY: [ModuleId; 0] = [];
    match self {
      NormOrExt::Normal(module) => &module.dependencies,
      NormOrExt::External(_) => &DUMMY,
    }
  }

  pub fn dynamic_dependencies(&self) -> &[ModuleId] {
    static DUMMY: [ModuleId; 0] = [];
    match self {
      NormOrExt::Normal(module) => &module.dyn_dependencies,
      NormOrExt::External(_) => &DUMMY,
    }
  }

  pub fn exec_order(&self) -> usize {
    match self {
      NormOrExt::Normal(module) => module.exec_order,
      NormOrExt::External(m) => m.exec_order,
    }
  }

  pub fn set_exec_order(&mut self, exec_order: usize) {
    match self {
      NormOrExt::Normal(module) => module.exec_order = exec_order,
      NormOrExt::External(m) => m.exec_order = exec_order,
    }
  }

  pub fn as_norm(&self) -> Option<&NormalModule> {
    match self {
      NormOrExt::Normal(m) => Some(m),
      _ => None,
    }
  }

  pub fn as_norm_mut(&mut self) -> Option<&mut NormalModule> {
    match self {
      NormOrExt::Normal(m) => Some(m),
      _ => None,
    }
  }

  #[allow(unused)]
  pub fn as_ext(&self) -> Option<&ExternalModule> {
    match self {
      NormOrExt::External(m) => Some(m),
      _ => None,
    }
  }

  #[allow(unused)]
  pub fn as_ext_mut(&mut self) -> Option<&mut ExternalModule> {
    match self {
      NormOrExt::External(m) => Some(m),
      _ => None,
    }
  }

  pub fn expect_norm(&self) -> &NormalModule {
    if let NormOrExt::Normal(m) = self {
      m
    } else {
      panic!("Expected NormalModule, Got ExternalModule({})", self.id())
    }
  }

  pub fn expect_norm_mut(&mut self) -> &mut NormalModule {
    if let NormOrExt::Normal(m) = self {
      m
    } else {
      panic!("Expected NormalModule, Got ExternalModule({})", self.id())
    }
  }

  #[allow(unused)]
  pub fn expect_ext(&self) -> &ExternalModule {
    if let NormOrExt::External(m) = self {
      m
    } else {
      panic!("expected ExternalModule")
    }
  }

  pub(crate) fn runtime_helpers(&self) -> &RuntimeHelpers {
    match self {
      NormOrExt::Normal(m) => &m.runtime_helpers,
      NormOrExt::External(m) => &m.runtime_helpers,
    }
  }

  pub(crate) fn top_level_ctxt(&self) -> SyntaxContext {
    match self {
      NormOrExt::Normal(m) => m.top_level_ctxt,
      NormOrExt::External(m) => m.top_level_ctxt,
    }
  }
}
