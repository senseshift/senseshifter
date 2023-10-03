use std::fmt::Debug;
use num::Unsigned;
use crate::*;
use nalgebra::*;
use array_init::{array_init, try_array_init};

use proptest::arbitrary::*;
use proptest::collection::*;
use proptest::prelude::*;
use proptest::strategy::*;
use proptest::test_runner::*;

type Mapped<I, O> = Map<StrategyFor<I>, fn(_: I) -> O>;
type FilterMapped<I, O> = FilterMap<StrategyFor<I>, fn(_: I) -> Option<O>>;

///////////////////////////////////////////////////////////////////////////////
// Shrinkable point

#[derive(Clone)]
pub struct ShrinkablePoint<T, const N: usize>
  where
    T: Scalar + Debug
{
  point: Point<T, N>,
  shrink: usize,
  prev_shrink: Option<usize>,
}
impl<T, const N: usize> ValueTree for ShrinkablePoint<T, N>
  where
    T: Scalar + Debug + ValueTree,
    <T as ValueTree>::Value: Clone + PartialEq
{
  type Value = Point<<T as ValueTree>::Value, N>;

  fn current(&self) -> Point<T::Value, N> {
    Point::from(array_init(|i| self.point.coords.index(i).current()))
  }

  fn simplify(&mut self) -> bool {
    for ix in self.shrink..N {
      if !self.point.coords.index_mut(ix).simplify() {
        self.shrink = ix + 1;
      } else {
        self.prev_shrink = Some(ix);
        return true;
      }
    }
    false
  }
  fn complicate(&mut self) -> bool {
    match self.prev_shrink {
      None => false,
      Some(ix) => {
        if self.point.coords.index_mut(ix).complicate() {
          true
        } else {
          self.prev_shrink = None;
          false
        }
      }
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Point strategy

pub struct PointView<T: Scalar, const N: usize>(pub Point<T, N>);

impl<T, const N: usize> Debug for PointView<T, N>
  where T: Scalar + Debug + Clone + PartialEq
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("PointView")
      .field(&self.0)
      .finish()
  }
}

impl<T, const N: usize> Strategy for PointView<T, N>
  where
    T: Scalar + Clone + Debug + Strategy,
    <T as Strategy>::Tree: Clone + Debug + PartialEq,
    <T as Strategy>::Value: Clone + PartialEq,
{
  type Tree = ShrinkablePoint<T::Tree, N>;
  type Value = Point<<T as Strategy>::Value, N>;
  fn new_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
    let tree = ShrinkablePoint {
      point: Point::from(try_array_init(|i| self.0.coords.index(i).new_tree(runner))?),
      shrink: 0,
      prev_shrink: None,
    };

    Ok(tree)
  }
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Point

impl<T, const N: usize> Into<Point<T, N>> for PointView<T, N>
  where T: Scalar + Debug + Clone + PartialEq
{
  fn into(self) -> Point<T, N> {
    self.0
  }
}

impl<T, const N: usize> From<[T; N]> for PointView<T, N>
  where T: Scalar + Debug + Clone + PartialEq
{
  fn from(array: [T; N]) -> Self {
    Self(Point::from(array))
  }
}

impl<T> Arbitrary for PointView<T, 2>
  where
    T::Strategy: Clone,
    T::Parameters: Clone,
    T: Arbitrary + Scalar + Clone,
{
  type Parameters = T::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    vec(any_with::<T>(params), 2)
      .prop_map(|vec: Vec<T>| [vec[0].clone(), vec[1].clone()].into())
  }
  type Strategy = proptest::arbitrary::Mapped<Vec<T>, PointView<T, 2>>;
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Circle

impl<T, R> Arbitrary for Circle<T, R>
  where
    T::Strategy: Clone,
    T::Parameters: Clone,
    T: Arbitrary + Scalar + Clone,
    R::Strategy: Clone,
    R::Parameters: Clone,
    R: Arbitrary + Scalar + Unsigned + Clone,
{
  type Parameters = <(T, R) as Arbitrary>::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    any_with::<(PointView<T, 2>, R)>((params.0.clone(), params.1.clone()))
      .prop_map(|(center, radius)| Circle { center: center.into(), radius })
  }
  type Strategy = Mapped<(PointView<T, 2>, R), Circle<T, R>>;
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Ellipse

impl<T, R> Arbitrary for Ellipse<T, R>
  where
    T::Strategy: Clone,
    T::Parameters: Clone,
    T: Arbitrary + Scalar + Clone,
    R::Strategy: Clone,
    R::Parameters: Clone,
    R: Arbitrary + Scalar + Unsigned + Clone,
{
  type Parameters = <(T, R) as Arbitrary>::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    any_with::<(PointView<T, 2>, (R, R))>((params.0.clone(), (params.1.clone(), params.1.clone())))
      .prop_map(|(center, radius)| Ellipse { center: center.into(), radius })
  }
  type Strategy = Mapped<(PointView<T, 2>, (R, R)), Ellipse<T, R>>;
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Line

impl<T> Arbitrary for Line<T>
  where
    T::Strategy: Clone,
    T::Parameters: Clone,
    T: Arbitrary + Scalar + Clone,
{
  type Parameters = <(PointView<T, 2>, PointView<T, 2>) as Arbitrary>::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    any_with::<(PointView<T, 2>, PointView<T, 2>)>(params)
      .prop_map(|(p1, p2)| Line::new(p1.into(), p2.into()))
  }
  type Strategy = Mapped<(PointView<T, 2>, PointView<T, 2>), Line<T>>;
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Rectangle

impl<T> Arbitrary for Rectangle<T>
  where
    T::Strategy: Clone,
    T::Parameters: Clone,
    T: Arbitrary + Scalar + Copy + Ord,
{
  type Parameters = <(PointView<T, 2>, PointView<T, 2>) as Arbitrary>::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    any_with::<(PointView<T, 2>, PointView<T, 2>)>(params)
      .prop_map(|(p1, p2)| Rectangle::new(p1.into(), p2.into()))
  }
  type Strategy = Mapped<(PointView<T, 2>, PointView<T, 2>), Rectangle<T>>;
}
