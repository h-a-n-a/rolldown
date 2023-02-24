use std::{fmt::Debug, hash::Hash, sync::Mutex};

use ena::unify::{InPlaceUnificationTable, UnifyKey};
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnaKey(u32);

impl UnifyKey for EnaKey {
  type Value = ();

  fn index(&self) -> u32 {
    self.0
  }

  fn from_index(u: u32) -> Self {
    EnaKey(u)
  }

  fn tag() -> &'static str {
    "EnaKey"
  }
}

#[derive(Debug, Default)]
pub struct UnionFind<Key: Eq + Hash + Clone + Debug> {
  store: Mutex<InPlaceUnificationTable<EnaKey>>,
  store_key_to_key: Mutex<FxHashMap<EnaKey, Key>>,
  key_to_store_key: Mutex<FxHashMap<Key, EnaKey>>,
  // stored: Vec<Key>,
}

impl<Key: Eq + Hash + Clone + Debug> UnionFind<Key> {
  fn intern_key(&mut self, key: &Key) -> EnaKey {
    *self
      .key_to_store_key
      .get_mut()
      .unwrap()
      .entry(key.clone())
      .or_insert_with_key(|key| {
        let ena_key = self.store.get_mut().unwrap().new_key(());
        self
          .store_key_to_key
          .get_mut()
          .unwrap()
          .insert(ena_key, key.clone());
        ena_key
      })
  }

  fn intern_key_par(&self, key: &Key) -> EnaKey {
    *self
      .key_to_store_key
      .lock()
      .unwrap()
      .entry(key.clone())
      .or_insert_with_key(|key| {
        let ena_key = self.store.lock().unwrap().new_key(());
        self
          .store_key_to_key
          .lock()
          .unwrap()
          .insert(ena_key, key.clone());
        ena_key
      })
  }

  pub fn union(&mut self, key1: &Key, key2: &Key) -> &mut Self {
    let k1 = self.intern_key(key1);
    let k2 = self.intern_key(key2);
    self.store.get_mut().unwrap().union(k1, k2);
    self
  }

  pub fn unioned(&mut self, key1: &Key, key2: &Key) -> bool {
    let k1 = self.intern_key(key1);
    let k2 = self.intern_key(key2);
    self.store.get_mut().unwrap().unioned(k1, k2)
  }

  pub fn find_root(&mut self, key: &Key) -> Option<&Key> {
    let ena_key = self.intern_key(key);
    let ena_root = self.store.get_mut().unwrap().find(ena_key);
    self.store_key_to_key.get_mut().unwrap().get(&ena_root)
  }

  pub fn find_root_par(&self, key: &Key) -> Option<Key> {
    let ena_key = self.intern_key_par(key);
    let ena_root = self.store.lock().unwrap().find(ena_key);
    self
      .store_key_to_key
      .lock()
      .unwrap()
      .get(&ena_root)
      .cloned()
  }
}
