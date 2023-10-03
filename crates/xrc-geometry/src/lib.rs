pub use nalgebra::geometry::*;
use nalgebra::Scalar;
use num::cast::AsPrimitive;
use num::Num;
use num::traits::NumOps;

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
