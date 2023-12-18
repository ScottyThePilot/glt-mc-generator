use std::num::NonZeroUsize;
use std::ops::{Deref, Index};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListIndex(NonZeroUsize);

impl ListIndex {
  fn new(n: usize) -> Self {
    ListIndex(n.checked_add(1).and_then(NonZeroUsize::new).unwrap())
  }

  fn get(self) -> usize {
    self.0.get().checked_sub(1).unwrap()
  }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListCache<T> {
  values: Vec<T>
}

impl<T> ListCache<T> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn get(&self, index: ListIndex) -> Option<&T> {
    self.values.get(index.get())
  }

  pub fn get_or_insert(&mut self, value: T) -> ListIndex where T: Eq {
    match self.values.iter().position(|v| v == &value) {
      Some(n) => ListIndex::new(n),
      None => {
        let n = self.values.len();
        self.values.push(value);
        ListIndex::new(n)
      }
    }
  }
}

impl<T> Deref for ListCache<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    self.values.as_slice()
  }
}

impl<T> AsRef<[T]> for ListCache<T> {
  fn as_ref(&self) -> &[T] {
    self.values.as_slice()
  }
}

impl<T> Index<ListIndex> for ListCache<T> {
  type Output = T;

  fn index(&self, index: ListIndex) -> &Self::Output {
    self.get(index).expect("index not found in index cache")
  }
}

impl<T> Default for ListCache<T> {
  fn default() -> Self {
    ListCache { values: Vec::default() }
  }
}
