use crate::Vec3;
use crate::geometry::*;



pub struct Unify<T> {
  pub contents: T
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

      fn describe(&self, mut reciever: &mut impl GeometryReceiver<Block = X>) {
        let ($($g),*) = &self.contents;
        $($g.describe(&mut reciever);)*
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
