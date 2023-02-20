use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default, Debug)]
pub struct RuntimeHelpers {
  inner: Inner,
}

macro_rules! define_helpers {
    (
        Helpers {
            $( $name:ident ( $( $declared:ident ),* ): ( $( $dep:ident ),* ), )*
        }
    ) => {

        #[derive(Debug,Default)]
        struct Inner {
            $( $name: AtomicBool, )*
        }

        impl RuntimeHelpers {
            pub fn extend_from(&self, other: &Self) {
                $(
                    if other.inner.$name.load(Ordering::SeqCst) {
                        self.inner.$name.store(true, Ordering::Relaxed);
                    }
                )*
            }

            pub fn generate_helpers(&self) -> Vec<&'static str> {
                let mut to = vec![];
                $(
                    if self.inner.$name.load(Ordering::Relaxed) {
                        to.push(include_str!(concat!(
                            "./snippets/_",
                            stringify!($name),
                            ".js"
                        )));
                    }
                )*
                to
            }

            pub fn is_used_any_helpers(&self) -> bool {
                $(
                    if self.inner.$name.load(Ordering::Relaxed) {
                        return true;
                    }
                )*
                false
            }

            pub fn used_names(&self) -> HashSet<&'static str> {
                let mut to = HashSet::new();
                $(
                    if self.inner.$name.load(Ordering::Relaxed) {
                        $(
                            to.insert(stringify!($declared));
                        )*
                    }
                )*
                to
            }

            $(
                pub fn $name(&self) {
                    self.inner.$name.store(true, Ordering::Relaxed);
                    $(
                        self.$dep();
                    )*
                }
            )*
        }
    };
}

impl RuntimeHelpers {
  pub fn new() -> Self {
    Self::default()
  }
}

define_helpers!(Helpers {
    merge_namespaces(_mergeNamespaces): (),
});

#[test]
fn test() {
  let helpers = RuntimeHelpers::new();
  helpers.merge_namespaces();
  assert!(helpers.used_names().contains("_mergeNamespaces"));
  assert_eq!(
    helpers.generate_helpers(),
    vec![include_str!("./snippets/_merge_namespaces.js")]
  );
}
