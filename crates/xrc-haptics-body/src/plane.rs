use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;
use anyhow::{anyhow, Context, Result};
use dashmap::DashMap;
use getset::Getters;
use num::{Zero, traits::Num, Unsigned};
use nalgebra::Scalar;
use num::traits::NumOps;
use xrc_geometry::{Shape, Point2, distance_squared, Circle};
use crate::{ActuatorEvent, ActuatorSender};

#[repr(transparent)]
pub struct PlaneState<T, const D: usize>(pub [[T; D]; D]);

impl<T, const D: usize> Default for PlaneState<T, D>
  where
    T: Zero + Copy,
{
  fn default() -> Self {
    Self([[T::zero(); D]; D])
  }
}

impl<T, const D: usize> PlaneState<T, D>
{
  pub fn new(source: [[T; D]; D]) -> Self {
    Self(source)
  }
}

impl<T, const D: usize> PlaneState<T, D>
  where
    T: Scalar,
    usize: From<T>,
{
  #[inline]
  pub fn point_coords<B>(point: &Point2<B>) -> (usize, usize)
    where
      B: Scalar + Into<usize> + Copy,
  {
    let x: usize = point.x.into();
    let y: usize = point.y.into();

    return (x, y);
  }

  pub fn set<B>(&mut self, point: &Point2<B>, value: T)
    where
      B: Scalar + Into<usize> + Copy,
  {
    let (x, y) = Self::point_coords::<B>(point);
    self[x][y] = value;
  }
}

impl<T, const D: usize> Index<usize> for PlaneState<T, D>
{
  type Output = [T; D];

  fn index(&self, index: usize) -> &Self::Output {
    &self.0[index]
  }
}

impl<T, const D: usize> IndexMut<usize> for PlaneState<T, D>
{
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.0[index]
  }
}

#[derive(Getters)]
pub struct HapticPlane<T, I, const D: usize, A>
  where
    T: Scalar + Unsigned + Hash + Eq,
    I: Num,
{
  #[getset(get = "pub")]
  actuators: DashMap<Shape<T, T>, A>,

  /// A map of actuator centers to their geometries.
  ///
  /// # Example:
  /// ```rust
  /// use xrc_geometry::{Circle, Shape};
  /// use xrc_haptics_body::{HapticPlaneU8};
  ///
  /// let mut plane = HapticPlaneU8::<()>::default();
  /// let geometry = Circle::new([5, 83].into(), 10);
  /// let geometry = Shape::from(geometry);
  ///
  /// plane.insert(geometry.clone(), ());
  ///
  /// let center = geometry.center();
  /// assert_eq!(plane.centers().get(&center).unwrap().value(), &geometry);
  /// ```
  #[getset(get = "pub")]
  centers: DashMap<Point2<T>, Shape<T, T>>,

  #[getset(get = "pub")]
  closest: [[Point2<T>; D]; D],

  #[getset(get = "pub")]
  intensities: DashMap<Shape<T, T>, I>,

  /// The current state of the plane.
  ///
  /// This is a 2D array of intensities, where X is the first dimension and Y is the second.
  /// <small>To be honest, it should be the other way, but it is easier to use it this way</small>
  ///
  /// # Example:
  /// ```rust
  /// use xrc_haptics_body::PlaneState;
  /// use xrc_geometry::Point2;
  ///
  /// let plane = PlaneState::<u8, 4>::new([
  ///   [0,  1,  2,  3 ],
  ///   [4,  5,  6,  7 ],
  ///   [8,  9,  10, 11],
  ///   [12, 13, 14, 15],
  /// ]);
  ///
  /// assert_eq!(plane[0][0], 0);
  ///
  /// assert_eq!(plane[0][1], 1);
  /// assert_eq!(plane[1][0], 4);
  /// assert_eq!(plane[1][1], 5);
  ///
  /// assert_eq!(plane[2][0], 8);
  /// assert_eq!(plane[0][2], 2);
  /// assert_eq!(plane[2][2], 10);
  /// ```
  #[getset(get = "pub")]
  state: PlaneState<I, D>,
}

impl<T, I, const D: usize, A> Default for HapticPlane<T, I, D, A>
  where
    T: Scalar + Num + Copy + Unsigned + Hash + Eq,
    I: Num + Copy,
{
  fn default() -> Self {
    Self {
      actuators: DashMap::new(),
      centers: DashMap::new(),
      intensities: DashMap::new(),
      closest: [[Point2::new(T::zero(), T::zero()); D]; D],
      state: PlaneState::default(),
    }
  }
}

impl<T, I, const D: usize, A> HapticPlane<T, I, D, A>
  where
    T: Scalar + Unsigned + Num + Hash + Eq + Copy,
    I: Num + Copy,
    usize: From<T>,
{
  #[inline]
  pub fn point_coords(point: &Point2<T>) -> (usize, usize) {
    let x: usize = point.x.into();
    let y: usize = point.y.into();

    return (x, y);
  }

  pub fn state_at(&self, point: &Point2<T>) -> I {
    let (x, y) = Self::point_coords(point);
    return self.state[x][y];
  }
}

/// A haptic plane with all 8-bit values (width, height, and intensity).
pub type HapticPlaneU8<A = ActuatorSender> = HapticPlane<u8, u8, { u8::MAX as usize + 1 }, A>;

impl<A> HapticPlaneU8<A>
{
  /// Inserts an actuator into the plane.
  ///
  /// # Example:
  /// ```rust
  /// use xrc_geometry::{Circle, Shape};
  /// use xrc_haptics_body::{HapticPlaneU8};
  ///
  /// let mut plane = HapticPlaneU8::<()>::default();
  /// let geometry = Circle::new([5, 83].into(), 10);
  ///
  /// plane.insert(Shape::from(geometry), ());
  ///
  /// assert_eq!(plane.actuators().len(), 1);
  /// assert_eq!(plane.centers().len(), 1);
  /// assert_eq!(plane.intensities().len(), 1);
  ///
  /// let center = geometry.center();
  /// assert!(plane.centers().contains_key(&center));
  ///
  /// ```
  pub fn insert(&mut self, geometry: Shape<u8, u8>, sender: A) {
    self.insert_no_recalc(geometry.clone(), sender);
    self.recalc_closest();
  }

  /// **WARNING**: This method does not recalculate the closest actuator for each point.
  /// You should call [`Self::recalc_closest`] after inserting all actuators.
  ///
  /// Inserts an actuator into the plane without recalculating the closest actuator for each point.
  /// This is useful when inserting multiple actuators at once.
  pub fn insert_no_recalc(&mut self, geometry: Shape<u8, u8>, sender: A) {
    self.actuators.insert(geometry.clone(), sender);
    self.centers.insert(geometry.center(), geometry.clone());
    self.intensities.insert(geometry, 0);
  }

  /// Get the center for the closest actuator to the given point.
  pub fn get_closest(&self, point: &Point2<u8>) -> Point2<u8> {
    let (x, y) = Self::point_coords(point);
    return self.closest[x][y];
  }

  /// Get closest actuators for each point in the given circle.
  ///
  /// Returns a vector of tuples, where the first element is the closest point and the second
  /// element is the magnitude of the distance between the point and the closest actuator.
  pub fn get_closest_circle(&self, circle: &Circle<u8, u8>) -> Vec<(Point2<u8>, f64)> {
    let points: Vec<_> = circle.points_inside()
      .into_iter()
      .collect::<HashSet<_>>()
      .into_iter()
      .collect();

    let len = points.len() as f64;

    let mut closest = points
      .into_iter()
      .map(|point| (point, 1.0 / len))
      .collect();

    closest
  }

  /// Computes the center for the closest actuator to the given point.
  pub fn search_closest(&self, point: &Point2<u8>) -> Point2<u8> {
    use xrc_geometry::Distance;

    // // Center of the closest actuator
    // self.centers.iter()
    //   .map(|entry| (entry.key().clone(), entry.key().distance(point)))
    //   .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
    //   .map(|(geometry, _)| geometry)
    //   .unwrap()

    // Accurate
    self.actuators.iter()
      .map(|entry| (entry.key().clone(), entry.key().distance(point)))
      .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
      .map(|(geometry, _)| geometry.center())
      .unwrap()

    // // Center of bbox
    // self.actuators.iter()
    //   .map(|entry| (entry.key().clone().bbox().center(), entry.key().clone().bbox().center().distance(point)))
    //   .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
    //   .map(|(geometry, _)| geometry)
    //   .unwrap()
  }

  pub fn recalc_closest(&mut self) {
    let mut closest = self.closest.clone();

    for (x, row) in self.closest.iter().enumerate() {
      for (y, _) in row.iter().enumerate() {
        let point = Point2::new(x as u8, y as u8);
        closest[x][y] = self.search_closest(&point);
      }
    }

    self.closest = closest;
  }
}

impl HapticPlaneU8<ActuatorSender>
{
  pub fn new() -> Self {
    Self::default()
  }

  pub async fn effect(&mut self, point: &Point2<u8>, effect: ActuatorEvent) -> Result<()> {
    if self.actuators.is_empty() {
      return Err(anyhow!("No actuators in the plane"));
    }

    let (x, y) = Self::point_coords(point);

    return match effect {
      ActuatorEvent::Vibrate(intensity) => {
        self.state.set(point, intensity);

        let closest = self.search_closest(point);
        let geometry = self.centers.get(&closest)
          .context(anyhow!("No geometry for point {}", point))?;
        let actuator = self.actuators.get(&geometry)
          .context(anyhow!("No actuator at known point {}", point))?;

        actuator.send(effect).await?;

        Ok(())
      }
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use xrc_geometry::*;
  use flaky_test::flaky_test;

  #[flaky_test(5)]
  fn test_search_closest() {
    let mut plane = HapticPlaneU8::<()>::default();

    plane.insert(Shape::from(Circle::new([10 , 10 ].into(), 10)), ());
    plane.insert(Shape::from(Circle::new([10 , 245].into(), 10)), ());
    plane.insert(Shape::from(Circle::new([245, 10 ].into(), 10)), ());
    plane.insert(Shape::from(Circle::new([245, 245].into(), 10)), ());

    plane.insert(Shape::from(Circle::new([100, 100].into(), 10)), ());
    plane.insert(Shape::from(Circle::new([100, 155].into(), 10)), ());
    plane.insert(Shape::from(Circle::new([155, 100].into(), 10)), ());
    plane.insert(Shape::from(Circle::new([155, 155].into(), 10)), ());

    assert_eq!(plane.search_closest(&Point2::new(0  , 0  )), Point2::new(10 , 10 ));
    assert_eq!(plane.search_closest(&Point2::new(0  , 255)), Point2::new(10 , 245));
    assert_eq!(plane.search_closest(&Point2::new(255, 0  )), Point2::new(245, 10 ));
    assert_eq!(plane.search_closest(&Point2::new(255, 255)), Point2::new(245, 245));

    assert_eq!(plane.search_closest(&Point2::new(100, 100)), Point2::new(100, 100));
    assert_eq!(plane.search_closest(&Point2::new(100, 200)), Point2::new(100, 155));
    assert_eq!(plane.search_closest(&Point2::new(200, 100)), Point2::new(155, 100));
    assert_eq!(plane.search_closest(&Point2::new(200, 200)), Point2::new(155, 155));

    assert_eq!(plane.search_closest(&Point2::new(127, 127)), Point2::new(100, 100));
    assert_eq!(plane.search_closest(&Point2::new(127, 128)), Point2::new(100, 155));
    assert_eq!(plane.search_closest(&Point2::new(128, 127)), Point2::new(155, 100));
    assert_eq!(plane.search_closest(&Point2::new(128, 128)), Point2::new(155, 155));
  }
}