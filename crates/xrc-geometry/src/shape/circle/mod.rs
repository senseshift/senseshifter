use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use crate::*;
use getset::Getters;
use num::{Unsigned, Zero};
use nalgebra::*;

#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Getters)]
pub struct Circle<T: Scalar, R: Scalar + Unsigned> {
  #[getset(get = "pub")]
  pub(crate) center: Point2<T>,
  #[getset(get = "pub")]
  pub(crate) radius: R,
}

impl<T, R> Default for Circle<T, R>
  where
    T: Scalar + Default + Zero,
    R: Scalar + Unsigned + Default,
{
  fn default() -> Self {
    Self {
      center: Point2::default(),
      radius: R::default(),
    }
  }
}

impl<T, R> Debug for Circle<T, R>
  where
    T: Scalar + Debug,
    R: Scalar + Unsigned + Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Circle")
      .field("center", &self.center)
      .field("radius", &self.radius)
      .finish()
  }
}

impl<T, R> Circle<T, R>
  where
    T: Scalar,
    R: Scalar + Unsigned,
{
  pub fn new(center: Point2<T>, radius: R) -> Self {
    Self { center, radius }
  }
}

impl<T, R> Hash for Circle<T, R>
  where
    T: Scalar + Hash,
    R: Scalar + Unsigned + Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.center.hash(state);
    self.radius.hash(state);
  }
}

impl<T, R> Copy for Circle<T, R>
  where
    T: Scalar + Copy,
    R: Scalar + Unsigned + Copy,
{}

impl<T, R> Clone for Circle<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned + Clone,
{
  fn clone(&self) -> Self {
    Self {
      center: self.center.clone(),
      radius: self.radius.clone(),
    }
  }
}

impl<T, R> PartialEq for Circle<T, R>
  where
    T: Scalar + PartialEq,
    R: Scalar + Unsigned + PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.center == other.center && self.radius == other.radius
  }
}

impl<T, R> Eq for Circle<T, R>
  where
    T: Scalar + Eq,
    R: Scalar + Unsigned + Eq,
{}

#[cfg(test)]
mod tests {}