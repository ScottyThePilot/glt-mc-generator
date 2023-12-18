use crate::Vec3;
use crate::geometry::*;



macro_rules! reverse_statements {
  ($stmt0:stmt;) => ($stmt0);
  ($stmt0:stmt; $($stmt:stmt;)*) => (reverse_statements!{$($stmt;)*} $stmt0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unify<T> {
  pub contents: T
}

impl<G: Geometry, const N: usize> Geometry for Unify<[G; N]> {
  #[inline]
  fn bounding_box(&self) -> Option<BoundingBox3> {
    unify_bounding_box(&self.contents)
  }

  #[inline]
  fn block_at(&self, pos: Vec3) -> bool {
    unify_block_at(&self.contents, pos)
  }
}

impl<G: MaterialGeometry, const N: usize> MaterialGeometry for Unify<[G; N]> {
  type Block = <G as MaterialGeometry>::Block;

  #[inline]
  fn block_material_at(&self, pos: Vec3) -> Option<Self::Block> {
    unify_block_material_at(&self.contents, pos)
  }
}

impl<G: GeometryDescriber, const N: usize> GeometryDescriber for Unify<[G; N]> {
  type Block = <G as GeometryDescriber>::Block;

  #[inline]
  fn describe<R>(&self, receiver: &mut R)
  where R: GeometryReceiver<Block = Self::Block> {
    unify_describe(&self.contents, receiver);
  }
}

impl<G: Geometry> Geometry for Unify<Vec<G>> {
  #[inline]
  fn bounding_box(&self) -> Option<BoundingBox3> {
    unify_bounding_box(&self.contents)
  }

  #[inline]
  fn block_at(&self, pos: Vec3) -> bool {
    unify_block_at(&self.contents, pos)
  }
}

impl<G: MaterialGeometry> MaterialGeometry for Unify<Vec<G>> {
  type Block = <G as MaterialGeometry>::Block;

  #[inline]
  fn block_material_at(&self, pos: Vec3) -> Option<Self::Block> {
    unify_block_material_at(&self.contents, pos)
  }
}

impl<G: GeometryDescriber> GeometryDescriber for Unify<Vec<G>> {
  type Block = <G as GeometryDescriber>::Block;

  #[inline]
  fn describe<R>(&self, receiver: &mut R)
  where R: GeometryReceiver<Block = Self::Block> {
    unify_describe(&self.contents, receiver);
  }
}

macro_rules! impl_unify_tuple {
  ($($g:ident $G:ident),* $(,)?) => {
    impl<$($G: Geometry,)*> Geometry for Unify<($($G,)*)> {
      fn bounding_box(&self) -> Option<BoundingBox3> {
        let ($($g),*) = &self.contents;
        coalesce!(BoundingBox3::try_union, $($g.bounding_box()),*)
      }

      fn block_at(&self, pos: Vec3) -> bool {
        let ($($g),*) = &self.contents;
        any!($($g.block_at(pos)),*)
      }
    }

    impl<X, $($G,)*> MaterialGeometry for Unify<($($G,)*)>
    where $($G: MaterialGeometry<Block = X>),* {
      type Block = X;

      fn block_material_at(&self, pos: Vec3) -> Option<X> {
        let ($($g),*) = &self.contents;
        $(if let Some(block) = $g.block_material_at(pos) { return Some(block) };)*
        None
      }
    }

    impl<X, $($G,)*> GeometryDescriber for Unify<($($G,)*)>
    where $($G: GeometryDescriber<Block = X>),* {
      type Block = X;

      fn describe<R>(&self, mut receiver: &mut R)
      where R: GeometryReceiver<Block = Self::Block> {
        let ($($g),*) = &self.contents;
        reverse_statements!{ $($g.describe(&mut receiver);)* }
      }
    }
  };
}

impl_unify_tuple!(a A, b B);
impl_unify_tuple!(a A, b B, c C);
impl_unify_tuple!(a A, b B, c C, d D);
impl_unify_tuple!(a A, b B, c C, d D, e E);
impl_unify_tuple!(a A, b B, c C, d D, e E, f F);
impl_unify_tuple!(a A, b B, c C, d D, e E, f F, g G);
impl_unify_tuple!(a A, b B, c C, d D, e E, f F, g G, h H);
impl_unify_tuple!(a A, b B, c C, d D, e E, f F, g G, h H, i I);
impl_unify_tuple!(a A, b B, c C, d D, e E, f F, g G, h H, i I, j J);

fn unify_bounding_box<G: Geometry>(contents: &[G]) -> Option<BoundingBox3> {
  contents.iter()
    .map(|geometry| geometry.bounding_box())
    .reduce(BoundingBox3::try_union)
    .flatten()
}

fn unify_block_at<G: Geometry>(contents: &[G], pos: Vec3) -> bool {
  contents.iter().any(|geometry| geometry.block_at(pos))
}

fn unify_block_material_at<G, B>(contents: &[G], pos: Vec3) -> Option<B>
where G: MaterialGeometry<Block = B> {
  contents.iter().find_map(|geometry| geometry.block_material_at(pos))
}

fn unify_describe<G, B, R>(contents: &[G], receiver: &mut R)
where G: GeometryDescriber<Block = B>, R: GeometryReceiver<Block = B> {
  // describers must run in reverse order
  for geometry in contents.iter().rev() {
    geometry.describe(receiver);
  };
}
