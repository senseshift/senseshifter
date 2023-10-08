use crate::{Circle, Ellipse, Rectangle, Triangle, ShapeCollection, Shape};
use nalgebra::*;
use num::{Num, Unsigned};

pub trait Within<T> {
  type Result;

  fn within(&self, other: T) -> Self::Result;
}

// impl<T, U> Within<&T> for U
//   where
//     U: Within<T, Result = PointWithin>,
// {
//   type Result = bool;
//
//   fn within(&self, other: &T) -> bool {
//     self.within(other) == true
//   }
// }

impl Within<&Point2<u8>> for Circle<u8, u8>
{
  type Result = bool;

  fn within(&self, other: &Point2<u8>) -> Self::Result {
    if self.radius == 0 {
      return other == &self.center;
    }

    // manually calculate distance squared to avoid overflow
    let dx: u16 = ((other.x as i16) - (self.center.x as i16)).abs() as u16;
    let dx2: u16 = dx * dx;

    let dy: u16 = ((other.y as i16) - (self.center.y as i16)).abs() as u16;
    let dy2: u16 = dy * dy;

    let distance_squared: u32 = dx2 as u32 + dy2 as u32;
    let radius_squared: u32 = (self.radius as u32) * (self.radius as u32);

    distance_squared <= radius_squared
  }
}

impl Within<Point2<u8>> for Circle<u8, u8> {
  type Result = bool;

  fn within(&self, other: Point2<u8>) -> Self::Result {
    self.within(&other)
  }
}

impl Within<&Point2<u8>> for Ellipse<u8, u8>
{
  type Result = bool;

  fn within(&self, other: &Point2<u8>) -> Self::Result {
    if *self.width() == 0 || *self.height() == 0 {
      return other == &self.center;
    }

    if !self.bbox().within(other) {
      return false;
    }

    let px = ((other.x as f64) - (self.center.x as f64));
    let px2 = px.powi(2);

    let py = ((other.y as f64) - (self.center.y as f64));
    let py2 = py.powi(2);

    let rx2 = (*self.width() as f64).powi(2);
    let ry2 = (*self.height() as f64).powi(2);

    let dst = px2 / rx2 + py2 / ry2;

    dst <= 1.0
  }
}

impl Within<Point2<u8>> for Ellipse<u8, u8> {
  type Result = bool;

  fn within(&self, other: Point2<u8>) -> Self::Result {
    self.within(&other)
  }
}

impl<T> Within<&Point2<T>> for Rectangle<T>
  where
    T: Scalar + PartialOrd,
{
  type Result = bool;

  fn within(&self, other: &Point2<T>) -> Self::Result {
    // check if outside
    if other.x < self.min().x || other.x > self.max().x {
      return false;
    } else if other.y < self.min().y || other.y > self.max().y {
      return false;
    }

    true
  }
}
impl<T> Within<Point2<T>> for Rectangle<T>
  where
    T: Scalar + PartialOrd,
{
  type Result = bool;

  fn within(&self, other: Point2<T>) -> Self::Result {
    self.within(&other)
  }
}

impl Within<&Point2<u8>> for Triangle<u8>
{
  type Result = bool;

  fn within(&self, other: &Point2<u8>) -> Self::Result {
    if !self.bbox().within(other) {
      return false;
    }

    let a = self.0.map(|x| x as f64);
    let b = self.1.map(|x| x as f64);
    let c = self.2.map(|x| x as f64);

    let p = other.map(|x| x as f64);

    let area = |a: &Point2<f64>, b: &Point2<f64>, c: &Point2<f64>| {
      let x1 = a.x - c.x;
      let y1 = a.y - c.y;
      let x2 = b.x - c.x;
      let y2 = b.y - c.y;
      let area = x1 * y2 - x2 * y1;
      area.abs()
    };

    let area_abc = area(&a, &b, &c);
    let area_pbc = area(&p, &b, &c);
    let area_apc = area(&a, &p, &c);
    let area_abp = area(&a, &b, &p);

    area_abc == area_pbc + area_apc + area_abp
  }
}
impl Within<Point2<u8>> for Triangle<u8>
{
  type Result = bool;

  fn within(&self, other: Point2<u8>) -> Self::Result {
    self.within(&other)
  }
}

impl<T, U> Within<&Point2<T>> for ShapeCollection<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
    Shape<T, U>: for<'a> Within<&'a Point2<T>, Result = bool>,
{
  type Result = bool;

  fn within(&self, other: &Point2<T>) -> Self::Result {
    for geometry in &self.shapes {
      if geometry.within(other) == true {
        return true;
      }
    }
    false
  }
}
impl<T, U> Within<Point2<T>> for ShapeCollection<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
    Shape<T, U>: for<'a> Within<&'a Point2<T>, Result = bool>,
{
  type Result = bool;

  fn within(&self, other: Point2<T>) -> Self::Result {
    self.within(&other)
  }
}

impl<T, U> Within<&Point2<T>> for Shape<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
    Ellipse<T, U>: for<'a> Within<&'a Point<T, 2>, Result=bool>,
    Circle<T, U>: for<'a> Within<&'a Point<T, 2>, Result=bool>,
    Rectangle<T>: for<'a> Within<&'a Point<T, 2>, Result=bool>,
    Triangle<T>: for<'a> Within<&'a Point<T, 2>, Result=bool>,
{
  type Result = bool;

  fn within(&self, other: &Point2<T>) -> Self::Result {
    match self {
      Self::Circle(circle) => circle.within(other),
      Self::Ellipse(ellipse) => ellipse.within(other),
      Self::Rectangle(rectangle) => rectangle.within(other),
      Self::Triangle(triangle) => triangle.within(other),
      Self::Collection(collection) => collection.within(other),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::*;

  use test_case::test_case;
  use test_strategy::proptest;
  use crate::testing::PointView;

  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([0, 0]) => true; "center")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([1, 0]) => true; "inside right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([0, 1]) => true; "inside top")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([10, 0]) => true; "inside edge right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([0, 10]) => true; "inside edge top")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([10, 10]) => false; "outside edge top-right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([11, 0]) => false; "outside right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([0, 11]) => false; "outside top")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([11, 11]) => false; "outside top-right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([255, 255]) => false; "outside max")]
  fn circle_within_u8(circle: Circle<u8, u8>, point: Point<u8, 2>) -> bool {
    use crate::Within;
    circle.within(&point)
  }

  #[proptest]
  fn circle_within_u8_fuzz(circle: Circle<u8, u8>, point: PointView<u8, 2>) {
    use crate::Within;
    let _out = circle.within(&point.into());
  }

  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([0, 0]) => true; "center")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([2, 2]) => true; "inside")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (5, 5)), Point::from([0, 5]) => true; "edge")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([10, 11]) => false; "outside")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([255, 255]) => false; "outside max")]
  #[test_case(Ellipse::new(Point::from([5, 5]), (4, 3)), Point::from([6, 6]) => true; "non-centered inside")]
  #[test_case(Ellipse::new(Point::from([5, 5]), (4, 3)), Point::from([10, 10]) => false; "non-centered outside")]
  fn ellipse_within_u8(ellipse: Ellipse<u8, u8>, point: Point<u8, 2>) -> bool {
    use crate::Within;
    ellipse.within(&point)
  }

  #[proptest]
  fn ellipse_within_u8_fuzz(ellipse: Ellipse<u8, u8>, point: PointView<u8, 2>) {
    use crate::Within;
    let _out = ellipse.within(&point.into());
  }

  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([0, 0]) => true; "top-left")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([10, 10]) => true; "bottom-right")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([5, 5]) => true; "center")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([0, 5]) => true; "left")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([5, 0]) => true; "top")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([10, 5]) => true; "right")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([5, 10]) => true; "bottom")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([11, 5]) => false; "outside right")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([5, 11]) => false; "outside top")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([11, 11]) => false; "outside top-right")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([255, 255]) => false; "outside max")]
  #[test_case(Rectangle::new(Point::from([5, 5]), Point::from([10, 10])), Point::from([6, 6]) => true; "non-centered inside")]
  #[test_case(Rectangle::new(Point::from([5, 5]), Point::from([10, 10])), Point::from([11, 11]) => false; "non-centered outside")]
  fn rectangle_within_u8(rectangle: Rectangle<u8>, point: Point<u8, 2>) -> bool {
    use crate::Within;
    rectangle.within(&point)
  }

  #[proptest]
  fn rectangle_within_u8_fuzz(rectangle: Rectangle<u8>, point: PointView<u8, 2>) {
    use crate::Within;
    let _out = rectangle.within(&point.into());
  }
}