use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use crate::*;
use num::{Num, Zero};
use nalgebra::*;

#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Rectangle<T: Scalar>(Point2<T>, Point2<T>);

impl<T> Default for Rectangle<T>
  where
    T: Scalar + Default + Zero,
{
  fn default() -> Self {
    Self(Point2::default(), Point2::default())
  }
}

impl<T> Debug for Rectangle<T>
  where
    T: Scalar + Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Rectangle")
      .field("min", &self.min())
      .field("max", &self.max())
      .finish()
  }
}

impl<T> Rectangle<T>
  where
    T: Scalar,
{
  pub fn min(&self) -> &Point2<T> {
    &self.0
  }

  pub fn max(&self) -> &Point2<T> {
    &self.1
  }
}

impl<T> Rectangle<T>
  where
    T: Scalar + Ord + Copy,
{
  pub fn new(min: Point2<T>, max: Point2<T>) -> Self {
    // normalize
    let min = Point2::new(min.x.min(max.x), min.y.min(max.y));
    let max = Point2::new(min.x.max(max.x), min.y.max(max.y));

    Self(min, max)
  }
}

impl<T> Rectangle<T>
  where
    T: Scalar + ?Ord,
{
  pub fn new_unchecked(min: Point2<T>, max: Point2<T>) -> Self {
    Self(min, max)
  }
}

impl<T> Rectangle<T>
  where
    T: Scalar + Num,
{
  /// Returns the width of the rectangle.
  ///
  /// # Example
  /// ```rust
  /// use xrc_geometry::{Point2, Rectangle};
  ///
  /// let rectangle = Rectangle::new(Point2::new(0, 0), Point2::new(10, 10));
  /// assert_eq!(rectangle.width(), 10);
  ///
  /// ```
  pub fn width(&self) -> T {
    self.max().x.clone() - self.min().x.clone()
  }

  /// Returns the height of the rectangle.
  ///
  /// # Example
  /// ```rust
  /// use xrc_geometry::{Point2, Rectangle};
  ///
  /// let rectangle = Rectangle::new(Point2::new(0, 0), Point2::new(10, 10));
  /// assert_eq!(rectangle.height(), 10);
  ///
  /// ```
  pub fn height(&self) -> T {
    self.max().y.clone() - self.min().y.clone()
  }
}

impl<T> Hash for Rectangle<T>
  where
    T: Scalar + Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.min().hash(state);
    self.max().hash(state);
  }
}

impl<T> PartialEq for Rectangle<T>
  where
    T: Scalar + PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.min() == other.min() && self.max() == other.max()
  }
}

impl<T> Eq for Rectangle<T>
  where
    T: Scalar + Eq,
{}

impl<T> Copy for Rectangle<T>
  where
    T: Scalar + Copy,
{}

impl<T> Clone for Rectangle<T>
  where
    T: Scalar + Clone,
{
  fn clone(&self) -> Self {
    Self(self.min().clone(), self.max().clone())
  }
}

impl Rectangle<u8> {
  /// Returns the center of the rectangle.
  ///
  /// # Example
  /// ```rust
  /// use xrc_geometry::{Point2, Rectangle};
  ///
  /// let rectangle = Rectangle::new(Point2::new(0, 0), Point2::new(10, 10));
  /// assert_eq!(rectangle.center(), Point2::new(5, 5));
  ///
  /// ```
  pub fn center(&self) -> Point2<u8> {
    let min = self.min().map(|x| x as u16);
    let max = self.max().map(|x| x as u16);

    Point2::new(
      ((min.x + max.x) / 2) as u8,
      ((min.y + max.y) / 2) as u8,
    )
  }

  /// Returns a vector of all points inside the rectangle.
  ///
  /// # Example
  /// ```rust
  /// use xrc_geometry::{Point2, Rectangle};
  ///
  /// let rectangle = Rectangle::new(Point2::new(0, 0), Point2::new(2, 2));
  /// assert_eq!(rectangle.points_inside(), vec![
  ///   Point2::new(0, 0),
  ///   Point2::new(0, 1),
  ///   Point2::new(1, 0),
  ///   Point2::new(1, 1),
  /// ]);
  /// ```
  pub fn points_inside(&self) -> Vec<Point2<u8>> {
    let mut points = Vec::with_capacity(self.width() as usize * self.height() as usize);

    for x in self.min().x..self.max().x {
      for y in self.min().y..self.max().y {
        points.push(Point2::new(x, y));
      }
    }

    points
  }
}

#[cfg(test)]
mod tests {}