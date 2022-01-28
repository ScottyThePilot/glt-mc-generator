use glam::{IVec2, IVec3, Vec3Swizzles};
use rand::Rng;

use super::landmass_shape::*;
use crate::generation::{BoundingBox, Geometry};
use crate::generation::pillar::Pillar;



pub const LANDMASS_THICKNESS: u32 = 5;
pub const PILLAR_RADIUS: u32 = 3;

#[derive(Debug, Clone)]
pub struct Landmass {
  shape: LandmassShape,
  pillars: Vec<Pillar>,
  bottom: i32,
  top: i32,
}

impl Landmass {
  pub fn new<R: Rng>(source_rng: &mut R, top: i32, bottom: i32, size: f64) -> Self {
    let shape = LandmassShape::new(source_rng.gen(), size);
    let pillars = shape.generate_pillar_points().into_iter()
      .map(|origin| Pillar::new_bounded(origin, PILLAR_RADIUS, Some(bottom), Some(top)))
      .collect::<Vec<Pillar>>();
    Landmass { shape, pillars, top, bottom }
  }

  pub fn max_y(&self) -> i32 {
    self.slab_pos_upper()
  }

  pub fn min_y(&self) -> i32 {
    self.slab_pos_lower().min(self.bottom)
  }

  fn slab_pos_upper(&self) -> i32 {
    self.top
  }

  fn slab_pos_lower(&self) -> i32 {
    self.top - LANDMASS_THICKNESS as i32 + 1
  }

  fn sample_slab(&self, pos: IVec3) -> bool {
    let slab_upper = self.slab_pos_upper();
    let slab_lower = self.slab_pos_lower();
    self.shape.sample(pos.xy()).is_some() && (
      (pos.z == slab_lower || pos.z == slab_upper) ||
      ((slab_lower..=slab_upper).contains(&pos.z) && (
        sample_checkered(2, pos.xy()) || self.shape.is_edge_at(pos.xy())
      ))
    )
  }

  fn sample_pillars(&self, pos: IVec3) -> bool {
    self.pillars.iter().any(|pillar| pillar.block_at(pos))
  }
}

impl Geometry for Landmass {
  fn bounding_box_guess(&self) -> BoundingBox {
    let min = self.shape.min().extend(self.min_y());
    let max = self.shape.max().extend(self.max_y());
    BoundingBox::new(min, max)
  }

  fn block_at(&self, pos: IVec3) -> bool {
    self.sample_slab(pos) || self.sample_pillars(pos)
  }
}

fn sample_checkered(size: u32, pos: IVec2) -> bool {
  let size = (size + 1) as i32;
  let xp = pos.x.rem_euclid(size * 2);
  let yp = pos.y.rem_euclid(size * 2);
  (xp == 0 && yp == 0) ||
  (xp - size == 0 && yp - size == 0)
}
