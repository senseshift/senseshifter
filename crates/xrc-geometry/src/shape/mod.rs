mod circle;
mod ellipse;
mod rectangle;
mod triangle;
mod shape_collection;

pub use circle::*;
pub use ellipse::*;
pub use rectangle::*;
pub use triangle::*;
pub use shape_collection::*;

use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use nalgebra::{Point2, Scalar};
use num::Unsigned;

pub enum Shape<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
{
  Ellipse(Ellipse<T, U>),
  Circle(Circle<T, U>),
  Rectangle(Rectangle<T>),
  Triangle(Triangle<T>),
  Collection(ShapeCollection<T, U>),
}

impl<T, R> Debug for Shape<T, R>
  where
    T: Scalar + Debug,
    R: Scalar + Debug + Unsigned,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Ellipse(ellipse) => ellipse.fmt(f),
      Self::Circle(circle) => circle.fmt(f),
      Self::Rectangle(rectangle) => rectangle.fmt(f),
      Self::Triangle(triangle) => triangle.fmt(f),
      Self::Collection(collection) => collection.fmt(f),
    }
  }
}

impl<T, R> Hash for Shape<T, R>
  where
    T: Scalar + Hash,
    R: Scalar + Hash + Unsigned,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      Self::Ellipse(ellipse) => {
        state.write_u8(0);
        ellipse.hash(state)
      },
      Self::Circle(circle) => {
        state.write_u8(1);
        circle.hash(state)
      },
      Self::Rectangle(rectangle) => {
        state.write_u8(2);
        rectangle.hash(state)
      },
      Self::Triangle(triangle) => {
        state.write_u8(3);
        triangle.hash(state)
      },
      Self::Collection(collection) => {
        state.write_u8(4);
        collection.hash(state)
      },
    }
  }
}

impl<T, R> Clone for Shape<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Clone + Unsigned,
{
  fn clone(&self) -> Self {
    match self {
      Self::Ellipse(ellipse) => Self::Ellipse(ellipse.clone()),
      Self::Circle(circle) => Self::Circle(circle.clone()),
      Self::Rectangle(rectangle) => Self::Rectangle(rectangle.clone()),
      Self::Triangle(triangle) => Self::Triangle(triangle.clone()),
      Self::Collection(collection) => Self::Collection(collection.clone()),
    }
  }
}

impl<T, R> PartialEq for Shape<T, R>
  where
    T: Scalar + PartialEq,
    R: Scalar + PartialEq + Unsigned,
{
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Ellipse(ellipse), Self::Ellipse(other)) => ellipse == other,
      (Self::Circle(circle), Self::Circle(other)) => circle == other,
      (Self::Rectangle(rectangle), Self::Rectangle(other)) => rectangle == other,
      (Self::Triangle(triangle), Self::Triangle(other)) => triangle == other,
      (Self::Collection(collection), Self::Collection(other)) => collection == other,
      _ => false,
    }
  }
}

impl<T, R> Eq for Shape<T, R>
  where
    T: Scalar + Eq,
    R: Scalar + Eq + Unsigned,
{}


impl<T, R> From<Circle<T, R>> for Shape<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned,
{
  fn from(circle: Circle<T, R>) -> Self {
    Self::Circle(circle)
  }
}

impl<T, R> From<Ellipse<T, R>> for Shape<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned,
{
  fn from(ellipse: Ellipse<T, R>) -> Self {
    Self::Ellipse(ellipse)
  }
}

impl<T, R> From<Rectangle<T>> for Shape<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned,
{
  fn from(rectangle: Rectangle<T>) -> Self {
    Self::Rectangle(rectangle)
  }
}

impl<T, R> From<Triangle<T>> for Shape<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned,
{
  fn from(triangle: Triangle<T>) -> Self {
    Self::Triangle(triangle)
  }
}

impl<T, R> From<ShapeCollection<T, R>> for Shape<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Unsigned,
{
  fn from(collection: ShapeCollection<T, R>) -> Self {
    Self::Collection(collection)
  }
}

impl Shape<u8, u8>
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