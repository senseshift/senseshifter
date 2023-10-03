pub trait Distance<T> {
  type Result;

  fn distance(&self, other: T) -> Self::Result;
}

pub mod to_point;
pub use to_point::*;