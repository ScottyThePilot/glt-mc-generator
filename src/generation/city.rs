mod building;
mod landmass_shape;
mod layer;

use std::iter::repeat_with;

use glam::IVec3;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use rayon::prelude::*;

use self::layer::Layer;
use super::{Block, BoundingBox, Geometry, MaterialGeometry};
use super::union::Union;



#[derive(Debug, Clone)]
pub struct City {
  layers: Union<Vec<Layer>>
}

impl City {
  pub fn generate_new<R: Rng>(mut source_rng: R, layer_count: usize) -> Self {
    let rngs = repeat_with(|| Xoshiro256PlusPlus::from_rng(&mut source_rng).unwrap())
      .take(layer_count).collect::<Vec<_>>();
    let mut layers = rngs.into_par_iter()
      .enumerate()
      .map(|(i, mut rng)| {
        let top = (i as i32 + 1) * 48;
        let bottom = if i == 0 { crate::WORLD_MIN_Z } else { i as i32 * 48 };
        let size = (layer_count - i) as f64;
        Layer::generate_new(&mut rng, top, bottom, size)
      })
      .collect::<Vec<Layer>>();
    windows_mut_each(&mut layers, |[ref mut below, ref above]| {
      below.remove_buildings_colliding_with(above);
    });

    City {
      layers: Union::new(layers)
    }
  }
}

impl Geometry for City {
  fn bounding_box(&self) -> BoundingBox {
    self.layers.bounding_box()
  }

  fn block_at(&self, pos: IVec3) -> bool {
    self.layers.block_at(pos)
  }
}

impl MaterialGeometry for City {
  fn block_material_at(&self, pos: IVec3) -> Option<Block> {
    self.layers.block_material_at(pos)
  }
}



fn windows_mut_each<T, F, const N: usize>(slice: &mut [T], mut f: F)
where F: FnMut(&mut [T; N]) {
  if slice.len() < N { return };
  for i in 0..(slice.len() + 1 - N) {
    match (&mut slice[i..(i + N)]).try_into() {
      Ok(window) => f(window),
      Err(err) => unreachable!("{}", err)
    };
  };
}
