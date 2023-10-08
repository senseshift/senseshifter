pub use nalgebra::geometry::*;
use nalgebra::Scalar;
use num::cast::AsPrimitive;
use num::Num;
use num::traits::NumOps;

// todo: support unsigned integers
pub trait FloatMath: Scalar + NumOps + PartialOrd + Copy + Into<f64> {}
impl FloatMath for u8 {}
impl FloatMath for u16 {}
impl FloatMath for u32 {}
impl FloatMath for f32 {}
impl FloatMath for f64 {}

mod data;
mod shape;

pub use data::*;
pub use shape::*;

mod within;
mod distance;

pub use within::*;
pub use distance::*;

#[cfg(test)]
pub mod testing;
