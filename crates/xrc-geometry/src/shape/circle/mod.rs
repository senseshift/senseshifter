use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use crate::*;
use getset::Getters;
use nalgebra::Vector2;
use num::{Unsigned, Zero};

#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Getters)]
pub struct Circle<T: Scalar, R: Scalar + Unsigned> {
  #[getset(get = "pub")]
  pub(crate) center: Point2<T>,
  #[getset(get = "pub")]
  pub(crate) radius: R,
}

impl<T, R> Default for Circle<T, R>
  where
    T: Scalar + Default + Zero,
    R: Scalar + Unsigned + Default,
{
  fn default() -> Self {
    Self {
      center: Point2::default(),
      radius: R::default(),
    }
  }
}

impl<T, R> Debug for Circle<T, R>
  where
    T: Scalar + Debug,
    R: Scalar + Unsigned + Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Circle")
      .field("center", &self.center)
      .field("radius", &self.radius)
      .finish()
  }
}

impl<T, R> Circle<T, R>
  where
    T: Scalar,
    R: Scalar + Unsigned,
{
  pub fn new(center: Point2<T>, radius: R) -> Self {
    Self { center, radius }
  }
}

impl<T, R> Hash for Circle<T, R>
  where
    T: Scalar + Hash,
    R: Scalar + Unsigned + Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.center.hash(state);
    self.radius.hash(state);
  }
}

impl<T, R> Copy for Circle<T, R>
  where
    T: Scalar + Copy,
    R: Scalar + Unsigned + Copy,
{}

impl<T, R> Clone for Circle<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned + Clone,
{
  fn clone(&self) -> Self {
    Self {
      center: self.center.clone(),
      radius: self.radius.clone(),
    }
  }
}

impl<T, R> PartialEq for Circle<T, R>
  where
    T: Scalar + PartialEq,
    R: Scalar + Unsigned + PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.center == other.center && self.radius == other.radius
  }
}

impl<T, R> Eq for Circle<T, R>
  where
    T: Scalar + Eq,
    R: Scalar + Unsigned + Eq,
{}

impl Circle<u8, u8> {
  pub fn bbox(&self) -> Rectangle<u8> {
    let radius = self.radius as i16;
    let center = self.center.map(|x| x as i16);

    let min = center - Vector2::new(radius, radius);
    let max = center + Vector2::new(radius, radius);

    Rectangle::new(
      min.map(|x| {
        if x < 0 {
          0
        } else {
          x as u8
        }
      }),
      max.map(|x| {
        if x >= u8::MAX as i16 {
          u8::MAX
        } else {
          x as u8 + 1
        }
      })
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

  #[test]
  fn test_circle_bbox() {
    let circle = Circle::new(Point2::new(12, 12), 10);
    let bbox = circle.bbox();

    assert_eq!(bbox.min(), &Point2::new(2, 2));
    assert_eq!(bbox.max(), &Point2::new(22, 22));
  }

  #[test]
  fn test_circle_bbox_edge() {
    let circle = Circle::new(Point2::new(0, 0), 10);
    let bbox = circle.bbox();

    assert_eq!(bbox.min(), &Point2::new(0, 0));
    assert_eq!(bbox.max(), &Point2::new(11, 11));
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
      Point2::new(6, 4),
      Point2::new(6, 5),
      Point2::new(6, 6),
    ];

    expected.iter().for_each(|point| {
      assert!(points.contains(point), "Point {:?} not found in {:?}", point, points);
    });
    points.iter().for_each(|point| {
      assert!(expected.contains(point), "Unexpected point {:?} found", point);
    });
    assert_eq!(points.len(), expected.len());
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

    expected.iter().for_each(|point| {
      assert!(points.contains(point), "Point {:?} not found in {:?}", point, points);
    });
    points.iter().for_each(|point| {
      assert!(expected.contains(point), "Unexpected point {:?} found", point);
    });
    assert_eq!(points.len(), expected.len());
  }
}