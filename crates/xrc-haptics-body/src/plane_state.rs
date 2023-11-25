use nalgebra::{Point2, Scalar, Vector2};
use num::{Num, Zero};
use rayon::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct PlaneState<T, const D: usize>(
  pub [[T; D]; D]
);

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

  pub fn vector_index<P>(point: Vector2<P>) -> usize
    where
      P: Num + Into<usize> + Copy,
  {
    point[0].into() * D + point[1].into()
  }

  pub fn index_coords(index: usize) -> (usize, usize) {
    let x = index / D;
    let y = index % D;

    (x, y)
  }
}

impl<T, const D: usize> std::ops::Index<[usize; 2]> for PlaneState<T, D>
{
  type Output = T;

  fn index(&self, index: [usize; 2]) -> &Self::Output {
    let (x, y) = (index[0], index[1]);
    &self.0[x][y]
  }
}

impl<T, const D: usize> std::ops::IndexMut<[usize; 2]> for PlaneState<T, D>
{
  fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
    let (x, y) = (index[0], index[1]);
    &mut self.0[x][y]
  }
}

impl<T, const D: usize> std::ops::Index<(usize, usize)> for PlaneState<T, D>
{
  type Output = T;

  fn index(&self, index: (usize, usize)) -> &Self::Output {
    let (x, y) = index;
    &self.0[x][y]
  }
}

impl<T, const D: usize> std::ops::IndexMut<(usize, usize)> for PlaneState<T, D>
{
  fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
    let (x, y) = index;
    &mut self.0[x][y]
  }
}

impl<T, const D: usize> std::ops::Index<usize> for PlaneState<T, D>
{
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    let (x, y) = Self::index_coords(index);
    &self.0[x][y]
  }
}

impl<T, const D: usize> std::ops::IndexMut<usize> for PlaneState<T, D>
{
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    let (x, y) = Self::index_coords(index);
    &mut self.0[x][y]
  }
}

impl<T, const D: usize, P> std::ops::Index<Vector2<P>> for PlaneState<T, D>
  where
    P: Num + Into<usize> + Copy,
{
  type Output = T;

  fn index(&self, index: Vector2<P>) -> &Self::Output {
    let (x, y) = (index[0].into(), index[1].into());
    &self.0[x][y]
  }
}

impl<T, const D: usize, P> std::ops::IndexMut<Vector2<P>> for PlaneState<T, D>
  where
    P: Num + Into<usize> + Copy,
{
  fn index_mut(&mut self, index: Vector2<P>) -> &mut Self::Output {
    let (x, y) = (index[0].into(), index[1].into());
    &mut self.0[x][y]
  }
}

impl<T, const D: usize, P> std::ops::Index<Point2<P>> for PlaneState<T, D>
  where
    P: Scalar + Num + Into<usize> + Copy,
{
  type Output = T;

  fn index(&self, index: Point2<P>) -> &Self::Output {
    let (x, y) = (index.x.into(), index.y.into());
    &self.0[x][y]
  }
}

impl<T, const D: usize, P> std::ops::IndexMut<Point2<P>> for PlaneState<T, D>
  where
    P: Scalar + Num + Into<usize> + Copy,
{
  fn index_mut(&mut self, index: Point2<P>) -> &mut Self::Output {
    let (x, y) = (index.x.into(), index.y.into());
    &mut self.0[x][y]
  }
}

// impl<T, const D: usize> IntoIterator for PlaneState<T, D> {
//   type Item = T;
// }
//
// impl<'a, T, const D: usize> IntoIterator for &'a PlaneState<T, D> {
//   type Item = T;
//
// }
//
// impl<'a, T, const D: usize> IntoIterator for &'a mut PlaneState<T, D> {
//   type Item = T;
//
// }

impl<T, const D: usize> PlaneState<T, D>
  where
    T: Copy + Send + Sync,
{
  pub fn iter(&self) -> impl Iterator<Item = &T> {
    self.0.iter().flatten()
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
    self.0.iter_mut().flatten()
  }

  pub fn par_iter(&self) -> impl ParallelIterator<Item = &T> {
    self.0.par_iter().flatten()
  }

  pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T> {
    self.0.par_iter_mut().flatten()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_index_usize() {
    let state = PlaneState::new([[0, 1], [2, 3]]);

    assert_eq!(state[0], 0);
    assert_eq!(state[1], 1);
    assert_eq!(state[2], 2);
    assert_eq!(state[3], 3);
  }

  #[test]
  fn test_index_vector2() {
    let state = PlaneState::new([[0, 1], [2, 3]]);

    assert_eq!(state[Vector2::<u8>::new(0, 0)], 0);
    assert_eq!(state[Vector2::<u8>::new(0, 1)], 1);
    assert_eq!(state[Vector2::<u8>::new(1, 0)], 2);
    assert_eq!(state[Vector2::<u8>::new(1, 1)], 3);
  }

  #[test]
  fn test_index_point2() {
    let state = PlaneState::new([[0, 1], [2, 3]]);

    assert_eq!(state[Point2::<u8>::new(0, 0)], 0);
    assert_eq!(state[Point2::<u8>::new(0, 1)], 1);
    assert_eq!(state[Point2::<u8>::new(1, 0)], 2);
    assert_eq!(state[Point2::<u8>::new(1, 1)], 3);
  }

  #[test]
  fn test_iter() {
    let mut state = PlaneState::new([[0, 1], [2, 3]]);

    let mut iter = state.iter();

    assert_eq!(iter.next(), Some(&0));
    assert_eq!(iter.next(), Some(&1));
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.next(), None);

    drop(iter);

    for mut item in state.iter_mut() {
      *item += 1;
    }

    assert_eq!(state[0], 1);
    assert_eq!(state[1], 2);
    assert_eq!(state[2], 3);
    assert_eq!(state[3], 4);
  }
}