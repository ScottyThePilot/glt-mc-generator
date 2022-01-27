use glam::{DVec2, IVec2, Vec2};
use noise::{NoiseFn, MultiFractal, Fbm, Perlin};

use crate::utility::{Grid, cardinal4, cardinal8};

use std::collections::VecDeque;



#[derive(Debug, Clone)]
pub struct LandmassShape {
  grid: Grid<LandmassCell>
}

impl LandmassShape {
  pub fn new(seed: u32, size: f64) -> Self {
    let grid = generate_landmass_shape(seed, size);
    LandmassShape { grid }
  }

  #[inline]
  pub fn sample(&self, pos: IVec2) -> Option<LandmassCell> {
    self.grid.get(pos).copied()
  }

  #[inline]
  pub fn is_edge_at(&self, pos: IVec2) -> bool {
    self.grid.get(pos).map_or(false, |cell| cell.edge)
  }

  #[inline]
  pub fn min(&self) -> IVec2 {
    self.grid.min()
  }

  #[inline]
  pub fn max(&self) -> IVec2 {
    self.grid.max()
  }
}

#[derive(Debug, Clone, Copy)]
pub struct LandmassCell {
  /// An ordering value relating to the point's position along the edge of the shape.
  /// Nearby landmass cells will have similar values.
  pub ordering: usize,
  pub edge: bool
}

impl LandmassCell {
  pub const MAX_ORDERING: f32 = MAX_ORDERING;

  pub fn new(ordering: usize, edge: bool) -> Self {
    LandmassCell { ordering, edge }
  }
}



const MAX_ORDERING: f32 = u32::MAX as f32;

fn generate_landmass_shape(seed: u32, size: f64) -> Grid<LandmassCell> {
  discover(landmass_generator(seed, size.max(1.0), 128.0))
}

/// # Explanation
/// This function does almost all of the heavy lifting related to generating the shape of the landmasses.
///
/// It is first fed a noise function (code for which can be found below), which it flood-fill searches.
/// When it encounters a positive value, it sets it in the grid as 'present' and adds their neighbors to
/// the search queue while for negative values, it marks them as 'boundary' and does not add their
/// neighbors to the search queue.
///
/// Next, it loops over every element in the sparse grid, searching for the most distant 'boundary' element
/// ("distance" meaning manhattan distance from the origin) and collecting all 'boundary' elements into an
/// array.
///
/// Because the shape was flood-fill discovered in step 1, there cannot be any disconnected blobs outside
/// of the shape, and thus the most distant point must lie on the outer edge of the shape. The outer edge
/// is then flood-fill discovered, using the most distant point as the starting point. Each outer edge
/// element is set as a 'final boundary' because I'm terrible at naming things, and is also given an index.
///
/// At this point, all outer edges are marked 'final boundary' and any inner edges on any holes will be
/// marked 'boundary'. The array of elements from step 2 is then partitioned into two arrays, one with
/// all 'boundary' elements, and one with all 'final boundary' elements. The 'boundary' elements array
/// is consumed as the starting queue for a flood-fill that fills in all of the holes in the shape.
fn discover(noise: impl NoiseFn<f64, 2>) -> Grid<LandmassCell> {
  use std::f32::consts::{TAU, PI};

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  enum Value {
    Present,
    Boundary,
    BoundaryFinal {
      index: usize
    }
  }

  #[inline]
  fn boundary_at(grid: &Grid<Value>, pos: IVec2) -> bool {
    matches!(grid.get(pos), Some(&Value::Boundary))
  }

  // Discover the basic shape that the noise function produces
  let grid = {
    let mut q = VecDeque::from([IVec2::ZERO]);
    let mut grid: Grid<Value> = Grid::new();
    while let Some(pos) = q.pop_front() {
      let value = noise.get(pos.as_dvec2());
      if value > 0.0 {
        grid.put_expand(pos, Value::Present);
        for candidate in cardinal4(pos) {
          if !grid.contains(candidate) && !q.contains(&candidate) {
            q.push_back(candidate);
          };
        };
      } else {
        grid.put_expand(pos, Value::Boundary);
      };
    };

    grid
  };

  // Discover all of the shape's edges and the most distant edge element
  let (all_edges, outer_edge_root) = {
    let mut all_edges: Vec<IVec2> = Vec::new();
    let outer_edge_root = grid.enumerate::<IVec2>()
      .filter_map(|(pos, value)| match *value {
        Value::Present => None,
        Value::Boundary => Some(pos),
        Value::BoundaryFinal { .. } => unreachable!()
      })
      .inspect(|&pos| all_edges.push(pos))
      .max_by_key(|&pos| pos.abs().max_element())
      .expect("unreachable");
    println!("{} total edges, outer_edge_root: {}", all_edges.len(), outer_edge_root);
    (all_edges, outer_edge_root)
  };

  // Discover the shape's outer edges, replacing them
  // with `BoundaryFinal` and giving them an index number
  let grid = {
    let mut grid = grid;
    let mut index = 0;
    let mut q = VecDeque::from([outer_edge_root]);
    while let Some(pos) = q.pop_front() {
      grid.put(pos, Value::BoundaryFinal { index });
      for candidate in cardinal8(pos) {
        if boundary_at(&grid, candidate) && !q.contains(&candidate) {
          q.push_back(candidate);
          if index == 0 { break };
        };
      };

      index += 1;
    };

    grid
  };

  // Since any remaining `Boundary`s are edges on the interior,
  // flood fill the interior voids using them as a source
  let (grid, outer_edges) = {
    let mut grid = grid;
    let (q, outer_edges) = all_edges.into_iter()
      .partition::<Vec<IVec2>, _>(|&pos| boundary_at(&grid, pos));
    let mut q = VecDeque::from(q);
    while let Some(pos) = q.pop_front() {
      grid.put(pos, Value::Present);
      for candidate in cardinal4(pos) {
        if grid.get(candidate) == None && !q.contains(&candidate) {
          q.push_back(candidate);
        };
      };
    };

    let len = outer_edges.len();
    let outer_edges: Vec<_> = outer_edges.into_iter()
      .map(|outer_edge| {
        let index = match grid[outer_edge] {
          Value::BoundaryFinal { index } => index,
          _ => unreachable!()
        };

        let a = (index as f32 / len as f32) * TAU;
        (outer_edge, Vec2::new(a.cos(), a.sin()))
      })
      .collect();

    (grid, outer_edges)
  };

  fn get_ordering(outer_edges: &[(IVec2, Vec2)], pos: IVec2) -> usize {
    let mut totaled_vector = Vec2::ZERO;
    for &(outer_edge, vector) in outer_edges {
      let d = outer_edge.as_vec2().distance(pos.as_vec2());
      let d = d.powi(2).recip();
      totaled_vector += vector * d;
    };

    let a = f32::atan2(-totaled_vector.y, -totaled_vector.x);
    ((a + PI) / TAU * MAX_ORDERING).floor() as usize
  }

  #[inline]
  fn get_ordering_from_index(index: usize, len: usize) -> usize {
    (index as f32 / len as f32 * MAX_ORDERING).floor() as usize
  }

  grid.enumerate::<IVec2>()
    .map(|(pos, value)| (pos, match *value {
      Value::Present => {
        LandmassCell::new(get_ordering(&outer_edges, pos), false)
      },
      Value::Boundary => unreachable!(),
      Value::BoundaryFinal { index } => {
        LandmassCell::new(get_ordering_from_index(index, outer_edges.len()), true)
      }
    }))
    .collect()
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
