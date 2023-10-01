use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use crate::*;
use getset::Getters;
use num::Zero;

#[derive(Getters)]
pub struct Ellipse<T: Scalar> {
  #[getset(get = "pub")]
  pub(crate) center: Point2<T>,
  #[getset(get = "pub")]
  pub(crate) radius: (T, T),
}

impl<T> Default for Ellipse<T>
  where
    T: Scalar + Default + Zero,
{
  fn default() -> Self {
    Self {
      center: Point2::default(),
      radius: (T::default(), T::default()),
    }
  }
}

impl<T> Debug for Ellipse<T>
  where
    T: Scalar + Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Ellipse")
      .field("center", &self.center)
      .field("radius", &self.radius)
      .finish()
  }
}

impl<T> Ellipse<T>
  where
    T: Scalar,
{
  pub fn new(center: Point2<T>, radius: (T, T)) -> Self {
    Self { center, radius }
  }

  pub fn width(&self) -> &T {
    &self.radius.0
  }

  pub fn height(&self) -> &T {
    &self.radius.1
  }
}

impl<T> Hash for Ellipse<T>
  where
    T: Scalar + Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.center.hash(state);
    self.radius.hash(state);
  }
}

impl<T> Copy for Ellipse<T>
  where
    T: Scalar + Copy,
{}

impl<T> Clone for Ellipse<T>
  where
    T: Scalar + Clone,
{
  fn clone(&self) -> Self {
    Self {
      center: self.center.clone(),
      radius: self.radius.clone(),
    }
  }
}

impl<T> PartialEq for Ellipse<T>
  where
    T: Scalar + PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.center == other.center && self.radius == other.radius
  }
}

impl<T> Eq for Ellipse<T> where T: Scalar + Eq {}

#[cfg(test)]
mod tests {}