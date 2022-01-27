use glam::IVec2;

use std::collections::VecDeque;
use std::collections::vec_deque::Iter as VecDequeIter;
use std::collections::vec_deque::IterMut as VecDequeIterMut;
use std::collections::vec_deque::IntoIter as VecDequeIntoIter;
use std::iter::{FilterMap, FlatMap, FusedIterator, DoubleEndedIterator};
use std::ops::{Index, IndexMut};



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid<T> {
  rows: VecDeque<VecDeque<Option<T>>>,
  columns: usize,
  offset: [isize; 2]
}

impl<T> Grid<T> {
  /// Creates a new empty grid
  pub fn new() -> Self {
    Grid {
      rows: unit(),
      columns: 1,
      offset: [0, 0]
    }
  }

  /// Creates a new empty grid with a specific offset
  pub fn with_offset(offset: impl IndexGrid) -> Self {
    Grid {
      rows: unit(),
      columns: 1,
      offset: offset.into_indexes()
    }
  }

  /// Converts a position to an absolute index within the grid's data
  fn pos(&self, [x, y]: [isize; 2]) -> Option<[usize; 2]> {
    let x = x.checked_sub(self.offset[0]).expect("integer overflow");
    let y = y.checked_sub(self.offset[1]).expect("integer overflow");
    if x < 0 || y < 0 { return None };

    let x = x as usize;
    let y = y as usize;
    if x < self.width() && y < self.height() {
      Some([x, y])
    } else {
      None
    }
  }

  /// Returns a measurement of how far out of bounds a position is
  fn oob(&self, [x, y]: [isize; 2]) -> [isize; 2] {
    let x = x.checked_sub(self.offset[0]).expect("integer overflow");
    let y = y.checked_sub(self.offset[1]).expect("integer overflow");

    let width = self.width() as isize;
    let height = self.height() as isize;
    let x = if x < 0 { x } else if x >= width { x - width + 1 } else { 0 };
    let y = if y < 0 { y } else if y >= height { y - height + 1 } else { 0 };

    [x, y]
  }

  pub fn min<I: IndexGrid>(&self) -> I {
    I::from_indexes(self.offset)
  }

  pub fn max<I: IndexGrid>(&self) -> I {
    let x = self.width() as isize + self.offset[0] - 1;
    let y = self.height() as isize + self.offset[1] - 1;
    I::from_indexes([x, y])
  }

  /// Gets a value from the grid
  pub fn get(&self, pos: impl IndexGrid) -> Option<&T> {
    let [x, y] = self.pos(pos.into_indexes())?;
    self.rows[y][x].as_ref()
  }

  /// Gets a mutable reference to a value in the grid
  pub fn get_mut(&mut self, pos: impl IndexGrid) -> Option<&mut T> {
    let [x, y] = self.pos(pos.into_indexes())?;
    self.rows[y][x].as_mut()
  }

  pub fn contains(&mut self, pos: impl IndexGrid) -> bool {
    self.get(pos).is_some()
  }

  /// Places a value into the grid, returns true if the operation succeeded (was within bounds)
  pub fn put(&mut self, pos: impl IndexGrid, value: T) -> bool {
    if let Some([x, y]) = self.pos(pos.into_indexes()) {
      self.rows[y][x] = Some(value);
      true
    } else {
      false
    }
  }

  /// Places a value into the grid, expanding the underlying data if it is not big enough
  pub fn put_expand(&mut self, pos: impl IndexGrid, value: T) {
    let pos = pos.into_indexes();
    self.expand_to_include(pos);

    let [x, y] = match self.pos(pos) {
      Some(xy) => xy,
      None => unreachable!()
    };

    self.rows[y][x] = Some(value);
  }

  pub fn remove(&mut self, pos: impl IndexGrid) -> Option<T> {
    let [x, y] = self.pos(pos.into_indexes())?;
    std::mem::replace(&mut self.rows[y][x], None)
  }

  /// Expands the underlying data to contain the given position
  pub fn expand_to_include(&mut self, pos: impl IndexGrid) {
    let [x_oob, y_oob] = self.oob(pos.into_indexes());

    match x_oob.signum() {
      0 => (),
      1 => for _ in 0..x_oob {
        self.push_columns_back();
      },
      -1 => for _ in 0..-x_oob {
        self.push_columns_front();
      },
      _ => unreachable!()
    };

    match y_oob.signum() {
      0 => (),
      1 => for _ in 0..y_oob {
        self.push_rows_back();
      },
      -1 => for _ in 0..-y_oob {
        self.push_rows_front();
      },
      _ => unreachable!()
    };
  }

  /// Adds a new row to the back (positive y) of the grid
  pub fn push_rows_back(&mut self) {
    self.rows.push_back(empty_row(self.width()));
  }

  /// Adds a new row to the front (negative y) of the grid
  pub fn push_rows_front(&mut self) {
    self.offset[1] -= 1;
    self.rows.push_front(empty_row(self.width()));
  }

  /// Adds a new column to the back (positive x) of the grid
  pub fn push_columns_back(&mut self) {
    self.columns += 1;
    for row in self.rows.iter_mut() {
      row.push_back(None);
    };
  }

  /// Adds a new column to the front (negative x) of the grid
  pub fn push_columns_front(&mut self) {
    self.columns += 1;
    self.offset[0] -= 1;
    for row in self.rows.iter_mut() {
      row.push_front(None);
    };
  }

  /// Returns the (width, height) of the underlying data
  #[inline]
  pub fn size(&self) -> (usize, usize) {
    (self.width(), self.height())
  }

  /// Returns the width of the underlying data
  #[inline]
  pub fn width(&self) -> usize {
    self.columns
  }

  /// Returns the height of the underlying data
  #[inline]
  pub fn height(&self) -> usize {
    self.rows.len()
  }

  pub fn iter(&self) -> Iter<T> {
    let inner = self.rows.iter()
      .flat_map(VecDeque::iter as _)
      .filter_map(Option::as_ref as _);
    Iter { inner }
  }

  pub fn iter_mut(&mut self) -> IterMut<T> {
    let inner = self.rows.iter_mut()
      .flat_map(VecDeque::iter_mut as _)
      .filter_map(Option::as_mut as _);
    IterMut { inner }
  }

  pub fn into_iter(self) -> IntoIter<T> {
    #[inline(always)]
    fn noop<T>(t: T) -> T { t }

    let inner = self.rows.into_iter()
      .flat_map(VecDeque::into_iter as _)
      .filter_map(noop as _);
    IntoIter { inner }
  }

  pub fn enumerate<I>(&self) -> impl Iterator<Item = (I, &T)>
  where I: IndexGrid {
    let offset = self.offset;
    self.rows.iter()
      .enumerate()
      .flat_map(move |(y, row)| {
        row.iter()
          .enumerate()
          .filter_map(move |(x, value)| {
            let value = value.as_ref()?;
            let pos = inverse_pos::<I>(x, y, offset);
            Some((pos, value))
          })
      })
  }

  pub fn enumerate_mut<I>(&mut self) -> impl Iterator<Item = (I, &mut T)>
  where I: IndexGrid {
    let offset = self.offset;
    self.rows.iter_mut()
      .enumerate()
      .flat_map(move |(y, row)| {
        row.iter_mut()
          .enumerate()
          .filter_map(move |(x, value)| {
            let value = value.as_mut()?;
            let pos = inverse_pos::<I>(x, y, offset);
            Some((pos, value))
          })
      })
  }

  pub fn into_enumerate<I>(self) -> impl Iterator<Item = (I, T)>
  where I: IndexGrid  {
    let offset = self.offset;
    self.rows.into_iter()
      .enumerate()
      .flat_map(move |(y, row)| {
        row.into_iter()
          .enumerate()
          .filter_map(move |(x, value)| {
            let pos = inverse_pos::<I>(x, y, offset);
            Some((pos, value?))
          })
      })
  }
}

#[inline]
fn inverse_pos<I: IndexGrid>(x: usize, y: usize, offset: [isize; 2]) -> I {
  let x = x as isize + offset[0];
  let y = y as isize + offset[1];
  I::from_indexes([x, y])
}

impl<I, T> Index<I> for Grid<T>
where I: IndexGrid {
  type Output = T;

  fn index(&self, pos: I) -> &Self::Output {
    self.get(pos.into_indexes()).expect("index out of bounds")
  }
}

impl<I, T> IndexMut<I> for Grid<T>
where I: IndexGrid {
  fn index_mut(&mut self, pos: I) -> &mut Self::Output {
    self.get_mut(pos.into_indexes()).expect("index out of bounds")
  }
}

impl<T> IntoIterator for Grid<T> {
  type Item = T;
  type IntoIter = IntoIter<T>;

  fn into_iter(self) -> Self::IntoIter {
    self.into_iter()
  }
}

impl<'a, T> IntoIterator for &'a Grid<T> {
  type Item = &'a T;
  type IntoIter = Iter<'a, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'a, T> IntoIterator for &'a mut Grid<T> {
  type Item = &'a mut T;
  type IntoIter = IterMut<'a, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<I, T> FromIterator<(I, T)> for Grid<T>
where I: IndexGrid {
  fn from_iter<A: IntoIterator<Item = (I, T)>>(iter: A) -> Self {
    let mut grid = Grid::new();
    for (pos, value) in iter {
      grid.put_expand(pos, value);
    };

    grid
  }
}



macro_rules! impl_iterator_methods {
  () => {
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
      self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
      self.inner.size_hint()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where F: FnMut(B, Self::Item) -> B, {
      self.inner.fold(init, f)
    }
  };
}

macro_rules! impl_double_ended_iterator_methods {
  () => {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
      self.inner.next_back()
    }

    #[inline]
    fn rfold<A, F>(self, init: A, f: F) -> A
    where F: FnMut(A, Self::Item) -> A {
      self.inner.rfold(init, f)
    }
  };
}

pub struct Iter<'a, T> {
  inner: FilterMap<
    FlatMap<
      VecDequeIter<'a, VecDeque<Option<T>>>,
      VecDequeIter<'a, Option<T>>,
      for<'r> fn(&'r VecDeque<Option<T>>) -> VecDequeIter<'r, Option<T>>
    >,
    for<'r> fn(&'r Option<T>) -> Option<&'r T>
  >
}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a T;

  impl_iterator_methods!();
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
  impl_double_ended_iterator_methods!();
}

impl<'a, T> FusedIterator for Iter<'a, T> {}

pub struct IterMut<'a, T> {
  inner: FilterMap<
    FlatMap<
      VecDequeIterMut<'a, VecDeque<Option<T>>>,
      VecDequeIterMut<'a, Option<T>>,
      for<'r> fn(&'r mut VecDeque<Option<T>>) -> VecDequeIterMut<'r, Option<T>>
    >,
    for<'r> fn(&'r mut Option<T>) -> Option<&'r mut T>
  >
}

impl<'a, T> Iterator for IterMut<'a, T> {
  type Item = &'a mut T;

  impl_iterator_methods!();
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
  impl_double_ended_iterator_methods!();
}

impl<'a, T> FusedIterator for IterMut<'a, T> {}

pub struct IntoIter<T> {
  inner: FilterMap<
    FlatMap<
      VecDequeIntoIter<VecDeque<Option<T>>>,
      VecDequeIntoIter<Option<T>>,
      fn(VecDeque<Option<T>>) -> VecDequeIntoIter<Option<T>>,
    >,
    fn(Option<T>) -> Option<T>
  >
}

impl<T> Iterator for IntoIter<T> {
  type Item = T;

  impl_iterator_methods!();
}

impl<T> DoubleEndedIterator for IntoIter<T> {
  impl_double_ended_iterator_methods!();
}

impl<T> FusedIterator for IntoIter<T> {}



pub trait IndexGrid {
  fn into_indexes(self) -> [isize; 2];

  fn from_indexes(i: [isize; 2]) -> Self;
}

impl IndexGrid for [isize; 2] {
  #[inline]
  fn into_indexes(self) -> [isize; 2] {
    self
  }

  #[inline]
  fn from_indexes(i: [isize; 2]) -> Self {
    i
  }
}

impl IndexGrid for (isize, isize) {
  #[inline]
  fn into_indexes(self) -> [isize; 2] {
    [self.0, self.1]
  }

  #[inline]
  fn from_indexes(i: [isize; 2]) -> Self {
    (i[0], i[1])
  }
}

impl IndexGrid for IVec2 {
  #[inline]
  fn into_indexes(self) -> [isize; 2] {
    [self.x as isize, self.y as isize]
  }

  #[inline]
  fn from_indexes(i: [isize; 2]) -> Self {
    IVec2::new(i[0] as i32, i[1] as i32)
  }
}



#[inline]
fn empty_row<T>(len: usize) -> VecDeque<Option<T>> {
  let mut out = Vec::with_capacity(len);
  for _ in 0..len { out.push(None) };
  VecDeque::from(out)
}

#[inline]
fn unit<T>() -> VecDeque<VecDeque<Option<T>>> {
  VecDeque::from(vec![VecDeque::from(vec![None])])
}



#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn grid_1() {
    let mut grid: Grid<i32> = Grid::new();
    grid.put_expand([-1, -1], 15);
    grid.put_expand([1, 2], -15);
    grid.put_expand([0, 1], 0);

    assert_eq!(grid[[-1, -1]], 15);
    assert_eq!(grid[[1, 2]], -15);
    assert_eq!(grid[[0, 1]], 0);

    assert_eq!(grid, Grid {
      rows: VecDeque::from(vec![
        VecDeque::from(vec![Some(15), None, None]),
        VecDeque::from(vec![None, None, None]),
        VecDeque::from(vec![None, Some(0), None]),
        VecDeque::from(vec![None, None, Some(-15)]),
      ]),
      columns: 3,
      offset: [-1, -1]
    });

    assert_eq!([-1, -1], grid.min::<[isize; 2]>());
    assert_eq!([1, 2], grid.max::<[isize; 2]>());
  }
}
