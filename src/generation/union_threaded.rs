use std::ops::{Deref, DerefMut};

use glam::IVec3;
use once_cell::sync::OnceCell;
use rayon::prelude::*;

use super::{Block, BoundingBox, Geometry, MaterialGeometry};



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionThreaded<G> {
  geometries: G,
  bounding_box: OnceCell<BoundingBox>
}

impl<G> UnionThreaded<G> {
  pub fn new(geometries: G) -> Self {
    UnionThreaded {
      geometries,
      bounding_box: OnceCell::new()
    }
  }
}

impl<G> Deref for UnionThreaded<G> {
  type Target = G;

  #[inline]
  fn deref(&self) -> &G {
    &self.geometries
  }
}

impl<G> DerefMut for UnionThreaded<G> {
  #[inline]
  fn deref_mut(&mut self) -> &mut G {
    &mut self.geometries
  }
}

impl<G, const N: usize> Geometry for UnionThreaded<[G; N]>
where G: Geometry + Sync {
  fn bounding_box(&self) -> BoundingBox {
    *self.bounding_box.get_or_init(|| {
      self.geometries.iter()
        .map(Geometry::bounding_box)
        .reduce(BoundingBox::join)
        .unwrap()
    })
  }

  fn block_at(&self, pos: IVec3) -> bool {
    self.bounding_box().contains(pos) &&
    self.geometries.par_iter().any(|geometry| geometry.block_at(pos))
  }
}

impl<G> Geometry for UnionThreaded<Vec<G>>
where G: Geometry + Sync {
  fn bounding_box(&self) -> BoundingBox {
    *self.bounding_box.get_or_init(|| {
      self.geometries.iter()
        .map(Geometry::bounding_box)
        .reduce(BoundingBox::join)
        .unwrap()
    })
  }

  fn block_at(&self, pos: IVec3) -> bool {
    self.bounding_box().contains(pos) &&
    self.geometries.par_iter().any(|geometry| geometry.block_at(pos))
  }
}

impl<G, const N: usize> MaterialGeometry for UnionThreaded<[G; N]>
where G: MaterialGeometry + Sync {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    if self.bounding_box().contains(pos) {
      self.geometries.par_iter()
        .find_map_first(|geometry| geometry.block_material_at(pos))
    } else {
      None
    }
  }
}

impl<G> MaterialGeometry for UnionThreaded<Vec<G>>
where G: MaterialGeometry + Sync {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    if self.bounding_box().contains(pos) {
      self.geometries.par_iter()
        .find_map_first(|geometry| geometry.block_material_at(pos))
    } else {
      None
    }
  }
}
