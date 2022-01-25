use super::grid::{Grid, IndexGrid};



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoolGrid {
  inner: Grid<()>
}

impl BoolGrid {
  pub fn new() -> Self {
    BoolGrid {
      inner: Grid::new()
    }
  }

  pub fn min<I: IndexGrid>(&self) -> I {
    self.inner.min()
  }

  pub fn max<I: IndexGrid>(&self) -> I {
    self.inner.max()
  }

  /// Gets a value from the bitgrid
  pub fn get(&self, pos: impl IndexGrid) -> bool {
    self.inner.get(pos).is_some()
  }

  pub fn put(&mut self, pos: impl IndexGrid, value: bool) {
    if value {
      self.inner.put_expand(pos, ());
    } else {
      self.inner.remove(pos);
    };
  }

  /// Returns the (width, height) of the underlying data
  #[inline]
  pub fn size(&self) -> (usize, usize) {
    self.inner.size()
  }

  /// Returns the width of the underlying data
  #[inline]
  pub fn width(&self) -> usize {
    self.inner.width()
  }

  /// Returns the height of the underlying data
  #[inline]
  pub fn height(&self) -> usize {
    self.inner.height()
  }
}

impl<I> FromIterator<I> for BoolGrid
where I: IndexGrid {
  fn from_iter<A: IntoIterator<Item = I>>(iter: A) -> Self {
    let mut bool_grid = BoolGrid::new();
    for pos in iter {
      bool_grid.put(pos, true);
    };

    bool_grid
  }
}
