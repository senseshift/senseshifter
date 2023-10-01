use std::fmt::Debug;
use crate::*;

use proptest::arbitrary::*;
use proptest::collection::*;
use proptest::prelude::*;
use proptest::strategy::*;
use proptest::test_runner::*;

type Mapped<I, O> = Map<StrategyFor<I>, fn(_: I) -> O>;
type FilterMapped<I, O> = FilterMap<StrategyFor<I>, fn(_: I) -> Option<O>>;

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Point

#[derive(Debug)]
pub struct PointView<T: Scalar + Debug, const N: usize>(pub Point<T, N>);

impl<T, const N: usize> Into<Point<T, N>> for PointView<T, N>
  where T: Scalar + Debug + Clone + PartialEq
{
  fn into(self) -> Point<T, N> {
    self.0
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
      .prop_map(|vec: Vec<T>| PointView([vec[0].clone(), vec[1].clone()].into()))
  }
  type Strategy = proptest::arbitrary::Mapped<Vec<T>, PointView<T, 2>>;
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Circle

impl<T> Arbitrary for Circle<T>
  where
    T::Strategy: Clone,
    T::Parameters: Clone,
    T: Arbitrary + Scalar + Clone,
{
  type Parameters = T::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    any_with::<(PointView<T, 2>, T)>((params.clone(), params.clone()))
      .prop_map(|(center, radius)| Circle { center: center.into(), radius })
  }
  type Strategy = Mapped<(PointView<T, 2>, T), Circle<T>>;
}

///////////////////////////////////////////////////////////////////////////////
// Arbitrary Ellipse

impl<T: Arbitrary> Arbitrary for Ellipse<T>
  where
    T::Strategy: Clone,
    T::Parameters: Clone,
    T: Arbitrary + Scalar + Clone,
{
  type Parameters = T::Parameters;
  fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
    any_with::<(PointView<T, 2>, (T, T))>((params.clone(), (params.clone(), params.clone())))
      .prop_map(|(center, radius)| Ellipse { center: center.into(), radius })
  }
  type Strategy = Mapped<(PointView<T, 2>, (T, T)), Ellipse<T>>;
}