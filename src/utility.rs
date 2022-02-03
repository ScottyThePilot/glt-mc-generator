use glam::IVec2;

use std::sync::atomic::{AtomicBool, Ordering};
use std::io;



pub const CARDINAL4: [IVec2; 4] = [
  glam::const_ivec2!([1, 0]),
  glam::const_ivec2!([0, 1]),
  glam::const_ivec2!([-1, 0]),
  glam::const_ivec2!([0, -1])
];

pub const CARDINAL8: [IVec2; 8] = [
  glam::const_ivec2!([1, 0]),
  glam::const_ivec2!([1, 1]),
  glam::const_ivec2!([0, 1]),
  glam::const_ivec2!([-1, 1]),
  glam::const_ivec2!([-1, 0]),
  glam::const_ivec2!([-1, -1]),
  glam::const_ivec2!([0, -1]),
  glam::const_ivec2!([1, -1])
];

#[inline]
pub fn cardinal4(center: IVec2) -> impl Iterator<Item = IVec2> {
  CARDINAL4.into_iter().map(move |offset| offset + center)
}

#[inline]
pub fn cardinal8(center: IVec2) -> impl Iterator<Item = IVec2> {
  CARDINAL8.into_iter().map(move |offset| offset + center)
}



pub struct AtomicFlag(AtomicBool);

impl AtomicFlag {
  #[inline]
  pub fn new() -> Self {
    AtomicFlag(AtomicBool::new(false))
  }

  pub fn set(&self) {
    self.0.store(true, Ordering::Relaxed);
  }

  pub fn get(&self) -> bool {
    self.0.load(Ordering::Relaxed)
  }
}



pub trait Ignore {
  type Kind;
  type Output;

  fn ignore_err(self, kind: Self::Kind) -> Self::Output;
}

impl Ignore for io::Result<()> {
  type Kind = io::ErrorKind;
  type Output = io::Result<()>;

  fn ignore_err(self, kind: io::ErrorKind) -> io::Result<()> {
    match self {
      Ok(()) => Ok(()),
      Err(err) if err.kind() == kind => Ok(()),
      Err(err) => Err(err)
    }
  }
}



/// Creates a list of points along a square ring of a certain radius
pub fn ring(radius: usize) -> Vec<IVec2> {
  if radius == 0 { return vec![IVec2::ZERO] };
  let r = radius as i32;
  let mut pos = IVec2::new(-r, -r);
  let mut out = Vec::with_capacity(radius * 8);
  while pos.x < r { pos.x += 1; out.push(pos); };
  while pos.y < r { pos.y += 1; out.push(pos); };
  while pos.x > -r { pos.x -= 1; out.push(pos); };
  while pos.y > -r { pos.y -= 1; out.push(pos); };
  out
}
