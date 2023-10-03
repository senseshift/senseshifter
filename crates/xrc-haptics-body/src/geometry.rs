use std::hash::{Hash, Hasher};
use std::ops::Div;
use num::{Num, Unsigned};
use nalgebra::{Point2, Scalar, Vector2};
use xrc_geometry::{Within, PointWithin, Point, Circle, Ellipse, Rectangle, Triangle, Distance};

#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum ActuatorGeometry<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
{
  Ellipse(Ellipse<T, U>),
  Circle(Circle<T, U>),
  Rectangle(Rectangle<T>),
  Triangle(Triangle<T>),
  Collection(GeometryCollection<T, U>),
}

#[derive(Debug, Clone)]
pub struct GeometryCollection<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
{
  pub geometry: Vec<ActuatorGeometry<T, U>>,
}

impl<T, U> Within<&Point2<T>> for GeometryCollection<T, U>
  where
    T: Scalar + Clone,
    U: Scalar + Unsigned,
    Ellipse<T, U>: for<'a> Within<&'a Point2<T>, Result=PointWithin>,
    Circle<T, U>: for<'a> Within<&'a Point2<T>, Result=PointWithin>,
    Rectangle<T>: for<'a> Within<&'a Point2<T>, Result=PointWithin>,
    Triangle<T>: for<'a> Within<&'a Point2<T>, Result=PointWithin>,
{
  type Result = PointWithin;
  fn within(&self, point: &Point2<T>) -> Self::Result {
    for geometry in &self.geometry {
      if geometry.within(point) == PointWithin::Inside {
        return PointWithin::Inside;
      }
    }
    PointWithin::Outside
  }
}

impl Distance<&Point2<u8>> for GeometryCollection<u8, u8>
{
  type Result = f64;

  fn distance(&self, point: &Point2<u8>) -> f64 {
    self.geometry.clone()
      .into_iter()
      .map(|geometry| geometry.distance(point))
      .min_by(|a, b| a.partial_cmp(b).unwrap())
      .unwrap()
  }
}

impl GeometryCollection<u8, u8>
{
  pub fn center(&self) -> Point2<u8> {
    let mut center = Vector2::new(0., 0.);
    for geometry in &self.geometry {
      center += geometry.center().coords.map(|x| x as f64);
    }
    center.div(self.geometry.len() as f64).map(|x| x as u8).into()
  }
}

impl<T, U> Hash for GeometryCollection<T, U>
  where
    T: Scalar + Hash,
    U: Scalar + Unsigned + Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.geometry.hash(state)
  }
}

// impl<T, R> Copy for ActuatorGeometry<T, R>
//   where
//     T: Scalar + Copy,
//     R: Scalar + Unsigned + Copy,
// {}

impl<T, R> Hash for ActuatorGeometry<T, R>
  where
    T: Scalar + Hash,
    R: Scalar + Unsigned + Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      Self::Ellipse(ellipse) => {
        state.write_u8(0);
        ellipse.hash(state)
      }
      Self::Circle(circle) => {
        state.write_u8(1);
        circle.hash(state)
      }
      Self::Rectangle(rectangle) => {
        state.write_u8(2);
        rectangle.hash(state)
      }
      Self::Triangle(triangle) => {
        state.write_u8(3);
        triangle.hash(state)
      },
      Self::Collection(collection) => {
        state.write_u8(4);
        collection.hash(state)
      }
    }
  }
}

impl<T, R> PartialEq for ActuatorGeometry<T, R>
  where
    T: Scalar + PartialEq,
    R: Scalar + Unsigned + PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Ellipse(ellipse), Self::Ellipse(other_ellipse)) => ellipse.eq(other_ellipse),
      (Self::Circle(circle), Self::Circle(other_circle)) => circle.eq(other_circle),
      (Self::Rectangle(rectangle), Self::Rectangle(other_rectangle)) => rectangle.eq(other_rectangle),
      (Self::Triangle(triangle), Self::Triangle(other_triangle)) => triangle.eq(other_triangle),
      _ => false,
    }
  }
}

impl<T, R> Eq for ActuatorGeometry<T, R>
  where
    T: Scalar + Eq,
    R: Scalar + Unsigned + Eq,
{}

impl ActuatorGeometry<u8, u8>
{
  pub fn center(&self) -> Point2<u8> {
    match self {
      Self::Ellipse(ellipse) => ellipse.center().clone(),
      Self::Circle(circle) => circle.center().clone(),
      Self::Rectangle(rectangle) => rectangle.center(),
      Self::Triangle(triangle) => triangle.center(),
      Self::Collection(collection) => collection.center(),
    }
  }
}

impl<T, R> Within<&Point<T, 2>> for ActuatorGeometry<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned,
    Ellipse<T, R>: for<'a> Within<&'a Point<T, 2>, Result=PointWithin>,
    Circle<T, R>: for<'a> Within<&'a Point<T, 2>, Result=PointWithin>,
    Rectangle<T>: for<'a> Within<&'a Point<T, 2>, Result=PointWithin>,
    Triangle<T>: for<'a> Within<&'a Point<T, 2>, Result=PointWithin>,
{
  type Result = PointWithin;
  fn within(&self, point: &Point<T, 2>) -> Self::Result {
    match self {
      Self::Ellipse(ellipse) => ellipse.within(point),
      Self::Circle(circle) => circle.within(point),
      Self::Rectangle(rectangle) => rectangle.within(point),
      Self::Triangle(triangle) => triangle.within(point),
      Self::Collection(collection) => collection.within(point),
      _ => PointWithin::Outside,
    }
  }
}

impl<T, R> From<Circle<T, R>> for ActuatorGeometry<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned,
{
  fn from(circle: Circle<T, R>) -> Self {
    Self::Circle(circle)
  }
}

impl<T, R> From<Ellipse<T, R>> for ActuatorGeometry<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned,
{
  fn from(ellipse: Ellipse<T, R>) -> Self {
    Self::Ellipse(ellipse)
  }
}

impl<T, R> From<Rectangle<T>> for ActuatorGeometry<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned,
{
  fn from(rectangle: Rectangle<T>) -> Self {
    Self::Rectangle(rectangle)
  }
}

impl<T, R> From<Triangle<T>> for ActuatorGeometry<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned,
{
  fn from(triangle: Triangle<T>) -> Self {
    Self::Triangle(triangle)
  }
}

impl Distance<&Point2<u8>> for ActuatorGeometry<u8, u8>
{
  type Result = f64;

  fn distance(&self, point: &Point2<u8>) -> f64 {
    match self {
      Self::Ellipse(ellipse) => ellipse.distance(point),
      Self::Circle(circle) => circle.distance(point),
      Self::Rectangle(rectangle) => rectangle.distance(point),
      Self::Triangle(triangle) => triangle.distance(point),
      Self::Collection(collection) => collection.distance(point),
    }
  }
}