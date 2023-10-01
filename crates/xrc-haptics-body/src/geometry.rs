use std::hash::Hash;
use xrc_geometry::{Within, Point, Circle, Ellipse, Scalar};

#[derive(Debug, Clone)]
pub enum ActuatorGeometry<T>
  where T: Scalar
{
  Ellipse(Ellipse<T>),
  Circle(Circle<T>),
}

impl ActuatorGeometry<u8>
{
  pub fn center(&self) -> Point<u8, 2> {
    match self {
      Self::Ellipse(ellipse) => ellipse.center().clone(),
      Self::Circle(circle) => circle.center().clone(),
    }
  }
}

impl Within<&Point<u8, 2>> for ActuatorGeometry<u8>
{
  type Result = bool;
  fn within(&self, point: &Point<u8, 2>) -> Self::Result {
    match self {
      Self::Ellipse(ellipse) => ellipse.within(point),
      Self::Circle(circle) => circle.within(point),
    }
  }
}

impl Within<Point<u8, 2>> for ActuatorGeometry<u8>
{
  type Result = bool;
  fn within(&self, point: Point<u8, 2>) -> Self::Result {
    self.within(&point)
  }
}

impl<T> From<Ellipse<T>> for ActuatorGeometry<T>
  where T: Scalar + Clone
{
  fn from(ellipse: Ellipse<T>) -> Self {
    Self::Ellipse(ellipse)
  }
}

impl<T> From<Circle<T>> for ActuatorGeometry<T>
  where T: Scalar + Clone
{
  fn from(circle: Circle<T>) -> Self {
    Self::Circle(circle)
  }
}

impl<T> PartialEq for ActuatorGeometry<T>
  where T: Scalar + PartialEq
{
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Ellipse(ellipse), Self::Ellipse(other_ellipse)) => ellipse.eq(other_ellipse),
      (Self::Circle(circle), Self::Circle(other_circle)) => circle.eq(other_circle),
      _ => false,
    }
  }
}

impl<T> Eq for ActuatorGeometry<T>
  where T: Scalar + Eq
{}

impl<T> Hash for ActuatorGeometry<T>
  where T: Scalar + Hash
{
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Self::Ellipse(ellipse) => ellipse.hash(state),
      Self::Circle(circle) => circle.hash(state),
    }
  }
}