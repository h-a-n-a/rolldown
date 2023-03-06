use std::borrow::Cow;
use std::fmt::Display;

use swc_core::ecma::atoms as swc_atoms;
use swc_core::ecma::atoms::JsWord;
mod union_find;
pub use union_find::*;
mod symbol;
pub use symbol::*;
mod loader;
pub use loader::*;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ChunkId(JsWord);

impl ChunkId {
  pub fn new(value: impl Into<JsWord>) -> Self {
    Self(value.into())
  }

  pub fn value(&self) -> &JsWord {
    &self.0
  }
}

impl From<JsWord> for ChunkId {
  fn from(value: JsWord) -> Self {
    Self(value)
  }
}
impl From<String> for ChunkId {
  fn from(value: String) -> Self {
    Self(value.into())
  }
}
impl AsRef<str> for ChunkId {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ModuleId {
  value: JsWord,
  is_external: bool,
}

impl Display for ModuleId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value)
  }
}

impl ModuleId {
  pub fn new(value: impl Into<JsWord>, is_external: bool) -> Self {
    Self {
      value: value.into(),
      is_external,
    }
  }

  pub fn is_external(&self) -> bool {
    self.is_external
  }

  pub fn id(&self) -> &JsWord {
    &self.value
  }
}

impl AsRef<str> for ModuleId {
  fn as_ref(&self) -> &str {
    &self.value
  }
}

/// `export { foo as foo2 } from './foo'`, `foo` is `imported` and `foo2` is `exported_as`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReExportedSpecifier {
  pub exported_as: swc_atoms::JsWord,
  pub imported: swc_atoms::JsWord,
}

/// For `import { foo as foo2 } from './foo'`.
/// `foo` is `local_name` and `foo2` is `imported_as`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportedSpecifier {
  pub imported_as: Symbol,
  pub imported: swc_atoms::JsWord,
}

/// A `ExportedSpecifier` means
/// - A Symbol is exported from the owner.
/// - The owner either declared the symbol or imported it from the other module.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportedSpecifier {
  pub exported_as: JsWord,
  // export { foo as foo2 }, `foo` is local id and `foo2` is exported name.
  pub local_id: Symbol,
  // id of the module which exports the local id.
  pub owner: ModuleId,
}

pub type StaticStr = Cow<'static, str>;
