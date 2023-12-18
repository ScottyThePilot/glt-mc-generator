#![forbid(unsafe_code)]
#![warn(
  future_incompatible,
  missing_copy_implementations,
  missing_debug_implementations,
  unreachable_pub
)]
extern crate ahash;
extern crate exgrid;
extern crate fastanvil;
extern crate fastnbt;
extern crate flate2;
extern crate glam;
extern crate itertools;
#[macro_use]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate thiserror;

#[macro_use]
mod macros;
pub mod geometry;
pub mod list_cache;

pub use glam::{I64Vec2 as Vec2, I64Vec3 as Vec3};
