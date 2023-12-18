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

/// A structure for holding generic 3D world data, where `T` represents a block.
/// This type implements [`GeometryReceiver`], so can be used to
/// render [`GeometryDescriber`][crate::geometry::GeometryDescriber]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldData<T> {
  grid: ExGridSparse<T, 16, RandomState>
}

impl<T> WorldData<T> {
  /// Creates a new, empty [`WorldData`].
  pub fn new() -> Self {
    Self::default()
  }

  /// Gets a block ref from the [`WorldData`] at a given point.
  pub fn get(&self, pos: Vec3) -> Option<&T> {
    self.grid.get(pos)
  }

  /// Gets a mutable block ref from the [`WorldData`] at a given point.
  pub fn get_mut(&mut self, pos: Vec3) -> Option<&mut T> {
    self.grid.get_mut(pos)
  }

  /// Gets a mutable reference to a block cell in the [`WorldData`] at a given point.
  /// This is useful if you need the ability to remove, replace and read the block at a given position, all at the same time.
  pub fn get_mut_cell(&mut self, pos: Vec3) -> &mut Option<T> {
    self.grid.get_mut_default(pos)
  }

  /// Places a block into the [`WorldData`] at a given point, returning the block it replaced if one was there.
  pub fn insert(&mut self, pos: Vec3, value: T) -> Option<T> {
    self.grid.insert(pos, value)
  }

  /// Removes a block from the [`WorldData`] at a given point, if it exists.
  pub fn remove(&mut self, pos: Vec3) -> Option<T> {
    self.grid.get_mut_default(pos).take()
  }

  pub fn entry(&mut self, pos: Vec3) -> WorldDataEntry<T> {
    self.grid.entry(pos)
  }

  /// Ensures that a chunk is present internally at the given position.
  pub fn touch_chunk(&mut self, pos: impl Into<[i32; 3]>) {
    self.grid.get_chunk_entry(pos).or_default();
  }

  /// Gets the approximate bounding box of this [`WorldData`]'s contents.
  pub fn bounding_box(&self) -> Option<BoundingBox3> {
    self.grid.naive_bounds().map(|(min, max)| {
      BoundingBox3::new(Vec3::from(min), Vec3::from(max))
    })
  }

  /// Iterate over the blocks of this [`WorldData`] by reference.
  #[inline]
  pub fn iter(&self) -> WorldDataIter<T> {
    self.into_iter()
  }

  /// Iterate over the blocks of this [`WorldData`] by mutable reference.
  #[inline]
  pub fn iter_mut(&mut self) -> WorldDataIterMut<T> {
    self.into_iter()
  }

  /// Iterate over the blocks of this [`WorldData`], and their positions, by reference.
  #[inline]
  pub fn cells(&self) -> WorldDataCells<T> {
    self.grid.cells()
  }

  /// Iterate over the blocks of this [`WorldData`], and their positions, by mutable reference.
  #[inline]
  pub fn cells_mut(&mut self) -> WorldDataCellsMut<T> {
    self.grid.cells_mut()
  }

  /// Iterate over the blocks of this [`WorldData`], and their positions, consuming it.
  #[inline]
  pub fn into_cells(self) -> WorldDataIntoCells<T> {
    self.grid.into_cells()
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
