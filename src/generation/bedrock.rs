use glam::{IVec2, IVec3, Vec3Swizzles};
use noise::{NoiseFn, Perlin};
use rand::Rng;

use super::{Block, BoundingBox, Geometry, MaterialGeometry};



#[derive(Debug, Clone)]
pub struct Bedrock {
  inner: BedrockGenerator
}

impl Bedrock {
  pub fn new<R: Rng>(source_rng: &mut R) -> Self {
    let inner = BedrockGenerator::new(source_rng.gen());
    Bedrock { inner }
  }

  fn sample(&self, pos: IVec2) -> i32 {
    (self.inner.get(pos.as_dvec2()) - 64.0).floor() as i32
  }
}

impl Geometry for Bedrock {
  fn bounding_box(&self) -> BoundingBox {
    let min = IVec3::new(i32::MIN, i32::MIN, -64);
    let max = IVec3::new(i32::MAX, i32::MAX, -60);
    BoundingBox::new(min, max)
  }

  fn block_at(&self, pos: IVec3) -> bool {
    pos.z < -50 &&
    pos.z <= self.sample(pos.xy()).max(-64)
  }
}

impl MaterialGeometry for Bedrock {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    if self.block_at(pos) {
      Some(super::blocks::BEDROCK)
    } else {
      None
    }
  }
}



#[derive(Debug, Clone)]
struct BedrockGenerator {
  inner: noise::ScalePoint<super::MultiplyConstant<super::AddConstant<noise::Perlin>>>
}

impl BedrockGenerator {
  fn new(seed: u32) -> Self {
    const PHI: f64 = 1.61803398874989484820458683436563811;
    let inner = Perlin::new(seed)
      .add_constant(1.0)
      .multiply_constant(2.5)
      .scale_point_by(PHI * 10.0);
    BedrockGenerator { inner }
  }
}

impl NoiseFn<f64, 2> for BedrockGenerator {
  #[inline]
  fn get(&self, point: impl Into<[f64; 2]>) -> f64 {
    self.inner.get(point)
  }
}
