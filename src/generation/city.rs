mod landmass;
mod landmass_shape;

use super::{Block, BoundingBox, Geometry, MaterialGeometry};
use self::landmass::Landmass;

use glam::IVec3;
use rand::Rng;



#[derive(Debug, Clone)]
pub struct City {
  landmass: Landmass
}

impl City {
  pub fn new<R: Rng>(source_rng: &mut R) -> Self {
    City {
      landmass: Landmass::new(source_rng, 48, -64, 1.0)
    }
  }
}

impl Geometry for City {
  fn bounding_box_guess(&self) -> BoundingBox {
    self.landmass.bounding_box_guess()
  }

  fn block_at(&self, pos: IVec3) -> bool {
    self.landmass.block_at(pos)
  }
}

impl MaterialGeometry for City {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    if self.block_at(pos) {
      Some(super::blocks::GRAY_CONCRETE)
    } else {
      None
    }
  }
}
