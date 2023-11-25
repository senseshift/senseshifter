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

use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use derivative::Derivative;
use nalgebra::{Point2, Scalar};
use num::Unsigned;

#[derive(Derivative)]
#[derivative(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Shape<T, U>
  where
    T: Scalar,
    U: Scalar + Unsigned,
{
  Circle(Circle<T, U>),
  Ellipse(Ellipse<T, U>),
  Rectangle(Rectangle<T>),
  Triangle(Triangle<T>),
  Collection(ShapeCollection<T, U>),
}

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

  pub fn bbox(&self) -> Rectangle<u8> {
    match self {
      Self::Ellipse(ellipse) => ellipse.bbox().clone(),
      Self::Circle(circle) => circle.bbox().clone(),
      Self::Rectangle(rectangle) => rectangle.clone(),
      Self::Triangle(triangle) => triangle.bbox(),
      Self::Collection(collection) => collection.bbox(),
    }
  }

  pub fn points_inside(&self) -> Vec<Point2<u8>> {
    match self {
      Shape::Ellipse(ellipse) => ellipse.points_inside(),
      Shape::Circle(circle) => circle.points_inside(),
      Shape::Rectangle(rectangle) => rectangle.points_inside(),
      Shape::Triangle(triangle) => triangle.points_inside(),
      Shape::Collection(collection) => collection.points_inside(),
    }
  }
}