use std::fmt::Debug;
use std::hash::Hash;
use derivative::Derivative;
use crate::*;
use nalgebra::*;

#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative)]
#[derivative(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Line<T>
  where
    T: Scalar,
{
  pub start: Point2<T>,
  pub end: Point2<T>,
}

impl<T> Line<T>
  where
    T: Scalar,
{
  #[inline]
  pub fn new_unchecked(start: Point2<T>, end: Point2<T>) -> Self {
    Self { start, end }
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
