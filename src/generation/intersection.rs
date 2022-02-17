use glam::IVec3;

use super::{Block, BoundingBox, Geometry, MaterialGeometry};



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intersect<G1, G2> {
  geometry1: G1,
  geometry2: G2
}

impl<G1, G2> Intersect<G1, G2> {
  pub fn new(geometry1: G1, geometry2: G2) -> Self {
    Intersect { geometry1, geometry2 }
  }
}

impl<G1, G2> Geometry for Intersect<G1, G2>
where
  G1: Geometry,
  G2: Geometry
{
  fn bounding_box(&self) -> BoundingBox {
    let b1 = self.geometry1.bounding_box();
    let b2 = self.geometry2.bounding_box();
    BoundingBox::join(b1, b2)
  }

  fn block_at(&self, pos: IVec3) -> bool {
    self.geometry1.block_at(pos) && self.geometry2.block_at(pos)
  }
}

impl<G1, G2> MaterialGeometry for Intersect<G1, G2>
where
  G1: MaterialGeometry,
  G2: Geometry
{
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    let block = self.geometry1.block_material_at(pos)?;
    if self.geometry2.block_at(pos) {
      Some(block)
    } else {
      None
    }
  }
}
