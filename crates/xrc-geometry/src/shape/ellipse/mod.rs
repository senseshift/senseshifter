use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use crate::*;
use getset::Getters;
use num::{Unsigned, Zero};
use nalgebra::*;
use ordered_float::OrderedFloat;

#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Getters)]
pub struct Ellipse<T: Scalar, R: Scalar> {
  #[getset(get = "pub")]
  pub(crate) center: Point2<T>,
  #[getset(get = "pub")]
  pub(crate) radius: (R, R),
}

impl<T, R> Default for Ellipse<T, R>
  where
    T: Scalar + Default + Zero,
    R: Scalar + Default + Zero,
{
  fn default() -> Self {
    Self {
      center: Point2::default(),
      radius: (R::default(), R::default()),
    }
  }
}

impl<T, R> Debug for Ellipse<T, R>
  where
    T: Scalar + Debug,
    R: Scalar + Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Ellipse")
      .field("center", &self.center)
      .field("radius", &self.radius)
      .finish()
  }
}

impl<T, R> Ellipse<T, R>
  where
    T: Scalar,
    R: Scalar,
{
  pub fn new(center: Point2<T>, radius: (R, R)) -> Self {
    Self { center, radius }
  }

  pub fn width(&self) -> &R {
    &self.radius.0
  }

  pub fn height(&self) -> &R {
    &self.radius.1
  }
}

impl<T, R> Hash for Ellipse<T, R>
  where
    T: Scalar + Hash,
    R: Scalar + Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.center.hash(state);
    self.radius.hash(state);
  }
}

impl<T, R> Copy for Ellipse<T, R>
  where
    T: Scalar + Copy,
    R: Scalar + Copy,
{}

impl<T, R> Clone for Ellipse<T, R>
  where
    T: Scalar + Clone,
    R: Scalar + Clone,
{
  fn clone(&self) -> Self {
    Self {
      center: self.center.clone(),
      radius: self.radius.clone(),
    }
  }
}

impl<T, R> PartialEq for Ellipse<T, R>
  where
    T: Scalar + PartialEq,
    R: Scalar + PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.center == other.center && self.radius == other.radius
  }
}

impl<T, R> Eq for Ellipse<T, R>
  where
    T: Scalar + Eq,
    R: Scalar + Eq,
{}

impl Ellipse<u8, u8> {
  pub fn point_intersection(&self, point: &Point2<u8>, max_iterations: usize) -> Point2<f64> {
    let a = self.radius.0 as f64;
    let b = self.radius.1 as f64;

    let epsilon = 0.1 / a.max(b);

    let dx = point.x as f64 - self.center.x as f64;
    let dy = point.y as f64 - self.center.y as f64;
    let p1 = Point2::new(dx, dy);

    // Intersection of straight line from origin to p with ellipse as the first approximation:
    let mut phi = (a * p1.y).atan2(b * p1.x);

    // Newton iteration to find solution of
    // f(θ) := (a^2 − b^2) cos(phi) sin(phi) − x a sin(phi) + y b cos(phi) = 0:
    for _ in 0..max_iterations {
      let sin_phi = phi.sin();
      let cos_phi = phi.cos();

      let f = (a * a - b * b) * cos_phi * sin_phi - dx * a * sin_phi + dy * b * cos_phi;
      let f1 = (a * a - b * b) * (cos_phi * cos_phi - sin_phi * sin_phi) - p1.x * a * cos_phi - p1.y * b * sin_phi;

      let delta = f / f1;
      phi = phi - delta;
      if delta.abs() < epsilon {
        break;
      }
    }

    let x = a * phi.cos() + self.center.x as f64;
    let y = b * phi.sin() + self.center.y as f64;

    Point2::new(x, y)
  }

  pub fn bbox(&self) -> Rectangle<u8> {
    let x = self.center.x as f64 - self.radius.0 as f64;
    let y = self.center.y as f64 - self.radius.1 as f64;
    let width = self.radius.0 as f64 * 2.0;
    let height = self.radius.1 as f64 * 2.0;

    Rectangle::new(Point2::new(x, y).map(|c| c as u8), Point2::new(x + width, y + height).map(|c| c as u8))
  }
}

#[cfg(test)]
mod tests {}