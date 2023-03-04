use swc_core::ecma::atoms::{js_word, JsWord};

use crate::ModuleFormat;

pub(crate) fn preset_of_used_names(format: &ModuleFormat) -> Vec<JsWord> {
  let mut preset = vec![js_word!("Object"), js_word!("Promise")];

  match format {
    ModuleFormat::Esm => {}
    ModuleFormat::Cjs => {
      preset.push(js_word!("module"));
      preset.push(js_word!("require"));
      preset.push("__filename".into());
      preset.push("__dirname".into());
    }
  }

  preset
}
