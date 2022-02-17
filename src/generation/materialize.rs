use glam::IVec3;

use super::{Block, BoundingBox, Geometry, MaterialGeometry};



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Materialize<G> {
  material: Block,
  geometry: G
}

impl<G> Materialize<G> {
  pub fn new<B>(material: B, geometry: G) -> Self
  where B: Into<Block> {
    Materialize {
      material: material.into(),
      geometry
    }
  }
}

impl<G> Geometry for Materialize<G>
where G: Geometry {
  #[inline]
  fn bounding_box(&self) -> BoundingBox {
    self.geometry.bounding_box()
  }

  #[inline]
  fn block_at(&self, pos: IVec3) -> bool {
    self.geometry.block_at(pos)
  }
}

impl<G> MaterialGeometry for Materialize<G>
where G: Geometry {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    if self.block_at(pos) {
      Some(self.material.clone())
    } else {
      None
    }
  }
}
