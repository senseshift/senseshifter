use nalgebra::{Point2, Scalar};
use num::Num;
use num::traits::NumOps;
use crate::{Circle, Ellipse, Line, Rectangle, Triangle};

/// Calculate the squared distance between two points.
///
/// # Example:
/// ```rust
/// use xrc_geometry::{Point2, distance_squared};
///
/// let a = Point2::new(0, 0);
/// let b = Point2::new(3, 4);
///
/// assert_eq!(distance_squared(&a, &b), 25.0);
/// ```
pub fn distance_squared<T>(a: &Point2<T>, b: &Point2<T>) -> f64
  where
    T: Scalar + Num + NumOps + Ord + Copy + Into<f64>,
{
  let x_max = a.x.max(b.x);
  let x_min = a.x.min(b.x);
  let y_max = a.y.max(b.y);
  let y_min = a.y.min(b.y);

  let x = x_max - x_min;
  let y = y_max - y_min;

  let x = Into::<f64>::into(x);
  let y = Into::<f64>::into(y);

  x * x + y * y
}

/// Calculate the distance between two points.
///
/// # Example:
/// ```rust
/// use xrc_geometry::{Point2, distance};
///
/// let a = Point2::new(0, 0);
/// let b = Point2::new(3, 4);
///
/// assert_eq!(distance(&a, &b), 5.0);
/// ```
pub fn distance<T>(a: &Point2<T>, b: &Point2<T>) -> f64
  where
    T: Scalar + Num + NumOps + Ord + Copy + Into<f64>,
{
  use num::integer::Roots;
  distance_squared(a, b).sqrt()
}

pub trait Distance<T> {
  type Result;

  fn distance(&self, other: T) -> Self::Result;
}

impl Distance<&Point2<u8>> for Circle<u8, u8> {
  type Result = f64;

  /// Calculate the distance from the edge of the circle to the point.
  /// If the point is inside the circle, return 0.
  ///
  /// # Example:
  /// ```rust
  /// use xrc_geometry::{Point2, Circle, Distance};
  ///
  /// let circle = Circle::new([5, 5].into(), 10);
  /// let point = [20, 5].into();
  ///
  /// assert_eq!(circle.distance(&point), 5);
  /// ```
  fn distance(&self, point: &Point2<u8>) -> Self::Result {
    let distance_to_center = distance(&self.center, point);

    // if the point is inside the circle, return 0
    if distance_to_center < self.radius as f64 {
      return 0.0;
    }

    let distance_to_edge = distance_to_center - self.radius as f64;

    distance_to_edge
  }
}
impl Distance<Point2<u8>> for Circle<u8, u8> {
  type Result = f64;

  fn distance(&self, point: Point2<u8>) -> Self::Result {
    self.distance(&point)
  }
}

impl Distance<&Point2<u8>> for Ellipse<u8, u8> {
  type Result = f64;

  fn distance(&self, point: &Point2<u8>) -> f64 {
    use ordered_float::OrderedFloat;
    use crate::{Within, PointWithin};

    if self.within(point) == PointWithin::Inside {
      return 0.0;
    }

    let point_on_ellipse = self.point_intersection(point, 10);
    distance(
      &point_on_ellipse.map(|c| OrderedFloat(c)),
      &point.map(|c| OrderedFloat(c as f64)),
    )
  }
}
impl Distance<Point2<u8>> for Ellipse<u8, u8> {
  type Result = f64;

  fn distance(&self, point: Point2<u8>) -> f64 {
    self.distance(&point)
  }
}

impl Distance<&Point2<u8>> for Line<u8> {
  type Result = f64;

  /// # Example
  /// ```rust
  /// use xrc_geometry::{Point2, Line, Distance};
  ///
  /// let line = Line::new(Point2::new(5, 5), Point2::new(5, 10));
  ///
  /// assert_eq!(line.distance(&Point2::new(5, 7)), 0.0); // Point is on the line
  /// assert_eq!(line.distance(&Point2::new(5, 4)), 1.0); // Point is to the left of the line
  /// assert_eq!(line.distance(&Point2::new(5, 11)), 1.0); // Point is to the right of the line
  /// assert_eq!(line.distance(&Point2::new(6, 7)), 1.0); // Point is above the line
  /// assert_eq!(line.distance(&Point2::new(4, 7)), 1.0); // Point is below the line
  /// ```
  fn distance(&self, point: &Point2<u8>) -> f64 {
    let xy = point.map(|x| x as f64);
    let xy1 = self.start.map(|x| x as f64);
    let xy2 = self.end.map(|x| x as f64);

    let a = xy.x - xy1.x;
    let b = xy.y - xy1.y;
    let c = xy2.x - xy1.x;
    let d = xy2.y - xy1.y;

    let dot = a * c + b * d;
    let len_sq = c * c + d * d;

    let param = if len_sq != 0.0 { dot / len_sq } else { -1.0 };

    let mut xx;
    let mut yy;

    if param < 0.0 {
      xx = xy1.x;
      yy = xy1.y;
    } else if param > 1.0 {
      xx = xy2.x;
      yy = xy2.y;
    } else {
      xx = xy1.x + param * c;
      yy = xy1.y + param * d;
    }

    let dx = xy.x - xx;
    let dy = xy.y - yy;

    (dx * dx + dy * dy).sqrt()
  }
}
impl Distance<Point2<u8>> for Line<u8> {
  type Result = f64;

  fn distance(&self, point: Point2<u8>) -> f64 {
    self.distance(&point)
  }
}

impl Distance<&Point2<u8>> for Rectangle<u8> {
  type Result = f64;

  /// Calculate the distance from the edge of the rectangle to the point.
  ///
  /// # Example
  /// ```rust
  /// use xrc_geometry::{Point2, Rectangle, Distance};
  ///
  /// let rectangle = Rectangle::new(Point2::new(0, 0), Point2::new(10, 10));
  ///
  /// assert_eq!(rectangle.distance(&Point2::new(5, 5)), 0.0); // Point is inside the rectangle
  /// assert_eq!(rectangle.distance(&Point2::new(20, 10)), 10.0); // Point is to the right of the rectangle
  /// ```
  fn distance(&self, point: &Point2<u8>) -> f64 {
    use crate::{Within, PointWithin};
    if self.within(point) == PointWithin::Inside {
      return 0.0;
    }

    let top = Line::new(self.min().clone(), Point2::new(self.max().x.clone(), self.min().y.clone()));
    let right = Line::new(Point2::new(self.max().x.clone(), self.min().y.clone()), self.max().clone());
    let bottom = Line::new(self.max().clone(), Point2::new(self.min().x.clone(), self.max().y.clone()));
    let left = Line::new(Point2::new(self.min().x.clone(), self.max().y.clone()), self.min().clone());

    let distances = [
      top.distance(point),
      right.distance(point),
      bottom.distance(point),
      left.distance(point),
    ];

    distances
      .iter()
      .copied()
      .fold(f64::MAX, f64::min)
  }
}
impl Distance<Point2<u8>> for Rectangle<u8> {
  type Result = f64;

  fn distance(&self, point: Point2<u8>) -> f64 {
    self.distance(&point)
  }
}

impl Distance<&Point2<u8>> for Triangle<u8> {
  type Result = f64;

  /// Calculate the distance from the edge of the triangle to the point.
  ///
  /// # Example
  /// ```rust
  /// use xrc_geometry::{Point2, Triangle, Distance};
  ///
  /// let triangle = Triangle::new(
  ///   Point2::new(0, 0),
  ///   Point2::new(10, 0),
  ///   Point2::new(0, 10),
  /// );
  ///
  /// assert_eq!(triangle.distance(&Point2::new(5, 5)), 0.0); // Point is inside the triangle
  /// assert_eq!(triangle.distance(&Point2::new(20, 10)), 10.0); // Point is to the right of the triangle
  /// ```
  fn distance(&self, point: &Point2<u8>) -> f64 {
    // use crate::{Within, PointWithin};
    // if self.within(point) == PointWithin::Inside {
    //   return 0.0;
    // }

    let a = Line::new(self.0.clone(), self.1.clone());
    let b = Line::new(self.1.clone(), self.2.clone());
    let c = Line::new(self.2.clone(), self.0.clone());

    let distances = [
      a.distance(point),
      b.distance(point),
      c.distance(point),
    ];

    distances
      .iter()
      .copied()
      .fold(f64::MAX, f64::min)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::*;

  use test_case::test_case;
  use test_strategy::proptest;
  use crate::testing::PointView;

  #[test_case(Point2::new(0, 0), Point2::new(3, 4) => 25)]
  #[test_case(Point2::new(0, 0), Point2::new(0, 0) => 0)]
  #[test_case(Point2::new(0, 0), Point2::new(1, 1) => 2)]
  #[test_case(Point2::new(0, 0), Point2::new(1, 0) => 1)]
  #[test_case(Point2::new(0, 0), Point2::new(0, 1) => 1)]
  #[test_case(Point2::new(0, 0), Point2::new(255, 255) => 130050)]
  fn distance_squared_u8(a: Point2<u8>, b: Point2<u8>) -> u32 {
    distance_squared(&a, &b) as u32
  }

  #[proptest]
  fn distance_squared_u8_fuzz(a: PointView<u8, 2>, b: PointView<u8, 2>) {
    let _out = distance_squared(&a.into(), &b.into());
  }

  #[test_case(Point2::new(0, 0), Point2::new(3, 4) => 5)]
  #[test_case(Point2::new(0, 0), Point2::new(0, 0) => 0)]
  #[test_case(Point2::new(0, 0), Point2::new(1, 1) => 1)]
  #[test_case(Point2::new(0, 0), Point2::new(1, 0) => 1)]
  #[test_case(Point2::new(0, 0), Point2::new(0, 1) => 1)]
  #[test_case(Point2::new(0, 0), Point2::new(255, 255) => 360)]
  fn distance_u8(a: Point2<u8>, b: Point2<u8>) -> u32 {
    distance(&a, &b) as u32
  }

  #[proptest]
  fn distance_u8_fuzz(a: PointView<u8, 2>, b: PointView<u8, 2>) {
    let _out = distance(&a.into(), &b.into());
  }

  #[proptest]
  fn circle_distance_u8_fuzz(circle: Circle<u8, u8>, point: PointView<u8, 2>) {
    let _out = circle.distance(&point.into());
  }

  #[proptest]
  fn ellipse_distance_u8_fuzz(ellipse: Ellipse<u8, u8>, point: PointView<u8, 2>) {
    let _out = ellipse.distance(&point.into());
  }

  #[proptest]
  fn line_distance_u8_fuzz(line: Line<u8>, point: PointView<u8, 2>) {
    let _out = line.distance(&point.into());
  }

  #[proptest]
  fn rectangle_distance_u8_fuzz(rectangle: Rectangle<u8>, point: PointView<u8, 2>) {
    let _out = rectangle.distance(&point.into());
  }
}