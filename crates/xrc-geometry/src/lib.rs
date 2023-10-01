pub use nalgebra::*;

mod circle;
mod ellipse;

pub use ellipse::Ellipse;
pub use circle::Circle;

mod within;

pub use within::Within;

#[cfg(test)]
pub mod testing;
