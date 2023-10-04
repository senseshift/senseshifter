use nalgebra::{Point2, Scalar};
use num::traits::{Num, NumOps, Unsigned};

use crate::{Circle, Ellipse, Line, Rectangle, Triangle, ShapeCollection, Shape, Distance, distance};

impl Distance<&Circle<u8, u8>> for Circle<u8, u8>
{
  type Result = f64;

  fn distance(&self, other: &Circle<u8, u8>) -> Self::Result {
    let centers_distance = distance(&self.center, &other.center);
    let radius_sum = self.radius as f64 + other.radius as f64;

    if centers_distance > radius_sum {
      centers_distance - radius_sum
    } else {
      0.0
    }
  }
}
impl Distance<Circle<u8, u8>> for Circle<u8, u8>
{
  type Result = f64;

  fn distance(&self, other: Circle<u8, u8>) -> Self::Result {
    self.distance(&other)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::*;

  use test_case::test_case;
  use test_strategy::proptest;
  use crate::testing::PointView;

  #[proptest]
  fn circle_distance_u8_fuzz(circle1: Circle<u8, u8>, circle2: Circle<u8, u8>) {
    let _out = circle1.distance(&circle2);
  }
}