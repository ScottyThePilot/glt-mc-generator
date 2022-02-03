use crate::generation::{BoundingBox, Geometry};

use glam::{IVec2, IVec3, Vec3Swizzles};



#[derive(Debug, Clone)]
pub struct Building {
  edge_min: IVec2,
  edge_max: IVec2,
  level: i32,
  height: u32
}

impl Building {
  pub fn new(edge1: IVec2, edge2: IVec2, level: i32, height: u32) -> Self {
    let edge_min = IVec2::min(edge1, edge2);
    let edge_max = IVec2::max(edge1, edge2);
    Building { edge_min, edge_max, level, height }
  }

  fn top(&self) -> i32 {
    self.level + self.height as i32
  }
}

impl Geometry for Building {
  fn bounding_box_guess(&self) -> BoundingBox {
    let min = self.edge_min.extend(self.level);
    let max = self.edge_max.extend(self.top());
    BoundingBox::new(min, max)
  }

  fn block_at(&self, pos: IVec3) -> bool {
    if (self.level..=self.top()).contains(&pos.z) {
      let matches_x = self.edge_min.x == pos.x || self.edge_max.x == pos.x;
      let matches_y = self.edge_min.y == pos.y || self.edge_max.y == pos.y;
      let within_x = pos.x >= self.edge_min.x && pos.x <= self.edge_max.x;
      let within_y = pos.y >= self.edge_min.y && pos.y <= self.edge_max.y;
      let z = pos.z - self.level;
      (matches_x && matches_y) ||
      (matches_x && within_y && !(pos.y.rem_euclid(2) == 0 && z.rem_euclid(2) == 0)) ||
      (matches_y && within_x && !(pos.x.rem_euclid(2) == 0 && z.rem_euclid(2) == 0))
    } else {
      false
    }
  }
}
