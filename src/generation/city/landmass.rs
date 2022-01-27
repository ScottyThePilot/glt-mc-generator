use glam::{IVec2, IVec3, Vec3Swizzles};
use rand::Rng;

use super::landmass_shape::*;
use crate::generation::{BoundingBox, Geometry};

pub const LANDMASS_THICKNESS: u32 = 5;



#[derive(Debug, Clone)]
pub struct Landmass {
  shape: LandmassShape,
  vertical_pos: i32
}

impl Landmass {
  pub fn new<R: Rng>(source_rng: &mut R, vertical_pos: i32, size: f64) -> Self {
    Landmass {
      shape: LandmassShape::new(source_rng.gen(), size),
      vertical_pos
    }
  }

  pub fn max_y(&self) -> i32 {
    self.slab_pos_upper()
  }

  pub fn min_y(&self) -> i32 {
    self.slab_pos_lower()
  }

  fn slab_pos_upper(&self) -> i32 {
    self.vertical_pos
  }

  fn slab_pos_lower(&self) -> i32 {
    self.vertical_pos - LANDMASS_THICKNESS as i32 + 1
  }

  #[inline]
  pub fn sample_z(&self, z: i32) -> bool {
    z == self.slab_pos_upper() || z == self.slab_pos_lower()
  }

  #[inline]
  pub fn sample_xy(&self, xy: IVec2) -> bool {
    self.shape.sample(xy).is_some()
  }
}

impl Geometry for Landmass {
  fn bounding_box_guess(&self) -> BoundingBox {
    let min = self.shape.min().extend(self.min_y());
    let max = self.shape.max().extend(self.max_y());
    BoundingBox::new(min, max)
  }

  fn block_at(&self, pos: IVec3) -> bool {
    let slab_upper = self.slab_pos_upper();
    let slab_lower = self.slab_pos_lower();
    self.sample_xy(pos.xy()) && (
      (pos.z == slab_lower || pos.z == slab_upper) ||
      ((slab_lower..=slab_upper).contains(&pos.z) && (
        sample_checkered(2, pos.xy()) || self.shape.is_edge_at(pos.xy())
      ))
    )
  }
}

fn sample_checkered(size: u32, pos: IVec2) -> bool {
  let size = (size + 1) as i32;
  let xp = pos.x.rem_euclid(size * 2);
  let yp = pos.y.rem_euclid(size * 2);
  (xp == 0 && yp == 0) ||
  (xp - size == 0 && yp - size == 0)
}
