use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use crate::*;
use num::{Num, Zero};
use nalgebra::*;

pub struct Line<T>
  where
    T: Scalar,
{
  pub start: Point2<T>,
  pub end: Point2<T>,
}

impl<T> Debug for Line<T>
  where
    T: Scalar + Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Line")
      .field("start", &self.start)
      .field("end", &self.end)
      .finish()
  }
}

impl<T> Line<T>
  where
    T: Scalar + PartialOrd,
{
  pub fn new(a: Point2<T>, b: Point2<T>) -> Self {
    if a < b {
      Self { start: a, end: b }
    } else {
      Self { start: b, end: a }
    }
  }
}

impl<T> Line<T>
  where
    T: Scalar,
{
  pub fn new_unchecked(start: Point2<T>, end: Point2<T>) -> Self {
    Self { start, end }
  }
}
