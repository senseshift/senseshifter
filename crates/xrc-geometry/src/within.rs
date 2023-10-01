use crate::{
  Point, Circle, Ellipse,
};

pub trait Within<T> {
  type Result;

  fn within(&self, other: T) -> bool;
}

macro_rules! fixed_precision_within {
  ( $ty:ty, $uty:ty, $long:ty, $ulong: ty ) => {
    impl crate::Within<&Point<$ty, 2>> for Circle<$ty>
    {
      type Result = bool;

      fn within(&self, point: &Point<$ty, 2>) -> Self::Result {
        if self.radius == 0 {
          return point == &self.center;
        }

        // manually calculate distance squared to avoid overflow
        let dx: $long = (point.x as $long) - (self.center.x as $long);
        let dx2: $ulong = dx * dx;
        let dy: $long = (point.y as $long) - (self.center.y as $long);
        let dy2: $ulong = dy * dy;

        let distance_squared = dx2 + dy2;
        let radius_squared: $ulong = (self.radius as $long) * (self.radius as $long);

        distance_squared <= radius_squared
      }
    }
    impl crate::Within<Point<$ty, 2>> for Circle<$ty> {
      type Result = bool;

      fn within(&self, point: Point<$ty, 2>) -> Self::Result {
        self.within(&point)
      }
    }

    impl crate::Within<&Point<$ty, 2>> for Ellipse<$ty> {
      type Result = bool;

      fn within(&self, point: &Point<$ty, 2>) -> Self::Result {
        if *self.width() == 0 && *self.height() == 0 {
          return point == &self.center;
        }

        if *self.width() == 0 {
          let py = ((point.y as $long) - (self.center.y as $long)).abs();

          return point.x == self.center.x
              && point.y >= (self.center.y - py as $ty)
              && point.y <= (self.center.y + py as $ty);
        } else if *self.height() == 0 {
          let px = ((point.x as $long) - (self.center.x as $long)).abs();

          return point.y == self.center.y
              && point.x >= (self.center.x - px as $ty)
              && point.x <= (self.center.x + px as $ty);
        }

        let px = (point.x as $long) - (self.center.x as $long);
        let px2 = px * px;

        let py = (point.y as $long) - (self.center.y as $long);
        let py2 = py * py;

        let rx2 = (*self.width() as $long) * (*self.width() as $long);
        let ry2 = (*self.height() as $long) * (*self.height() as $long);

        px2 / rx2 + py2 / ry2 <= 1
      }
    }
    impl crate::Within<Point<$ty, 2>> for Ellipse<$ty> {
      type Result = bool;

      fn within(&self, point: Point<$ty, 2>) -> Self::Result {
        self.within(&point)
      }
    }
  };
}

fixed_precision_within!(u8, i16, i32, i32);

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
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([10, 10]) => false; "inside edge top-right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([11, 0]) => false; "outside right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([0, 11]) => false; "outside top")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([11, 11]) => false; "outside top-right")]
  #[test_case(Circle::new(Point::from([0, 0]), 10), Point::from([255, 255]) => false; "outside max")]
  fn circle_within_u8(circle: Circle<u8>, point: Point<u8, 2>) -> bool {
    use crate::Within;
    circle.within(&point)
  }

  #[proptest]
  fn circle_within_u8_fuzz(circle: Circle<u8>, point: PointView<u8, 2>) {
    use crate::Within;
    let _out: bool = circle.within(&point.into());
  }

  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([0, 0]) => true; "center")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([2, 2]) => true; "inside")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (5, 5)), Point::from([0, 5]) => true; "edge")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([10, 11]) => false; "outside")]
  #[test_case(Ellipse::new(Point::from([0, 0]), (4, 5)), Point::from([255, 255]) => false; "outside max")]
  #[test_case(Ellipse::new(Point::from([5, 5]), (4, 3)), Point::from([6, 6]) => true; "non-centered inside")]
  #[test_case(Ellipse::new(Point::from([5, 5]), (4, 3)), Point::from([10, 10]) => false; "non-centered outside")]
  fn ellipse_within_u8(ellipse: Ellipse<u8>, point: Point<u8, 2>) -> bool {
    use crate::Within;
    ellipse.within(&point)
  }

  #[proptest]
  fn ellipse_within_u8_fuzz(ellipse: Ellipse<u8>, point: PointView<u8, 2>) {
    use crate::Within;
    let _out: bool = ellipse.within(&point.into());
  }
}