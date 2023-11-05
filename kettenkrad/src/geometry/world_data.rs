use ahash::random_state::RandomState;
use exgrid::dim3::grid::{
  ExGridSparseCells,
  ExGridSparseCellsMut,
  ExGridSparseIntoCells,
  ExGridSparseIntoIter,
  ExGridSparseIter,
  ExGridSparseIterMut,
  ExGridSparse,
  ExGridSparseEntry
};

use crate::Vec3;
use crate::geometry::{GeometryReceiver, BoundingBox3};

use std::num::NonZeroUsize;
use std::ops::Index;



impl<T> GeometryReceiver for WorldData<T> {
  type Block = T;

  fn receive_block(&mut self, pos: Vec3, block: Self::Block) {
    self.insert(pos, block);
  }
}

pub type WorldDataEntry<'a, T> = ExGridSparseEntry<'a, T, 16>;
pub type WorldDataIter<'a, T> = ExGridSparseIter<'a, T, 16>;
pub type WorldDataIterMut<'a, T> = ExGridSparseIterMut<'a, T, 16>;
pub type WorldDataIntoIter<T> = ExGridSparseIntoIter<T, 16>;
pub type WorldDataCells<'a, T> = ExGridSparseCells<'a, T, 16>;
pub type WorldDataCellsMut<'a, T> = ExGridSparseCellsMut<'a, T, 16>;
pub type WorldDataIntoCells<T> = ExGridSparseIntoCells<T, 16>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldData<T> {
  grid: ExGridSparse<T, 16, RandomState>
}

impl<T> WorldData<T> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn get(&self, pos: Vec3) -> Option<&T> {
    self.grid.get(pos)
  }

  pub fn get_mut(&mut self, pos: Vec3) -> Option<&mut T> {
    self.grid.get_mut(pos)
  }

  pub fn get_mut_default(&mut self, pos: Vec3) -> &mut Option<T> {
    self.grid.get_mut_default(pos)
  }

  pub fn insert(&mut self, pos: Vec3, value: T) -> Option<T> {
    self.grid.insert(pos, value)
  }

  pub fn entry(&mut self, pos: Vec3) -> WorldDataEntry<T> {
    self.grid.entry(pos)
  }

  pub fn bounding_box(&self) -> Option<BoundingBox3> {
    self.grid.naive_bounds().map(|(min, max)| {
      BoundingBox3::new(Vec3::from(min), Vec3::from(max))
    })
  }

  #[inline]
  pub fn iter(&self) -> WorldDataIter<T> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> WorldDataIterMut<T> {
    self.into_iter()
  }
}

impl<'a, T> IntoIterator for &'a WorldData<T> {
  type Item = &'a T;
  type IntoIter = WorldDataIter<'a, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.grid.iter()
  }
}

impl<'a, T> IntoIterator for &'a mut WorldData<T> {
  type Item = &'a mut T;
  type IntoIter = WorldDataIterMut<'a, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.grid.iter_mut()
  }
}

impl<T> IntoIterator for WorldData<T> {
  type Item = T;
  type IntoIter = WorldDataIntoIter<T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.grid.into_iter()
  }
}

impl<T> Default for WorldData<T> {
  fn default() -> Self {
    WorldData { grid: ExGridSparse::default() }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexCache<T> {
  values: Vec<T>
}

impl<T> IndexCache<T> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn get(&self, index: NonZeroUsize) -> Option<&T> {
    self.values.get(from_index(index))
  }

  pub fn get_mut(&mut self, index: NonZeroUsize) -> Option<&mut T> {
    self.values.get_mut(from_index(index))
  }

  pub fn get_or_insert(&mut self, value: T) -> NonZeroUsize where T: Eq {
    match self.values.iter().position(|v| v == &value) {
      Some(n) => into_index(n),
      None => {
        let n = self.values.len();
        self.values.push(value);
        into_index(n)
      }
    }
  }
}

impl<T> Index<NonZeroUsize> for IndexCache<T> {
  type Output = T;

  fn index(&self, index: NonZeroUsize) -> &Self::Output {
    self.get(index).expect("index not found in index cache")
  }
}

impl<T> Default for IndexCache<T> {
  fn default() -> Self {
    IndexCache { values: Vec::default() }
  }
}

fn into_index(n: usize) -> NonZeroUsize {
  n.checked_add(1).and_then(NonZeroUsize::new).unwrap()
}

fn from_index(n: NonZeroUsize) -> usize {
  n.get().checked_sub(1).unwrap()
}
