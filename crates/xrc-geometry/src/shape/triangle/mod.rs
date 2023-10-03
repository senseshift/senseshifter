use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use crate::*;
use getset::Getters;
use num::{Unsigned, Zero};
use nalgebra::*;

#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Triangle<T: Scalar>(pub Point2<T>, pub Point2<T>, pub Point2<T>);

impl<T> Debug for Triangle<T>
  where
    T: Scalar + Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Triangle")
      .field("0", &self.0)
      .field("1", &self.1)
      .field("2", &self.2)
      .finish()
  }
}

impl<T> Triangle<T>
  where
    T: Scalar,
{
  pub fn new(a: Point2<T>, b: Point2<T>, c: Point2<T>) -> Self {
    Self(a, b, c)
  }
}

impl<T> Hash for Triangle<T>
  where
    T: Scalar + Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.hash(state);
    self.1.hash(state);
    self.2.hash(state);
  }
}

impl<T> Copy for Triangle<T>
  where
    T: Scalar + Copy,
{}

impl<T> Clone for Triangle<T>
  where
    T: Scalar + Clone,
{
  fn clone(&self) -> Self {
    Self(self.0.clone(), self.1.clone(), self.2.clone())
  }
}

impl<T> PartialEq for Triangle<T>
  where
    T: Scalar + PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0 && self.1 == other.1 && self.2 == other.2
  }
}

impl<T> Eq for Triangle<T>
  where
    T: Scalar + Eq,
{}


impl Triangle<u8> {
  /// Returns the center of the triangle.
  ///
  /// # Example
  /// ```rust
  /// use xrc_geometry::{Point2, Triangle};
  ///
  /// let triangle = Triangle::new([0, 0].into(), [10, 0].into(), [0, 10].into());
  /// assert_eq!(triangle.center(), [3, 3].into());
  /// ```
  pub fn center(&self) -> Point2<u8> {
    let x_sum: f64 = self.0.x as f64 + self.1.x as f64 + self.2.x as f64;
    let y_sum: f64 = self.0.y as f64 + self.1.y as f64 + self.2.y as f64;

    Point2::new(
      (x_sum / 3.0).round() as u8,
      (y_sum / 3.0).round() as u8,
    )
  }

  pub fn bbox(&self) -> Rectangle<u8> {
    let min_x = self.0.x.min(self.1.x).min(self.2.x);
    let min_y = self.0.y.min(self.1.y).min(self.2.y);
    let max_x = self.0.x.max(self.1.x).max(self.2.x);
    let max_y = self.0.y.max(self.1.y).max(self.2.y);

    Rectangle::new(
      Point2::new(min_x, min_y),
      Point2::new(max_x, max_y),
    )
  }
}