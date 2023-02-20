use once_cell::sync::Lazy;
use swc_common::{sync::Lrc, SourceMap};

pub(crate) static SOURCE_MAP: Lazy<Lrc<SourceMap>> = Lazy::new(Default::default);
