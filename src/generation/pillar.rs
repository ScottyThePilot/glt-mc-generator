use super::{BoundingBox, Geometry};

use glam::{IVec2, IVec3, Vec3Swizzles};



#[derive(Debug, Clone)]
pub struct Pillar {
  min_height: i32,
  max_height: i32,
  origin: IVec2,
  radius: i32
}

impl Pillar {
  pub fn new(origin: IVec2, radius: i32) -> Self {
    Pillar {
      min_height: crate::WORLD_Z_MIN,
      max_height: crate::WORLD_Z_MAX,
      origin,
      radius
    }
  }

  pub fn new_bounded(origin: IVec2, radius: i32, min: Option<i32>, max: Option<i32>) -> Self {
    Pillar {
      min_height: min.unwrap_or(crate::WORLD_Z_MIN),
      max_height: max.unwrap_or(crate::WORLD_Z_MAX),
      origin,
      radius
    }
  }
}

impl Geometry for Pillar {
  fn bounding_box_guess(&self) -> BoundingBox {
    let o = IVec2::splat(self.radius + 1);
    let min = (self.origin - o).extend(self.min_height);
    let max = (self.origin + o).extend(self.max_height);
    BoundingBox::new(min, max)
  }

  fn block_at(&self, pos: IVec3) -> bool {
    let radius = self.radius as f32 + 0.5;
    (self.min_height..=self.max_height).contains(&pos.z) &&
    self.origin.as_vec2().distance(pos.xy().as_vec2()) <= radius
  }
}
