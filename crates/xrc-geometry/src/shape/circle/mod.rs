use std::fmt::Debug;
use std::hash::Hash;
use derivative::Derivative;
use getset::Getters;
use nalgebra::Vector2;
use num::{Unsigned, Zero};

use crate::*;

#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Getters, Derivative)]
#[derivative(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Circle<T: Scalar, R: Scalar + Unsigned> {
  #[getset(get = "pub")]
  pub(crate) center: Point2<T>,
  #[getset(get = "pub")]
  pub(crate) radius: R,
}

impl<T, R> Circle<T, R>
  where
    T: Scalar,
    R: Scalar + Unsigned,
{
  #[inline]
  pub fn new(center: Point2<T>, radius: R) -> Self {
    Self { center, radius }
  }
}

impl Circle<u8, u8> {
  pub fn bbox(&self) -> Rectangle<u8> {
    let radius = self.radius as i16;
    let center = self.center.map(|x| x as i16);

    let min = center - Vector2::new(radius, radius);
    let max = center + Vector2::new(radius, radius);

    Rectangle::new(
      min.map(|x| x.clamp(0, u8::MAX as i16) as u8),
      max.map(|x| x.clamp(0, u8::MAX as i16) as u8)
    )
  }

  pub fn points_inside(&self) -> Vec<Point2<u8>> {
    return self.bbox().points_inside()
      .into_iter()
      .filter(|point| {
        self.within(point)
      })
      .collect();
  }
}

#[cfg(test)]
mod tests {
  use crate::assert_vec_eq;
  use super::*;

  use test_case::test_case;
  use test_strategy::proptest;

  #[test_case(Circle::new(Point2::new(12 , 12 ), 10), Point2::new(2  , 2  ), Point2::new(22 , 22 ); "normal")]
  #[test_case(Circle::new(Point2::new(0  , 0  ), 10), Point2::new(0  , 0  ), Point2::new(10 , 10 ); "edge/top+start")]
  #[test_case(Circle::new(Point2::new(255, 0  ), 10), Point2::new(245, 0  ), Point2::new(255, 10 ); "edge/top+end")]
  #[test_case(Circle::new(Point2::new(0  , 255), 10), Point2::new(0  , 245), Point2::new(10 , 255); "edge/bottom+start")]
  #[test_case(Circle::new(Point2::new(255, 255), 10), Point2::new(245, 245), Point2::new(255, 255); "edge/bottom+end")]
  fn test_circle_bbox(circle: Circle<u8, u8>, min: Point2<u8>, max: Point2<u8>) {
    let bbox = circle.bbox();

    assert_eq!(bbox.min(), &min);
    assert_eq!(bbox.max(), &max);
  }

  #[proptest]
  fn circle_bbox_u8_fuzz(circle: Circle<u8, u8>) {
    let _out = circle.bbox();
  }

  #[test]
  fn test_points_inside() {
    let circle = Circle::new(Point2::new(5, 5), 2);
    let points = circle.points_inside();

    let expected = vec![
      Point2::new(3, 5),
      Point2::new(4, 4),
      Point2::new(4, 5),
      Point2::new(4, 6),
      Point2::new(5, 3),
      Point2::new(5, 4),
      Point2::new(5, 5),
      Point2::new(5, 6),
      Point2::new(5, 7),
      Point2::new(6, 4),
      Point2::new(6, 5),
      Point2::new(6, 6),
      Point2::new(7, 5),
    ];

    assert_vec_eq!(points, expected);
  }

  #[test]
  pub fn test_points_inside_edge() {
    let circle = Circle::new(Point2::new(0, 0), 5);
    let points = circle.points_inside();

    let expected = vec![
      Point2::new(0, 0),
      Point2::new(0, 1),
      Point2::new(0, 2),
      Point2::new(0, 3),
      Point2::new(0, 4),
      Point2::new(0, 5),
      Point2::new(1, 0),
      Point2::new(1, 1),
      Point2::new(1, 2),
      Point2::new(1, 3),
      Point2::new(1, 4),
      Point2::new(2, 0),
      Point2::new(2, 1),
      Point2::new(2, 2),
      Point2::new(2, 3),
      Point2::new(2, 4),
      Point2::new(3, 0),
      Point2::new(3, 1),
      Point2::new(3, 2),
      Point2::new(3, 3),
      Point2::new(3, 4),
      Point2::new(4, 0),
      Point2::new(4, 1),
      Point2::new(4, 2),
      Point2::new(4, 3),
      Point2::new(5, 0),
    ];

    assert_vec_eq!(points, expected);
  }

  #[proptest]
  fn circle_points_inside_u8_fuzz(circle: Circle<u8, u8>) {
    let _out = circle.points_inside();
  }
}