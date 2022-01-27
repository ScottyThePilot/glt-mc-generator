extern crate glam;
extern crate dirs;
extern crate noise;
extern crate pyo3;
extern crate rand;
extern crate rand_xoshiro;

mod generation;
mod utility;

use glam::{IVec3, IVec2, Vec3Swizzles};
use pyo3::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use std::fs;
use std::io;
use std::path::Path;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::generation::{Block, Geometry, MaterialGeometry};
use crate::generation::bedrock::Bedrock;
use crate::generation::city::City;
use crate::generation::limit_bounds::LimitBounds;
use crate::generation::ocean::Ocean;
use crate::generation::union::Union;
use crate::utility::*;

const WORLD_Z_MIN: i32 = -64;
const WORLD_Z_MAX: i32 = WORLD_Z_MIN + 64 + 512;

#[derive(Debug, Clone)]
pub struct Generator {
  inner: LimitBounds<Union<(
    Bedrock,
    City,
    Ocean
  )>>
}

impl Generator {
  fn new(seed: u64) -> Generator {
    let mut source_rng = Xoshiro256PlusPlus::seed_from_u64(seed);

    let bedrock = Bedrock::new(&mut source_rng);
    let ocean_floor = Ocean::new(&mut source_rng);
    let city = City::new(&mut source_rng, 64);

    let city_bounds = city.bounding_box_guess();
    let city_bounds_min = city_bounds.min.xy();
    let city_bounds_max = city_bounds.max.xy();

    let inner = Union::new((bedrock, city, ocean_floor));
    let inner = LimitBounds::new(inner, city_bounds_min, city_bounds_max);
    Generator { inner }
  }

  pub fn chunk_exists(&self, pos: IVec2) -> bool {
    self.inner.bounding_box_guess().in_chunk(pos)
  }

  pub fn block_at(&self, pos: IVec3) -> Option<Block> {
    self.inner.block_material_at(pos)
  }
}

fn main() -> PyResult<()> {
  let seed = std::env::args().nth(1)
    .and_then(|seed| seed.parse::<u64>().ok())
    .unwrap_or(0);
  #[cfg(debug_assertions)]
  let level_path = dirs::home_dir()
    .ok_or(io::Error::new(io::ErrorKind::Other, "no home dir"))?
    .join("AppData/Roaming/.minecraft/saves/glt");
  #[cfg(not(debug_assertions))]
  let level_path = std::path::PathBuf::from("./output");

  let generator = Generator::new(seed);

  println!("bounding box: {:?}", generator.inner.bounding_box_guess());

  Python::with_gil(|py| {
    disable_python_logging(py)?;

    let amulet = py.import("amulet")?;

    reset_level(&level_path)?;
    let level = amulet.call_method1("load_level", (&level_path,))?;

    for chunk_pos_list in (0..).map(crate::utility::ring) {
      let touched = AtomicFlag::new();
      for chunk_pos in chunk_pos_list {
        if generator.chunk_exists(chunk_pos) {
          touched.set();
          println!("generating chunk: ({:>3}, {:>3})", chunk_pos.x, chunk_pos.y);
          generate_chunk(py, &generator, &level, chunk_pos)?;
        };
      };

      if touched.get() {
        println!("saving chunks");
        level.call_method0("save")?;
      } else {
        break;
      };
    };

    level.call_method0("close")?;

    Ok(())
  })
}

fn generate_chunk(py: Python, generator: &Generator, level: &PyAny, chunk_pos: IVec2) -> PyResult<()> {
  let chunk = level.call_method1("create_chunk", (chunk_pos.x, chunk_pos.y, "minecraft:overworld"))?;
  let block_palette = chunk.getattr("block_palette")?;
  let mut block_list: HashMap<Block, usize> = HashMap::new();

  for block_pos in iter_chunk_blocks() {
    let global_pos = block_pos + (chunk_pos * 16).extend(0);
    let block = match generator.block_at(global_pos) {
      Some(block) => block,
      None => continue
    };

    // Bypasses a performance bottleneck within Amulet's `BlockManager.get_add_block`
    let block_num = match block_list.entry(block.clone()) {
      Entry::Occupied(entry) => {
        *entry.get()
      },
      Entry::Vacant(entry) => {
        let amulet_block = block.into_amulet_block(py)?;
        let block_num = block_palette
          .call_method1("get_add_block", (amulet_block,))?
          .extract::<usize>()?;
        entry.insert(block_num);
        block_num
      }
    };

    let pos: (i32, i32, i32) = block_pos.xzy().into();
    chunk.getattr("blocks")?.set_item(pos, block_num)?;
  };

  Ok(())
}

/// Disables all python logging
fn disable_python_logging(py: Python) -> PyResult<()> {
  let logging = py.import("logging")?;
  logging.call_method1("disable", (logging.getattr("WARNING")?,))?;
  Ok(())
}

/// Creates a new template world at the given path, ready for amulet to load
fn reset_level(path: &Path) -> io::Result<()> {
  const TEMPLATE_DATAPACKS_PACK: &[u8] = include_bytes!("../world-template/datapacks/world-size.zip");
  const TEMPLATE_ICON_PNG: &[u8] = include_bytes!("../world-template/icon.png");
  const TEMPLATE_LEVEL_DAT: &[u8] = include_bytes!("../world-template/level.dat");

  fs::remove_dir_all(&path)
    .ignore_err(io::ErrorKind::NotFound)?;
  fs::create_dir_all(&path)?;
  let datapacks_path = path.join("datapacks");
  fs::create_dir(&datapacks_path)
    .ignore_err(io::ErrorKind::AlreadyExists)?;
  fs::write(datapacks_path.join("glt-mc-world-base.zip"), TEMPLATE_DATAPACKS_PACK)?;
  fs::write(path.join("icon.png"), TEMPLATE_ICON_PNG)?;
  fs::write(path.join("level.dat"), TEMPLATE_LEVEL_DAT)?;
  Ok(())
}

/// Iterates through every block in a chunk
fn iter_chunk_blocks() -> impl Iterator<Item = IVec3> {
  (WORLD_Z_MIN..WORLD_Z_MAX).flat_map(|z| {
    (0..16).flat_map(move |x| {
      (0..16).map(move |y| {
        IVec3::new(x, y, z)
      })
    })
  })
}
