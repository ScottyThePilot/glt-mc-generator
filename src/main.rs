extern crate glam;
extern crate noise;
extern crate pyo3;
extern crate rand;
extern crate rand_xoshiro;

#[macro_use]
mod utility;
mod generation;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

use glam::{IVec2, IVec3, Vec3Swizzles};
use pyo3::prelude::*;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::generation::bedrock::Bedrock;
use crate::generation::city::City;
use crate::generation::limit_bounds::LimitBounds;
use crate::generation::ocean::Ocean;
use crate::generation::union::Union;
use crate::generation::{Block, BoundingBox, Geometry, MaterialGeometry};
use crate::utility::*;

const WORLD_MIN_Z: i32 = -64;
const WORLD_MAX_Z: i32 = WORLD_MIN_Z + 64 + 512;

#[derive(Debug, Clone)]
pub struct Generator {
  inner: LimitBounds<Union<(Bedrock, City, Ocean)>>,
  bounding_box: BoundingBox
}

impl Generator {
  fn new(seed: u64) -> Generator {
    let source_rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    let city = City::generate_new(source_rng, 3);
    let city_bounds = city.bounding_box();
    let city_bounds_min = city_bounds.min.xy() - 128;
    let city_bounds_max = city_bounds.max.xy() + 128;

    let mut source_rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    let bedrock = Bedrock::new(&mut source_rng);
    let ocean_floor = Ocean::new(&mut source_rng);

    let inner = Union::new((bedrock, city, ocean_floor));
    let inner = LimitBounds::new(inner, city_bounds_min, city_bounds_max);
    let bounding_box = inner.bounding_box();
    Generator { inner, bounding_box }
  }

  pub fn chunk_exists(&self, pos: IVec2) -> bool {
    self.bounding_box.in_chunk(pos)
  }

  pub fn block_at(&self, pos: IVec3) -> Option<Block> {
    self.inner.block_material_at(pos)
  }
}

fn get_level_path() -> PathBuf {
  #[cfg(debug_assertions)]
  if let Ok(location) = fs::read_to_string("debug-output-location.txt") {
    return Path::new(location.trim()).join("glt");
  };

  PathBuf::from("./output")
}

fn get_seed() -> u64 {
  std::env::args().nth(1)
    .and_then(|seed| seed.parse::<u64>().ok())
    .unwrap_or(0)
}

fn main() -> PyResult<()> {
  println!("generating features...");
  let generator = Generator::new(get_seed());

  let level_path = get_level_path();

  reset_level(&level_path)?;

  println!("rendering chunks...");
  Python::with_gil(|py| {
    disable_python_logging(py)?;
    let level = load_level(py, &level_path)?;
    render_chunks(py, &generator, level)
  })
}

// Steps through rings of chunks expanding out from 0,0 until a ring
// is reached where no chunks would be inside the generator's bounding box
fn render_chunks(py: Python, generator: &Generator, level: &PyAny) -> PyResult<()> {
  let chunks_pos_list = create_chunk_list(generator);
  let chunk_count = chunks_pos_list.len();
  for (i, chunk_pos) in chunks_pos_list.into_iter().enumerate() {
    let progress = (i + 1) as f32 / chunk_count as f32 * 100.0;
    println!("rendering chunk: {:>3}, {:>3}  {:>5.2}%", chunk_pos.x, chunk_pos.y, progress);
    render_chunk(py, &generator, &level, chunk_pos)?;
  };

  println!("saving chunks...");
  level.call_method0("save")?;
  level.call_method0("close")?;

  Ok(())
}

fn render_chunk(py: Python, generator: &Generator, level: &PyAny, chunk_pos: IVec2) -> PyResult<()> {
  let chunk = level.call_method1("create_chunk", (chunk_pos.x, chunk_pos.y, "minecraft:overworld"))?;
  let block_palette = chunk.getattr("block_palette")?;
  let mut block_list: HashMap<Block, usize> = HashMap::new();

  let min_z = generator.bounding_box.min.z;
  let max_z = generator.bounding_box.max.z;
  for block_pos in iter_chunk_blocks(min_z, max_z) {
    let global_pos = block_pos + (chunk_pos * 16).extend(0);
    let block = match generator.block_at(global_pos) {
      Some(block) => block,
      None => continue
    };

    // Bypasses a performance bottleneck within Amulet's `BlockManager.get_add_block`
    let block_num = match block_list.entry(block.clone()) {
      Entry::Occupied(entry) => *entry.get(),
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

/// Loads an `amulet.api.level.world.World` instance at the given path
fn load_level<'py>(py: Python<'py>, level_path: &Path) -> PyResult<&'py PyAny> {
  let amulet = py.import("amulet").expect("failed to import `amulet`");
  let anvil_format_class = amulet
    .getattr("level").expect("failed to import `amulet.level`")
    .getattr("formats").expect("failed to import `amulet.level.formats`")
    .getattr("anvil_world").expect("failed to import `amulet.level.formats.anvil_world`")
    .getattr("AnvilFormat").expect("failed to import `amulet.level.formats.anvil_world.AnvilWorld`");
  let world_class = amulet
    .getattr("api").expect("failed to import `amulet.api`")
    .getattr("level").expect("failed to import `amulet.api.level`")
    .getattr("world").expect("failed to import `amulet.api.level.world`")
    .getattr("World").expect("failed to import `amulet.api.level.world.World`");
  world_class.call1((level_path, anvil_format_class.call1((level_path,))?))
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

  fs::remove_dir_all(&path).ignore_err(io::ErrorKind::NotFound)?;
  fs::create_dir_all(&path)?;
  let datapacks_path = path.join("datapacks");
  fs::create_dir(&datapacks_path).ignore_err(io::ErrorKind::AlreadyExists)?;
  fs::write(datapacks_path.join("glt-mc-world-base.zip"), TEMPLATE_DATAPACKS_PACK)?;
  fs::write(path.join("icon.png"), TEMPLATE_ICON_PNG)?;
  fs::write(path.join("level.dat"), TEMPLATE_LEVEL_DAT)?;
  Ok(())
}

fn create_chunk_list(generator: &Generator) -> Vec<IVec2> {
  let mut list = Vec::new();
  for chunk_pos_list in (0..).map(crate::utility::ring) {
    let mut touched = false;
    for chunk_pos in chunk_pos_list {
      if generator.chunk_exists(chunk_pos) {
        list.push(chunk_pos);
        touched = true;
      };
    };

    if !touched {
      break;
    };
  };

  list
}

/// Iterates through every block in a chunk
fn iter_chunk_blocks(min_z: i32, max_z: i32) -> impl Iterator<Item = IVec3> {
  (min_z..=max_z).flat_map(|z| {
    (0..16).flat_map(move |x| {
      (0..16).map(move |y| {
        IVec3::new(x, y, z)
      })
    })
  })
}
