use glam::{DVec2, IVec2, IVec3, Vec3Swizzles};
use noise::{NoiseFn, MultiFractal, Fbm, Perlin};
use rand::Rng;

use crate::generation::{BoundingBox, Geometry};
use crate::utility::Grid;

use std::collections::VecDeque;

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
    self.shape.sample(xy)
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



#[derive(Debug, Clone)]
struct LandmassShape {
  grid: Grid<LandmassBlock>
}

impl LandmassShape {
  fn new(seed: u32, size: f64) -> Self {
    let grid = generate_landmass_shape(seed, size);
    LandmassShape { grid }
  }

  #[inline]
  fn sample(&self, pos: IVec2) -> bool {
    self.grid.get(pos).is_some()
  }

  #[inline]
  fn is_edge_at(&self, pos: IVec2) -> bool {
    self.grid.get(pos)
      .map_or(false, |block| block.edge)
  }

  #[inline]
  fn min(&self) -> IVec2 {
    self.grid.min()
  }

  #[inline]
  fn max(&self) -> IVec2 {
    self.grid.max()
  }
}

#[derive(Debug, Clone, Copy)]
struct LandmassBlock {
  edge: bool
}

const CARDINAL4: [IVec2; 4] = [
  glam::const_ivec2!([1, 0]),
  glam::const_ivec2!([0, 1]),
  glam::const_ivec2!([-1, 0]),
  glam::const_ivec2!([0, -1])
];

const CARDINAL8: [IVec2; 8] = [
  glam::const_ivec2!([1, 0]),
  glam::const_ivec2!([1, 1]),
  glam::const_ivec2!([0, 1]),
  glam::const_ivec2!([-1, 1]),
  glam::const_ivec2!([-1, 0]),
  glam::const_ivec2!([-1, -1]),
  glam::const_ivec2!([0, -1]),
  glam::const_ivec2!([1, -1])
];

fn generate_landmass_shape(seed: u32, size: f64) -> Grid<LandmassBlock> {
  discover(landmass_generator(seed, size.max(1.0), 128.0))
}

fn landmass_generator(seed: u32, size: f64, resolution: f64) -> impl NoiseFn<f64, 2> {
  Fbm::<Perlin>::new(seed)
    .set_octaves(8)
    .set_persistence(0.25)
    .multiply_constant(0.5)
    .scale_point_by(2.0)
    .add(OriginDistance::new(size))
    .scale_point_by(resolution.recip())
}

fn discover(noise: impl NoiseFn<f64, 2>) -> Grid<LandmassBlock> {
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  enum Value { Filled, Edge, EdgeOuter }

  let mut q = VecDeque::from([IVec2::ZERO]);
  let mut grid: Grid<Value> = Grid::new();
  while !q.is_empty() {
    let node = q.pop_front().unwrap();
    let value = noise.get(node.as_dvec2());
    grid.put_expand(node, match value.is_sign_positive() {
      true => Value::Filled,
      false => Value::Edge
    });

    if value.is_sign_positive() {
      for c in CARDINAL4.map(|c| c + node) {
        if !grid.contains(c) && !q.contains(&c) {
          q.push_back(c);
        };
      };
    };
  };

  // Discover all of the shape's edges and the most distant edge pixel
  let mut edges: Vec<IVec2> = Vec::new();
  let (edge_root, _) = grid.enumerate::<IVec2>()
    .filter_map(|(pos, value)| match *value {
      Value::Edge => Some((pos, pos.abs().max_element())),
      Value::Filled | Value::EdgeOuter => None
    })
    .inspect(|&(pos, _)| edges.push(pos))
    .max_by_key(|&(_, len)| len)
    .unwrap();

  // Discover the shape's outer edges
  let mut q = VecDeque::from([edge_root]);
  while !q.is_empty() {
    let node = q.pop_front().unwrap();
    grid.put(node, Value::EdgeOuter);
    for c in CARDINAL8.map(|c| c + node) {
      if grid.get(c) == Some(&Value::Edge) && !q.contains(&c) {
        q.push_back(c);
      };
    };
  };

  // Any edges that remain are on the interior of the noise's shape
  let mut q = edges.into_iter()
    .filter(|&pos| grid.get(pos) == Some(&Value::Edge))
    .collect::<VecDeque<IVec2>>();
  while !q.is_empty() {
    let node = q.pop_front().unwrap();
    grid.put(node, Value::Filled);
    for c in CARDINAL4.map(|c| c + node) {
      if grid.get(c) == None && !q.contains(&c) {
        q.push_back(c);
      };
    };
  };

  // Convert to a BoolGrid
  grid.into_enumerate::<[isize; 2]>()
    .map(|(pos, value)| (pos, match value {
      Value::Filled => LandmassBlock { edge: false },
      Value::Edge | Value::EdgeOuter => LandmassBlock { edge: true }
    }))
    .collect()
}

struct OriginDistance {
  offset: f64
}

impl OriginDistance {
  pub fn new(offset: f64) -> Self {
    OriginDistance { offset }
  }
}

impl NoiseFn<f64, 2> for OriginDistance {
  fn get(&self, point: impl Into<[f64; 2]>) -> f64 {
    let dist = DVec2::from(point.into()).length();
    (self.offset - dist).clamp(-1.0, 1.0)
  }
}

fn sample_checkered(size: u32, pos: IVec2) -> bool {
  let size = (size + 1) as i32;
  let xp = pos.x.rem_euclid(size * 2);
  let yp = pos.y.rem_euclid(size * 2);
  (xp == 0 && yp == 0) ||
  (xp - size == 0 && yp - size == 0)
}
