pub use nalgebra::geometry::*;
use nalgebra::Scalar;
use num::cast::AsPrimitive;
use num::Num;
use num::traits::NumOps;

mod circle;
mod ellipse;
mod line;
mod rectangle;
mod triangle;

pub use circle::Circle;
pub use ellipse::Ellipse;
pub use line::Line;
pub use rectangle::Rectangle;
pub use triangle::Triangle;

mod within;
mod distance;

pub use within::*;
pub use distance::*;

#[cfg(test)]
pub mod testing;
