//! This geometry module generates the following features:
//!
//! - An ocean spanning from y=0 downwards until it meets the sea floor.
//! - A seafloor that starts at roughly y=-32, with roughly 2 block of gravel and deepslate underneath.
//! - Randomly placed seagrass and tall seagrass on the gravel sea floor.
use super::{Block, BoundingBox, Geometry, MaterialGeometry};

use glam::{IVec2, IVec3, Vec3Swizzles};
use noise::{NoiseFn, MultiFractal, Fbm, Perlin};
use rand::Rng;



#[derive(Debug, Clone)]
pub struct Ocean {
  ocean1: OceanGenerator,
  ocean2: OceanGenerator,
  seagrass: SeagrassGenerator
}

impl Ocean {
  pub fn new<R: Rng>(source_rng: &mut R) -> Self {
    let seed = source_rng.gen();
    let ocean1 = OceanGenerator::new_v1(seed);
    let ocean2 = OceanGenerator::new_v2(seed);
    let seagrass = SeagrassGenerator::new(source_rng.gen());
    Ocean {
      ocean1,
      ocean2,
      seagrass
    }
  }

  fn sample_ocean1(&self, pos: IVec2) -> i32 {
    (self.ocean1.get(pos.as_dvec2()) - 32.0).floor() as i32
  }

  fn sample_ocean2(&self, pos: IVec2) -> i32 {
    (self.ocean2.get(pos.as_dvec2()) - 34.0).floor() as i32
  }

  fn sample_seagrass(&self, pos: IVec2) -> SeagrassPresence {
    self.seagrass.sample(pos.as_dvec2())
  }
}

impl Geometry for Ocean {
  fn bounding_box_guess(&self) -> BoundingBox {
    let min = IVec3::new(i32::MIN, i32::MIN, -64);
    let max = IVec3::new(i32::MAX, i32::MAX, 0);
    BoundingBox::new(min, max)
  }

  fn block_at(&self, pos: IVec3) -> bool {
    pos.z <= 0
  }
}

impl MaterialGeometry for Ocean {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    if pos.z > 0 { return None };
    let ocean1 = self.sample_ocean1(pos.xy());
    let ocean2 = self.sample_ocean2(pos.xy());
    if pos.z >= ocean1 {
      use SeagrassPresence::{Short, Tall};
      let seagrass = self.sample_seagrass(pos.xy());
      if seagrass == Short && pos.z == ocean1 {
        Some(super::blocks::SEAGRASS_SHORT)
      } else if seagrass == Tall && pos.z == ocean1 {
        Some(super::blocks::SEAGRASS_TALL_LOWER)
      } else if seagrass == Tall && pos.z == ocean1 + 1 {
        Some(super::blocks::SEAGRASS_TALL_UPPER)
      } else {
        Some(super::blocks::WATER)
      }
    } else if pos.z < ocean1 && pos.z >= ocean2 {
      Some(super::blocks::GRAVEL)
    } else if pos.z < ocean1 || pos.z < ocean2 {
      Some(super::blocks::DEEPSLATE)
    } else {
      None
    }
  }
}

#[derive(Debug, Clone)]
struct OceanGenerator {
  inner: noise::Multiply<f64, Fbm<Perlin>, noise::Constant, 2>
}

impl OceanGenerator {
  fn new_v1(seed: u32) -> Self {
    let inner = Fbm::new(seed)
      .set_octaves(5)
      .set_frequency(128f64.recip())
      .multiply_constant(4.0);
    OceanGenerator { inner }
  }

  fn new_v2(seed: u32) -> Self {
    let inner = Fbm::new(seed)
      .set_octaves(3)
      .set_frequency(128f64.recip())
      .multiply_constant(4.0);
    OceanGenerator { inner }
  }
}

impl NoiseFn<f64, 2> for OceanGenerator {
  #[inline]
  fn get(&self, point: impl Into<[f64; 2]>) -> f64 {
    self.inner.get(point)
  }
}

#[derive(Debug, Clone)]
struct SeagrassGenerator {
  inner: noise::ScalePoint<Perlin>
}

impl SeagrassGenerator {
  fn new(seed: u32) -> Self {
    const PHI: f64 = 1.61803398874989484820458683436563811;
    let inner = Perlin::new(seed);
    let inner = noise::ScalePoint::new(inner)
      .set_scale(PHI * 10.0);
    SeagrassGenerator {
      inner
    }
  }

  fn sample(&self, point: impl Into<[f64; 2]>) -> SeagrassPresence {
    let value = self.inner.get(point);
    let value = f64::floor((value + 1.0) * 100.0) as u32 % 10;
    match value {
      0..=5 => SeagrassPresence::None,
      6..=8 => SeagrassPresence::Short,
      9 => SeagrassPresence::Tall,
      _ => SeagrassPresence::None
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeagrassPresence {
  None,
  Short,
  Tall
}
