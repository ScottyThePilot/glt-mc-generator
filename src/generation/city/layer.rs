use glam::{IVec2, IVec3, Vec3Swizzles};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use super::building::Building;
use super::landmass_shape::*;
use crate::generation::blocks;
use crate::generation::pillar::Pillar;
use crate::generation::union::Union;
use crate::generation::union_threaded::UnionThreaded;
use crate::generation::{Block, BoundingBox, Geometry, MaterialGeometry};



pub const LANDMASS_THICKNESS: u32 = 5;
pub const PILLAR_RADIUS: u32 = 3;

#[derive(Debug, Clone)]
pub struct Layer {
  landmass: Landmass,
  pillars: Union<Vec<Pillar>>,
  buildings: UnionThreaded<Vec<Building>>,
  bounding_box: BoundingBox
}

impl Layer {
  pub fn generate_new<R: Rng>(source_rng: &mut R, top: i32, bottom: i32, size: f64) -> Self {
    //let shape = LandmassShape::generate_new(source_rng.gen(), size);
    let landmass = Landmass::generate_new(source_rng, top, size);

    let pillars = landmass.shape.generate_pillar_points().into_iter()
      .map(|origin| Pillar::new_bounded(origin, PILLAR_RADIUS, Some(bottom), Some(top)))
      .collect::<Vec<Pillar>>();

    let mut rng = Xoshiro256PlusPlus::from_rng(source_rng).unwrap();
    let buildings = landmass.shape.generate_building_shapes(&mut rng).into_iter()
      .map(|building_shape| Building::from_shape(building_shape, top, random_building_height(&mut rng)))
      .collect::<Vec<Building>>();

    let buildings_max_y = buildings.iter()
      .map(|building| building.top())
      .max().unwrap_or(top);
    let max = landmass.shape.max().extend(buildings_max_y);
    let min = landmass.shape.min().extend(bottom);
    let bounding_box = BoundingBox::new(min, max);

    Layer {
      landmass,
      pillars: Union::new(pillars),
      buildings: UnionThreaded::new(buildings),
      bounding_box
    }
  }

  /// Removes all buildings from this layer that collide with the pillars of another layer
  pub(super) fn remove_buildings_colliding_with(&mut self, above: &Layer) {
    self.buildings.retain(|building| {
      !above.pillars.iter().any(|pillar| do_geometries_intersect(building, pillar))
    })
  }
}

impl Geometry for Layer {
  fn bounding_box(&self) -> BoundingBox {
    self.bounding_box
  }

  fn block_at(&self, pos: IVec3) -> bool {
    self.landmass.block_at(pos) || self.pillars.block_at(pos) || self.buildings.block_at(pos)
  }
}

impl MaterialGeometry for Layer {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    ret_if_some!(self.landmass.block_at(pos).then(|| blocks::GRAY_CONCRETE));
    ret_if_some!(self.pillars.block_at(pos).then(|| blocks::GRAY_CONCRETE));
    ret_if_some!(self.buildings.block_at(pos).then(|| blocks::GRAY_CONCRETE));
    None
  }
}

fn do_geometries_intersect(g1: &impl Geometry, g2: &impl Geometry) -> bool {
  BoundingBox::intersects(g1.bounding_box(), g2.bounding_box())
}



#[derive(Debug, Clone)]
struct Landmass {
  shape: LandmassShape,
  level: i32
}

impl Landmass {
  fn generate_new<R: Rng>(source_rng: &mut R, level: i32, size: f64) -> Self {
    let shape = LandmassShape::generate_new(source_rng.gen(), size);
    Landmass { shape, level }
  }

  /// The z value at which the landmass' upper slab is located
  fn max_z(&self) -> i32 {
    self.level
  }

  /// The z value at which the landmass' lower slab is located
  fn min_z(&self) -> i32 {
    self.level - LANDMASS_THICKNESS as i32 + 1
  }
}

impl Geometry for Landmass {
  fn bounding_box(&self) -> BoundingBox {
    let min = self.shape.min().extend(self.min_z());
    let max = self.shape.max().extend(self.max_z());
    BoundingBox::new(min, max)
  }

  fn block_at(&self, pos: IVec3) -> bool {
    let max = self.max_z();
    let min = self.min_z();
    self.shape.sample(pos.xy()).is_some() && (
      (pos.z == min || pos.z == max) ||
      ((pos.z > min && pos.z < max) && (
        sample_checkered(2, pos.xy()) ||
        self.shape.is_edge_at(pos.xy())
      ))
    )
  }
}

fn sample_checkered(size: u32, pos: IVec2) -> bool {
  let size = (size + 1) as i32;
  let xp = pos.x.rem_euclid(size * 2);
  let yp = pos.y.rem_euclid(size * 2);
  (xp == 0 && yp == 0) || (xp - size == 0 && yp - size == 0)
}
