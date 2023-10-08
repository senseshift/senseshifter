use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Div;
use nalgebra::{Point2, Scalar, Vector2};
use num::Unsigned;

use super::{Shape, Rectangle};

pub struct ShapeCollection<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
{
  pub shapes: Vec<Shape<T, U>>,
}

impl<T, U> Default for ShapeCollection<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
{
  fn default() -> Self {
    Self {
      shapes: Vec::default(),
    }
  }
}

impl<T, U> Debug for ShapeCollection<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
    Shape<T, U>: Debug
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ShapeCollection")
      .field("geometry", &self.shapes)
      .finish()
  }
}

impl<T, U> ShapeCollection<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
{
  pub fn new(geometry: Vec<Shape<T, U>>) -> Self {
    Self { shapes: geometry }
  }
}

impl<T, U> Hash for ShapeCollection<T, U>
  where
    T: Scalar + Hash,
    U: Scalar + Unsigned + Hash,
    Shape<T, U>: Hash
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.shapes.hash(state);
  }
}

impl<T, U> Clone for ShapeCollection<T, U>
  where
    T: Scalar + Clone,
    U: Scalar + Unsigned + Clone,
    Shape<T, U>: Clone
{
  fn clone(&self) -> Self {
    Self {
      shapes: self.shapes.clone(),
    }
  }
}

impl<T, U> PartialEq for ShapeCollection<T, U>
  where
    T: Scalar + PartialEq,
    U: Scalar + Unsigned + PartialEq,
    Shape<T, U>: PartialEq
{
  fn eq(&self, other: &Self) -> bool {
    self.shapes == other.shapes
  }
}

impl<T, U> Eq for ShapeCollection<T, U>
  where
    T: Scalar + Eq,
    U: Scalar + Unsigned + Eq,
    Shape<T, U>: Eq
{}

impl ShapeCollection<u8, u8>
{
  pub fn center(&self) -> Point2<u8> {
    let mut center = Vector2::new(0., 0.);
    for geometry in &self.shapes {
      center += geometry.center().coords.map(|x| x as f64);
    }
    center.div(self.shapes.len() as f64).map(|x| x as u8).into()
  }

  pub fn bbox(&self) -> Rectangle<u8> {
    let bboxes = self.shapes.iter().map(|x| x.bbox());

    let mut min = Point2::new(u8::MAX, u8::MAX);
    let mut max = Point2::new(u8::MIN, u8::MIN);

    for bbox in bboxes {
      min = Point2::new(min.x.min(bbox.min().x), min.y.min(bbox.min().y));
      max = Point2::new(max.x.max(bbox.max().x), max.y.max(bbox.max().y));
    }

    Rectangle::new(min, max)
  }

  pub fn points_inside(&self) -> Vec<Point2<u8>> {
    let mut points = Vec::new();
    for geometry in &self.shapes {
      points.extend(geometry.points_inside());
    }
    points
      .into_iter()
      .collect::<HashSet<_>>()
      .into_iter()
      .collect()
  }
}

#[cfg(test)]
mod tests {}