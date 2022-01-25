pub mod bedrock;
pub mod blocks;
pub mod city;
pub mod intersection;
pub mod limit_bounds;
pub mod materialize;
pub mod ocean;
pub mod pillar;
pub mod union;

use glam::{IVec2, IVec3};
use pyo3::{Python, PyResult, PyObject};

use std::borrow::Cow;
use std::cmp::PartialOrd;



pub trait Geometry {
  fn bounding_box_guess(&self) -> BoundingBox;

  fn block_at(&self, pos: IVec3) -> bool;
}

pub trait MaterialGeometry: Geometry {
  fn block_material_at(&self, pos: IVec3) -> Option<Block>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Block {
  base_block: Cow<'static, str>,
  extra_block: Option<Cow<'static, str>>
}

impl Block {
  pub fn into_amulet_block(self, py: Python) -> PyResult<PyObject> {
    let amulet = py.import("amulet")?;
    let block_class = amulet.getattr("api")?.getattr("block")?.getattr("Block")?;
    let base_block = block_class.call_method1("from_string_blockstate", (self.base_block,))?;
    if let Some(extra_block) = self.extra_block {
      let extra_block = block_class.call_method1("from_string_blockstate", (extra_block,))?;
      // As far as I know, this does not break amulet
      base_block.setattr("_extra_blocks", (extra_block,))?;
    };

    Ok(base_block.into())
  }
}

impl From<&'static str> for Block {
  fn from(s: &'static str) -> Self {
    Block {
      base_block: Cow::Borrowed(s),
      extra_block: None
    }
  }
}

impl From<String> for Block {
  fn from(s: String) -> Self {
    Block {
      base_block: Cow::Owned(s),
      extra_block: None
    }
  }
}

impl<B, E> From<(B, E)> for Block
where B: Into<Cow<'static, str>>, E: Into<Cow<'static, str>> {
  fn from((base_block, extra_block): (B, E)) -> Self {
    Block {
      base_block: base_block.into(),
      extra_block: Some(extra_block.into())
    }
  }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
  pub min: IVec3,
  pub max: IVec3
}

impl BoundingBox {
  pub fn new(min: IVec3, max: IVec3) -> Self {
    let (min, max) = (IVec3::min(min, max), IVec3::max(min, max));
    BoundingBox { min, max }
  }

  pub fn join(self, other: Self) -> Self {
    let min = IVec3::min(self.min, other.min);
    let max = IVec3::max(self.max, other.max);
    BoundingBox { min, max }
  }

  pub fn in_chunk(self, chunk: IVec2) -> bool {
    let chunk_min = chunk * 16 + 0;
    let chunk_max = chunk * 16 + 15;
    let min = self.min.truncate();
    let max = self.max.truncate();

    boxes_intersect(min, max, chunk_min, chunk_max)
  }
}

fn boxes_intersect(min1: IVec2, max1: IVec2, min2: IVec2, max2: IVec2) -> bool {
  #[inline]
  fn value_in_range<T>(value: T, min: T, max: T) -> bool
  where T: std::cmp::PartialOrd {
    value >= min && value <= max
  }

  let x_overlap = value_in_range(min1.x, min2.x, max2.x) || value_in_range(min2.x, min1.x, max1.x);
  let y_overlap = value_in_range(min1.y, min2.y, max2.y) || value_in_range(min2.y, min1.y, max1.y);

  x_overlap && y_overlap
}



type AddConstant<Source> = noise::Add<f64, Source, noise::Constant, 2>;
type MultiplyConstant<Source> = noise::Multiply<f64, Source, noise::Constant, 2>;
