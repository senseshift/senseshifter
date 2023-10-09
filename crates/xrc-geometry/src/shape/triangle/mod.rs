use std::fmt::Debug;
use std::hash::Hash;
use derivative::Derivative;
use num::{Unsigned, Zero};

use crate::*;

#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative)]
#[derivative(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Triangle<T: Scalar>(pub Point2<T>, pub Point2<T>, pub Point2<T>);

impl<T> Triangle<T>
  where
    T: Scalar,
{
  #[inline]
  pub fn new(a: Point2<T>, b: Point2<T>, c: Point2<T>) -> Self {
    Self(a, b, c)
  }
}

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
    let x_sum: u16 = self.0.x as u16 + self.1.x as u16 + self.2.x as u16;
    let y_sum: u16 = self.0.y as u16 + self.1.y as u16 + self.2.y as u16;

    Point2::new(
      (x_sum as f64 / 3.0).round() as u8,
      (y_sum as f64 / 3.0).round() as u8,
    )
  }

  pub fn bbox(&self) -> Rectangle<u8> {
    let min_x = self.0.x.min(self.1.x).min(self.2.x);
    let min_y = self.0.y.min(self.1.y).min(self.2.y);
    let max_x = self.0.x.max(self.1.x).max(self.2.x);
    let max_y = self.0.y.max(self.1.y).max(self.2.y);

    Rectangle::new(
      Point2::new(min_x, min_y),
      Point2::new(max_x, max_y).map(|x| x.saturating_add(1)),
    )
  }

  pub fn points_inside(&self) -> Vec<Point2<u8>> {
    return self.bbox().points_inside()
      .into_iter()
      .filter(|point| {
        self.within(*point)
      })
      .collect();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use test_case::test_case;
  use test_strategy::proptest;

  #[proptest]
  fn triangle_bbox_u8_fuzz(triangle: Triangle<u8>) {
    let _bbox = triangle.bbox();
  }

  #[proptest]
  fn triangle_points_inside_u8_fuzz(triangle: Triangle<u8>) {
    let _points = triangle.points_inside();
  }
}