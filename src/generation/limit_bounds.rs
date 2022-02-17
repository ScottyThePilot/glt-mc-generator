use glam::{IVec2, IVec3};

use super::{Block, BoundingBox, Geometry, MaterialGeometry};



#[derive(Debug, Clone)]
pub struct LimitBounds<G> {
  geometry: G,
  bounds_min: IVec2,
  bounds_max: IVec2
}

impl<G> LimitBounds<G> {
  pub fn new(geometry: G, min: IVec2, max: IVec2) -> Self {
    LimitBounds {
      geometry,
      bounds_min: min,
      bounds_max: max
    }
  }
}

impl<G> Geometry for LimitBounds<G>
where G: Geometry {
  fn bounding_box(&self) -> BoundingBox {
    let mut bounding_box = self.geometry.bounding_box();
    bounding_box.min.x = bounding_box.min.x.max(self.bounds_min.x);
    bounding_box.min.y = bounding_box.min.y.max(self.bounds_min.y);
    bounding_box.max.x = bounding_box.max.x.min(self.bounds_max.x);
    bounding_box.max.y = bounding_box.max.y.min(self.bounds_max.y);
    bounding_box
  }

  #[inline]
  fn block_at(&self, pos: IVec3) -> bool {
    self.geometry.block_at(pos)
  }
}

impl<G> MaterialGeometry for LimitBounds<G>
where G: MaterialGeometry {
  #[inline]
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    self.geometry.block_material_at(pos)
  }
}
