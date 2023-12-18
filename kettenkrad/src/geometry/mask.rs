use crate::Vec3;
use crate::geometry::*;



pub type MaskBox2<G> = Mask<G, BoundingBox2>;
pub type MaskBox3<G> = Mask<G, BoundingBox3>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mask<G, M> {
  pub geometry: G,
  pub mask: M
}

impl<G, M> Mask<G, M> {
  pub const fn new(geometry: G, mask: M) -> Self {
    Mask { geometry, mask }
  }
}

impl<G, M> GeometryDescriber for Mask<G, M>
where G: GeometryDescriber, M: Geometry {
  type Block = <G as GeometryDescriber>::Block;

  #[inline]
  fn describe<R>(&self, receiver: &mut R) where R: GeometryReceiver<Block = Self::Block> {
    self.geometry.describe(&mut GeometryReceiverMasked::new(receiver, &self.mask));
  }
}

impl<G, M> Geometry for Mask<G, M>
where G: Geometry, M: Geometry {
  fn bounding_box(&self) -> Option<BoundingBox3> {
    let bounding_box_mask = self.mask.bounding_box()?;
    let bounding_box_geometry = self.geometry.bounding_box()?;
    bounding_box_mask.intersect(bounding_box_geometry)
  }

  fn block_at(&self, pos: Vec3) -> bool {
    self.mask.block_at(pos) && self.geometry.block_at(pos)
  }
}

impl<G, M> MaterialGeometry for Mask<G, M>
where G: MaterialGeometry, M: Geometry {
  type Block = <G as MaterialGeometry>::Block;

  fn block_material_at(&self, pos: Vec3) -> Option<Self::Block> {
    if self.mask.block_at(pos) {
      self.geometry.block_material_at(pos)
    } else {
      None
    }
  }
}

impl<G: GeometryDescriber> GeometryDescriber for Mask<G, BoundingBox2> {
  type Block = <G as GeometryDescriber>::Block;

  #[inline]
  fn describe<R>(&self, receiver: &mut R) where R: GeometryReceiver<Block = Self::Block> {
    self.geometry.describe(&mut GeometryReceiverMasked::new(receiver, self.mask));
  }
}

impl<G: Geometry> Geometry for Mask<G, BoundingBox2> {
  fn bounding_box(&self) -> Option<BoundingBox3> {
    self.geometry.bounding_box().and_then(|bounding_box| bounding_box.crop(self.mask))
  }

  #[inline]
  fn block_at(&self, pos: Vec3) -> bool {
    self.geometry.block_at(pos)
  }
}

impl<G: MaterialGeometry> MaterialGeometry for Mask<G, BoundingBox2> {
  type Block = <G as MaterialGeometry>::Block;

  #[inline]
  fn block_material_at(&self, pos: Vec3) -> Option<Self::Block> {
    self.geometry.block_material_at(pos)
  }
}

impl<G: GeometryDescriber> GeometryDescriber for Mask<G, BoundingBox3> {
  type Block = <G as GeometryDescriber>::Block;

  #[inline]
  fn describe<R>(&self, receiver: &mut R) where R: GeometryReceiver<Block = Self::Block> {
    self.geometry.describe(&mut GeometryReceiverMasked::new(receiver, self.mask));
  }
}

impl<G: Geometry> Geometry for Mask<G, BoundingBox3> {
  fn bounding_box(&self) -> Option<BoundingBox3> {
    self.geometry.bounding_box().and_then(|bounding_box| bounding_box.intersect(self.mask))
  }

  #[inline]
  fn block_at(&self, pos: Vec3) -> bool {
    self.geometry.block_at(pos)
  }
}

impl<G: MaterialGeometry> MaterialGeometry for Mask<G, BoundingBox3> {
  type Block = <G as MaterialGeometry>::Block;

  #[inline]
  fn block_material_at(&self, pos: Vec3) -> Option<Self::Block> {
    self.geometry.block_material_at(pos)
  }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeometryReceiverMasked<R, M> {
  pub receiver: R,
  pub mask: M
}

impl<R, M> GeometryReceiverMasked<R, M> {
  pub const fn new(receiver: R, mask: M) -> Self {
    GeometryReceiverMasked { receiver, mask }
  }
}

impl<R, M> GeometryReceiver for GeometryReceiverMasked<R, M>
where R: GeometryReceiver, M: Geometry {
  type Block = <R as GeometryReceiver>::Block;

  fn receive_block(&mut self, pos: Vec3, block: Self::Block) {
    if self.mask.block_at(pos) {
      self.receiver.receive_block(pos, block);
    };
  }
}

impl<R> GeometryReceiver for GeometryReceiverMasked<R, BoundingBox2>
where R: GeometryReceiver {
  type Block = <R as GeometryReceiver>::Block;

  fn receive_block(&mut self, pos: Vec3, block: Self::Block) {
    if self.mask.contains(pos.truncate()) {
      self.receiver.receive_block(pos, block);
    };
  }
}

impl<R> GeometryReceiver for GeometryReceiverMasked<R, BoundingBox3>
where R: GeometryReceiver {
  type Block = <R as GeometryReceiver>::Block;

  fn receive_block(&mut self, pos: Vec3, block: Self::Block) {
    if self.mask.contains(pos) {
      self.receiver.receive_block(pos, block);
    };
  }
}
