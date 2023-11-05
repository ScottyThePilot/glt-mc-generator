pub mod mask;
pub mod unify;
pub mod world_data;

use std::ops::RangeInclusive;

use crate::{Vec2, Vec3};

use itertools::Itertools;
use itertools::structs::Product;



/// Represents a type that can hold and store world data.
pub trait GeometryReceiver {
  type Block;

  fn receive_block(&mut self, pos: Vec3, block: Self::Block);
  fn receive_material_geometry(&mut self, geometry: &impl MaterialGeometry<Block = Self::Block>) where Self: Sized {
    GeometryMaterializer::new(geometry).describe(self);
  }
}

impl<T: GeometryReceiver> GeometryReceiver for &mut T {
  type Block = <T as GeometryReceiver>::Block;

  #[inline]
  fn receive_block(&mut self, pos: Vec3, block: Self::Block) {
    T::receive_block(self, pos, block)
  }
}



/// A type of geometry that is exclusively constructed by sending blocks
/// to a [`GeometryReceiver`], which may then produce the world data.
///
/// [`GeometryDescriber`] should be preferred for features that 'know' the contents of themselves.
pub trait GeometryDescriber {
  type Block;

  fn describe(&self, reciever: &mut impl GeometryReceiver<Block = Self::Block>);
}

impl<T: GeometryDescriber> GeometryDescriber for &T {
  type Block = <T as GeometryDescriber>::Block;

  #[inline]
  fn describe(&self, reciever: &mut impl GeometryReceiver<Block = Self::Block>) {
    T::describe(self, reciever)
  }
}



/// A type of geometry who's state is made known by sampling the geometry at all relevant
/// points in space. [`Geometry`] in particular only represents a binary *present* or *not present*
/// and does not contain any other information about blocks at given points in space
/// ([`MaterialGeometry`], however, provides this).
///
/// [`Geometry`] should be preferred for features that *must* be
/// sampled at every point (such as those based on noise functions).
pub trait Geometry {
  fn bounding_box(&self) -> Option<BoundingBox3>;
  fn block_at(&self, pos: Vec3) -> bool;
}

impl<T: Geometry> Geometry for &T {
  #[inline]
  fn bounding_box(&self) -> Option<BoundingBox3> {
    T::bounding_box(self)
  }

  #[inline]
  fn block_at(&self, pos: Vec3) -> bool {
    T::block_at(self, pos)
  }
}

/// An extension of [`Geometry`] that gives block information to the information it returns.
pub trait MaterialGeometry: Geometry {
  type Block;

  fn block_material_at(&self, pos: Vec3) -> Option<Self::Block>;
}

impl<T: MaterialGeometry> MaterialGeometry for &T {
  type Block = <T as MaterialGeometry>::Block;

  #[inline]
  fn block_material_at(&self, pos: Vec3) -> Option<Self::Block> {
    T::block_material_at(self, pos)
  }
}



/// Wraps a [`MaterialGeometry`] to make it behave like a [`GeometryDescriber`].
pub struct GeometryMaterializer<G> {
  pub geometry: G
}

impl<G> GeometryMaterializer<G> {
  #[inline]
  pub fn new(geometry: G) -> Self {
    GeometryMaterializer { geometry }
  }
}

impl<G: MaterialGeometry> GeometryDescriber for GeometryMaterializer<G> {
  type Block = <G as MaterialGeometry>::Block;

  fn describe(&self, reciever: &mut impl GeometryReceiver<Block = Self::Block>) {
    if let Some(bounding_box) = self.geometry.bounding_box() {
      for pos in bounding_box {
        if let Some(block) = self.geometry.block_material_at(pos) {
          reciever.receive_block(pos, block);
        };
      };
    };
  }
}



pub type BoundingBox2 = BoundingBox<Vec2>;
pub type BoundingBox3 = BoundingBox<Vec3>;

/// A bounding box formed by two points, a `min` and `max`, inclusive.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox<V> {
  pub min: V,
  pub max: V
}

impl BoundingBox2 {
  pub fn new(p1: Vec2, p2: Vec2) -> Self {
    let (min, max) = (Vec2::min(p1, p2), Vec2::max(p1, p2));
    BoundingBox { min, max }
  }

  pub fn union(self, other: Self) -> Self {
    let min = Vec2::min(self.min, other.min);
    let max = Vec2::max(self.max, other.max);
    BoundingBox { min, max }
  }

  pub fn try_union(box1: Option<Self>, box2: Option<Self>) -> Option<Self> {
    try_combine(box1, box2, BoundingBox2::union)
  }

  pub fn intersect(self, other: Self) -> Option<Self> {
    if self.intersects_with(other) {
      let min = Vec2::max(self.min, other.min);
      let max = Vec2::min(self.max, other.max);
      Some(BoundingBox { min, max })
    } else {
      None
    }
  }

  pub fn contains(self, pos: Vec2) -> bool {
    contains(pos.x, self.min.x, self.max.x) &&
    contains(pos.y, self.min.y, self.max.y)

    //pos.x >= self.min.x && pos.x <= self.max.x &&
    //pos.y >= self.min.y && pos.y <= self.max.y
  }

  pub fn intersects_with(self, other: Self) -> bool {
    let x_overlap = axis_overlapping(&self, &other, |v| v.x);
    let y_overlap = axis_overlapping(&self, &other, |v| v.y);
    x_overlap && y_overlap
  }

  pub fn extend(self, min: i64, max: i64) -> BoundingBox3 {
    let (min, max) = (i64::min(min, max), i64::max(min, max));
    BoundingBox {
      min: self.min.extend(min),
      max: self.max.extend(max)
    }
  }
}

impl IntoIterator for BoundingBox2 {
  type Item = Vec2;
  type IntoIter = IterBoundingBox2;

  fn into_iter(self) -> Self::IntoIter {
    IterBoundingBox2::new(self.min.x..=self.max.x, self.min.y..=self.max.y)
  }
}

impl BoundingBox3 {
  pub fn new(min: Vec3, max: Vec3) -> Self {
    let (min, max) = (Vec3::min(min, max), Vec3::max(min, max));
    BoundingBox { min, max }
  }

  pub fn union(self, other: Self) -> Self {
    let min = Vec3::min(self.min, other.min);
    let max = Vec3::max(self.max, other.max);
    BoundingBox { min, max }
  }

  pub fn try_union(box1: Option<Self>, box2: Option<Self>) -> Option<Self> {
    try_combine(box1, box2, BoundingBox3::union)
  }

  pub fn intersect(self, other: Self) -> Option<Self> {
    if self.intersects_with(other) {
      let min = Vec3::max(self.min, other.min);
      let max = Vec3::min(self.max, other.max);
      Some(BoundingBox { min, max })
    } else {
      None
    }
  }

  pub fn crop(self, bounding_box: BoundingBox2) -> Option<Self> {
    //let min = Vec2::max(self.min.truncate(), other.min).extend(self.min.z);
    //let max = Vec2::min(self.max.truncate(), other.max).extend(self.max.z);
    self.truncate().intersect(bounding_box)
      .map(|bounding_box| bounding_box.extend(self.min.z, self.max.z))
  }

  pub fn contains(self, pos: Vec3) -> bool {
    contains(pos.x, self.min.x, self.max.x) &&
    contains(pos.y, self.min.y, self.max.y) &&
    contains(pos.z, self.min.z, self.max.z)

    //pos.x >= self.min.x && pos.x <= self.max.x &&
    //pos.y >= self.min.y && pos.y <= self.max.y &&
    //pos.z >= self.min.z && pos.z <= self.max.z
  }

  pub fn intersects_with(self, other: Self) -> bool {
    let x_overlap = axis_overlapping(&self, &other, |v| v.x);
    let y_overlap = axis_overlapping(&self, &other, |v| v.y);
    let z_overlap = axis_overlapping(&self, &other, |v| v.z);
    x_overlap && y_overlap && z_overlap
  }

  pub fn intersects_with_chunk(self, chunk: Vec2) -> bool {
    let chunk = BoundingBox2::new(chunk * 16 + 0, chunk * 16 + 15);
    self.truncate().intersects_with(chunk)
  }

  pub fn truncate(self) -> BoundingBox2 {
    BoundingBox {
      min: self.min.truncate(),
      max: self.max.truncate()
    }
  }
}

impl IntoIterator for BoundingBox3 {
  type Item = Vec3;
  type IntoIter = IterBoundingBox3;

  fn into_iter(self) -> Self::IntoIter {
    IterBoundingBox3::new(self.min.x..=self.max.x, self.min.y..=self.max.y, self.min.z..=self.max.z)
  }
}

fn axis_overlapping<V, T: PartialOrd>(b1: &BoundingBox<V>, b2: &BoundingBox<V>, f: fn(&V) -> T) -> bool {
  contains(f(&b1.min), f(&b2.min), f(&b2.max)) || contains(f(&b2.min), f(&b1.min), f(&b1.max))
}

#[inline]
fn contains<T: PartialOrd>(value: T, min: T, max: T) -> bool {
  value >= min && value <= max
}

fn try_combine<T>(a: Option<T>, b: Option<T>, f: fn(T, T) -> T) -> Option<T> {
  match (a, b) {
    (Some(a), Some(b)) => Some(f(a, b)),
    (Some(v), None) | (None, Some(v)) => Some(v),
    (None, None) => None
  }
}



#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct IterBoundingBox2 {
  inner: Product<RangeInclusive<i64>, RangeInclusive<i64>>
}

impl IterBoundingBox2 {
  pub fn new(x: RangeInclusive<i64>, y: RangeInclusive<i64>) -> Self {
    IterBoundingBox2 {
      inner: x.cartesian_product(y)
    }
  }
}

impl Iterator for IterBoundingBox2 {
  type Item = Vec2;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|(x, y)| Vec2::new(x, y))
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.inner.size_hint()
  }

  fn fold<A, F>(self, init: A, mut f: F) -> A
  where F: FnMut(A, Self::Item) -> A {
    self.inner.fold(init, |a, (x, y)| f(a, Vec2::new(x, y)))
  }
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct IterBoundingBox3 {
  inner: Product<RangeInclusive<i64>, IterBoundingBox2>
}

impl IterBoundingBox3 {
  pub fn new(x: RangeInclusive<i64>, y: RangeInclusive<i64>, z: RangeInclusive<i64>) -> Self {
    IterBoundingBox3 {
      // Z goes first because I say so
      inner: z.cartesian_product(IterBoundingBox2 {
        inner: x.cartesian_product(y)
      })
    }
  }
}

impl Iterator for IterBoundingBox3 {
  type Item = Vec3;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|(z, xy)| xy.extend(z))
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.inner.size_hint()
  }

  fn fold<A, F>(self, init: A, mut f: F) -> A
  where F: FnMut(A, Self::Item) -> A {
    self.inner.fold(init, |a, (z, xy)| f(a, xy.extend(z)))
  }
}
