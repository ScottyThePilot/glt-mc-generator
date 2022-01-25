mod boolgrid;
mod grid;

use std::sync::atomic::{AtomicBool, Ordering};
use std::io;

pub use self::boolgrid::*;
pub use self::grid::*;



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
