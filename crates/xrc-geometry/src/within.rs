use crate::{
  Circle, Ellipse, Rectangle, Triangle,
};
use nalgebra::*;
use num::Num;

#[derive(Debug, PartialEq, Eq)]
pub enum PointWithin {
  Inside,
  Edge,
  Outside,
}

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
//     self.within(other) == PointWithin::Inside
//   }
// }

impl Within<&Point2<u8>> for Circle<u8, u8>
{
  type Result = PointWithin;

  fn within(&self, other: &Point2<u8>) -> Self::Result {
    if self.radius == 0 {
      return if other == &self.center {
        PointWithin::Inside
      } else {
        PointWithin::Outside
      };
    }

    // manually calculate distance squared to avoid overflow
    let dx: u16 = ((other.x as i16) - (self.center.x as i16)).abs() as u16;
    let dx2: u16 = dx * dx;

    let dy: u16 = ((other.y as i16) - (self.center.y as i16)).abs() as u16;
    let dy2: u16 = dy * dy;

    let distance_squared: u32 = dx2 as u32 + dy2 as u32;
    let radius_squared: u32 = (self.radius as u32) * (self.radius as u32);

    if distance_squared < radius_squared {
      PointWithin::Inside
    } else if distance_squared == radius_squared {
      PointWithin::Edge
    } else {
      PointWithin::Outside
    }
  }
}

impl Within<Point2<u8>> for Circle<u8, u8> {
  type Result = PointWithin;

  fn within(&self, other: Point2<u8>) -> Self::Result {
    self.within(&other)
  }
}

impl Within<&Point2<u8>> for Ellipse<u8, u8>
{
  type Result = PointWithin;

  fn within(&self, other: &Point2<u8>) -> Self::Result {
    if *self.width() == 0 || *self.height() == 0 {
      return if other == &self.center {
        PointWithin::Inside
      } else {
        PointWithin::Outside
      };
    }

    let px = ((other.x as f64) - (self.center.x as f64));
    let px2 = px.powi(2);

    let py = ((other.y as f64) - (self.center.y as f64));
    let py2 = py.powi(2);

    let rx2 = (*self.width() as f64).powi(2);
    let ry2 = (*self.height() as f64).powi(2);

    let dst = px2 / rx2 + py2 / ry2;

    if dst < 1.0 {
      PointWithin::Inside
    } else if dst == 1.0 {
      PointWithin::Edge
    } else {
      PointWithin::Outside
    }
  }
}

impl Within<Point2<u8>> for Ellipse<u8, u8> {
  type Result = PointWithin;

  fn within(&self, other: Point2<u8>) -> Self::Result {
    self.within(&other)
  }
}

impl<T> Within<&Point2<T>> for Rectangle<T>
  where
    T: Scalar + PartialOrd,
{
  type Result = PointWithin;

  fn within(&self, other: &Point2<T>) -> Self::Result {
    if other.x < self.min().x || other.x > self.max().x {
      return PointWithin::Outside;
    }

    if other.y < self.min().y || other.y > self.max().y {
      return PointWithin::Outside;
    }

    if other.x == self.min().x || other.x == self.max().x {
      return PointWithin::Edge;
    }

    if other.y == self.min().y || other.y == self.max().y {
      return PointWithin::Edge;
    }

    PointWithin::Inside
  }
}
impl<T> Within<Point2<T>> for Rectangle<T>
  where
    T: Scalar + PartialOrd,
{
  type Result = PointWithin;

  fn within(&self, other: Point2<T>) -> Self::Result {
    self.within(&other)
  }
}

impl Within<&Point2<u8>> for Triangle<u8>
{
  type Result = PointWithin;

  fn within(&self, other: &Point2<u8>) -> Self::Result {
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

    if area_abc == area_pbc + area_apc + area_abp {
      PointWithin::Inside
    } else {
      PointWithin::Outside
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

  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([0, 0]) => PointWithin::Inside; "center")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([1, 0]) => PointWithin::Inside; "inside right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([0, 1]) => PointWithin::Inside; "inside top")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([10, 0]) => PointWithin::Edge; "inside edge right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([0, 10]) => PointWithin::Edge; "inside edge top")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([10, 10]) => PointWithin::Outside; "outside edge top-right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([11, 0]) => PointWithin::Outside; "outside right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([0, 11]) => PointWithin::Outside; "outside top")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([11, 11]) => PointWithin::Outside; "outside top-right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([255, 255]) => PointWithin::Outside; "outside max")]
  fn circle_within_u8(circle: Circle<u8, u8>, point: Point<u8, 2>) -> PointWithin {
    use crate::Within;
    circle.within(&point)
  }

  #[proptest]
  fn circle_within_u8_fuzz(circle: Circle<u8, u8>, point: PointView<u8, 2>) {
    use crate::Within;
    let _out = circle.within(&point.into());
  }

  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([0, 0]) => PointWithin::Inside; "center")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([2, 2]) => PointWithin::Inside; "inside")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (5, 5)), Point::from([0, 5]) => PointWithin::Edge; "edge")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([10, 11]) => PointWithin::Outside; "outside")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([255, 255]) => PointWithin::Outside; "outside max")]
  #[test_case(Ellipse::new(Point::from([5, 5]), (4, 3)), Point::from([6, 6]) => PointWithin::Inside; "non-centered inside")]
  #[test_case(Ellipse::new(Point::from([5, 5]), (4, 3)), Point::from([10, 10]) => PointWithin::Outside; "non-centered outside")]
  fn ellipse_within_u8(ellipse: Ellipse<u8, u8>, point: Point<u8, 2>) -> PointWithin {
    use crate::Within;
    ellipse.within(&point)
  }

  #[proptest]
  fn ellipse_within_u8_fuzz(ellipse: Ellipse<u8, u8>, point: PointView<u8, 2>) {
    use crate::Within;
    let _out = ellipse.within(&point.into());
  }

  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([0, 0]) => PointWithin::Edge; "top-left")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([10, 10]) => PointWithin::Edge; "bottom-right")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([5, 5]) => PointWithin::Inside; "center")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([0, 5]) => PointWithin::Edge; "left")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([5, 0]) => PointWithin::Edge; "top")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([10, 5]) => PointWithin::Edge; "right")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([5, 10]) => PointWithin::Edge; "bottom")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([11, 5]) => PointWithin::Outside; "outside right")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([5, 11]) => PointWithin::Outside; "outside top")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([11, 11]) => PointWithin::Outside; "outside top-right")]
  #[test_case(Rectangle::new(Point::from([0, 0]), Point::from([10, 10])), Point::from([255, 255]) => PointWithin::Outside; "outside max")]
  #[test_case(Rectangle::new(Point::from([5, 5]), Point::from([10, 10])), Point::from([6, 6]) => PointWithin::Inside; "non-centered inside")]
  #[test_case(Rectangle::new(Point::from([5, 5]), Point::from([10, 10])), Point::from([11, 11]) => PointWithin::Outside; "non-centered outside")]
  fn rectangle_within_u8(rectangle: Rectangle<u8>, point: Point<u8, 2>) -> PointWithin {
    use crate::Within;
    rectangle.within(&point)
  }

  #[proptest]
  fn rectangle_within_u8_fuzz(rectangle: Rectangle<u8>, point: PointView<u8, 2>) {
    use crate::Within;
    let _out = rectangle.within(&point.into());
  }
}