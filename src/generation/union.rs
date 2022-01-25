use super::{Block, BoundingBox, Geometry, MaterialGeometry};

use glam::IVec3;

macro_rules! out {
  ($expr:expr) => (if let Some(value) = $expr { return Some(value); });
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Union<G> {
  geometries: G
}

impl<G> Union<G> {
  pub fn new(geometries: G) -> Self {
    Union { geometries }
  }
}

impl<G1, G2> Geometry for Union<(G1, G2)>
where G1: Geometry, G2: Geometry {
  fn bounding_box_guess(&self) -> BoundingBox {
    let (g1, g2) = &self.geometries;
    let b1 = g1.bounding_box_guess();
    let b2 = g2.bounding_box_guess();
    BoundingBox::join(b1, b2)
  }

  fn block_at(&self, pos: IVec3) -> bool {
    let (g1, g2) = &self.geometries;
    g1.block_at(pos) || g2.block_at(pos)
  }
}

impl<G1, G2, G3> Geometry for Union<(G1, G2, G3)>
where G1: Geometry, G2: Geometry, G3: Geometry {
  fn bounding_box_guess(&self) -> BoundingBox {
    let (g1, g2, g3) = &self.geometries;
    let b1 = g1.bounding_box_guess();
    let b2 = g2.bounding_box_guess();
    let b3 = g3.bounding_box_guess();
    [b1, b2, b3].into_iter()
      .reduce(BoundingBox::join)
      .unwrap()
  }

  fn block_at(&self, pos: IVec3) -> bool {
    let (g1, g2, g3) = &self.geometries;
    g1.block_at(pos) || g2.block_at(pos) || g3.block_at(pos)
  }
}

impl<G1, G2, G3, G4> Geometry for Union<(G1, G2, G3, G4)>
where G1: Geometry, G2: Geometry, G3: Geometry, G4: Geometry {
  fn bounding_box_guess(&self) -> BoundingBox {
    let (g1, g2, g3, g4) = &self.geometries;
    let b1 = g1.bounding_box_guess();
    let b2 = g2.bounding_box_guess();
    let b3 = g3.bounding_box_guess();
    let b4 = g4.bounding_box_guess();
    [b1, b2, b3, b4].into_iter()
      .reduce(BoundingBox::join)
      .unwrap()
  }

  fn block_at(&self, pos: IVec3) -> bool {
    let (g1, g2, g3, g4) = &self.geometries;
    g1.block_at(pos) || g2.block_at(pos) || g3.block_at(pos) || g4.block_at(pos)
  }
}

impl<G1, G2, G3, G4, G5> Geometry for Union<(G1, G2, G3, G4, G5)>
where G1: Geometry, G2: Geometry, G3: Geometry, G4: Geometry, G5: Geometry {
  fn bounding_box_guess(&self) -> BoundingBox {
    let (g1, g2, g3, g4, g5) = &self.geometries;
    let b1 = g1.bounding_box_guess();
    let b2 = g2.bounding_box_guess();
    let b3 = g3.bounding_box_guess();
    let b4 = g4.bounding_box_guess();
    let b5 = g5.bounding_box_guess();
    [b1, b2, b3, b4, b5].into_iter()
      .reduce(BoundingBox::join)
      .unwrap()
  }

  fn block_at(&self, pos: IVec3) -> bool {
    let (g1, g2, g3, g4, g5) = &self.geometries;
    g1.block_at(pos) || g2.block_at(pos) || g3.block_at(pos) || g4.block_at(pos) | g5.block_at(pos)
  }
}

impl<G, const N: usize> Geometry for Union<[G; N]>
where G: Geometry {
  fn bounding_box_guess(&self) -> BoundingBox {
    self.geometries.iter()
      .map(Geometry::bounding_box_guess)
      .reduce(BoundingBox::join)
      .unwrap()
  }

  fn block_at(&self, pos: IVec3) -> bool {
    self.geometries.iter()
      .any(|geometry| geometry.block_at(pos))
  }
}

impl<G> Geometry for Union<Vec<G>>
where G: Geometry {
  fn bounding_box_guess(&self) -> BoundingBox {
    self.geometries.iter()
      .map(Geometry::bounding_box_guess)
      .reduce(BoundingBox::join)
      .unwrap()
  }

  fn block_at(&self, pos: IVec3) -> bool {
    self.geometries.iter()
      .any(|geometry| geometry.block_at(pos))
  }
}

impl<G1, G2> MaterialGeometry for Union<(G1, G2)>
where G1: MaterialGeometry, G2: MaterialGeometry {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    let (g1, g2) = &self.geometries;
    out!(g1.block_material_at(pos));
    out!(g2.block_material_at(pos));
    None
  }
}

impl<G1, G2, G3> MaterialGeometry for Union<(G1, G2, G3)>
where G1: MaterialGeometry, G2: MaterialGeometry, G3: MaterialGeometry {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    let (g1, g2, g3) = &self.geometries;
    out!(g1.block_material_at(pos));
    out!(g2.block_material_at(pos));
    out!(g3.block_material_at(pos));
    None
  }
}

impl<G1, G2, G3, G4> MaterialGeometry for Union<(G1, G2, G3, G4)>
where G1: MaterialGeometry, G2: MaterialGeometry, G3: MaterialGeometry, G4: MaterialGeometry {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    let (g1, g2, g3, g4) = &self.geometries;
    out!(g1.block_material_at(pos));
    out!(g2.block_material_at(pos));
    out!(g3.block_material_at(pos));
    out!(g4.block_material_at(pos));
    None
  }
}

impl<G1, G2, G3, G4, G5> MaterialGeometry for Union<(G1, G2, G3, G4, G5)>
where G1: MaterialGeometry, G2: MaterialGeometry, G3: MaterialGeometry, G4: MaterialGeometry, G5: MaterialGeometry {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    let (g1, g2, g3, g4, g5) = &self.geometries;
    out!(g1.block_material_at(pos));
    out!(g2.block_material_at(pos));
    out!(g3.block_material_at(pos));
    out!(g4.block_material_at(pos));
    out!(g5.block_material_at(pos));
    None
  }
}

impl<G, const N: usize> MaterialGeometry for Union<[G; N]>
where G: MaterialGeometry {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    self.geometries.iter()
      .find_map(|geometry| geometry.block_material_at(pos))
  }
}

impl<G> MaterialGeometry for Union<Vec<G>>
where G: MaterialGeometry {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    self.geometries.iter()
      .find_map(|geometry| geometry.block_material_at(pos))
  }
}
