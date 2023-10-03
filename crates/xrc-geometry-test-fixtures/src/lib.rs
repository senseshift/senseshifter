use xrc_geometry::{
  Point2, Shape, ShapeCollection, Triangle, Ellipse,
};
use nalgebra::Scalar;
use num::Unsigned;

pub fn hardlight_vest_chest_front() -> Vec<Shape<u8, u8>>
{
  vec![
    // Left Side
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([29, 79].into(), [35, 44].into(), [86, 41].into())),
        Shape::from(Triangle::new([29, 79].into(), [86, 41].into(), [97, 68].into())),
        Shape::from(Triangle::new([29, 79].into(), [97, 68].into(), [85, 73].into())),
        Shape::from(Triangle::new([29, 79].into(), [85, 73].into(), [79, 78].into())),
        Shape::from(Triangle::new([29, 79].into(), [79, 78].into(), [76, 83].into())),
        Shape::from(Triangle::new([29, 79].into(), [76, 83].into(), [75, 89].into())),
        Shape::from(Triangle::new([29, 79].into(), [75, 89].into(), [39, 89].into())),
      ]
    }),
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([45, 107].into(), [84, 107].into(), [44, 113].into())),
        Shape::from(Triangle::new([44, 113].into(), [84, 107].into(), [83, 132].into())),
        Shape::from(Triangle::new([84, 107].into(), [93, 112].into(), [83, 132].into())),
        Shape::from(Triangle::new([93, 112].into(), [100, 114].into(), [83, 132].into())),
        Shape::from(Triangle::new([83, 132].into(), [100, 114].into(), [100, 132].into())),
      ]
    }),
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([41, 134].into(), [76, 151].into(), [45, 143].into())),
        Shape::from(Triangle::new([45, 143].into(), [76, 151].into(), [85, 169].into())),
        Shape::from(Triangle::new([76, 151].into(), [85, 169].into(), [99, 151].into())),
        Shape::from(Triangle::new([85, 169].into(), [99, 151].into(), [99, 169].into())),
      ]
    }),
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([47, 168].into(), [75, 184].into(), [52, 179].into())),
        Shape::from(Triangle::new([52, 179].into(), [75, 184].into(), [81, 202].into())),
        Shape::from(Triangle::new([75, 184].into(), [99, 186].into(), [81, 202].into())),
        Shape::from(Triangle::new([81, 202].into(), [99, 186].into(), [99, 206].into())),
      ]
    }),

    // Right Side
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([255 - 29, 79].into(), [255 - 35, 44].into(), [255 - 86, 41].into())),
        Shape::from(Triangle::new([255 - 29, 79].into(), [255 - 86, 41].into(), [255 - 97, 68].into())),
        Shape::from(Triangle::new([255 - 29, 79].into(), [255 - 97, 68].into(), [255 - 85, 73].into())),
        Shape::from(Triangle::new([255 - 29, 79].into(), [255 - 85, 73].into(), [255 - 79, 78].into())),
        Shape::from(Triangle::new([255 - 29, 79].into(), [255 - 79, 78].into(), [255 - 76, 83].into())),
        Shape::from(Triangle::new([255 - 29, 79].into(), [255 - 76, 83].into(), [255 - 75, 89].into())),
        Shape::from(Triangle::new([255 - 29, 79].into(), [255 - 75, 89].into(), [255 - 39, 89].into())),
      ]
    }),
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([255 - 45, 107].into(), [255 - 84, 107].into(), [255 - 44, 113].into())),
        Shape::from(Triangle::new([255 - 44, 113].into(), [255 - 84, 107].into(), [255 - 83, 132].into())),
        Shape::from(Triangle::new([255 - 84, 107].into(), [255 - 93, 112].into(), [255 - 83, 132].into())),
        Shape::from(Triangle::new([255 - 93, 112].into(), [255 - 100, 114].into(), [255 - 83, 132].into())),
        Shape::from(Triangle::new([255 - 83, 132].into(), [255 - 100, 114].into(), [255 - 100, 132].into())),
      ]
    }),
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([255 - 41, 134].into(), [255 - 76, 151].into(), [255 - 45, 143].into())),
        Shape::from(Triangle::new([255 - 45, 143].into(), [255 - 76, 151].into(), [255 - 85, 169].into())),
        Shape::from(Triangle::new([255 - 76, 151].into(), [255 - 85, 169].into(), [255 - 99, 151].into())),
        Shape::from(Triangle::new([255 - 85, 169].into(), [255 - 99, 151].into(), [255 - 99, 169].into())),
      ]
    }),
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([255 - 47, 168].into(), [255 - 75, 184].into(), [255 - 52, 179].into())),
        Shape::from(Triangle::new([255 - 52, 179].into(), [255 - 75, 184].into(), [255 - 81, 202].into())),
        Shape::from(Triangle::new([255 - 75, 184].into(), [255 - 99, 186].into(), [255 - 81, 202].into())),
        Shape::from(Triangle::new([255 - 81, 202].into(), [255 - 99, 186].into(), [255 - 99, 206].into())),
      ]
    }),

    // Center Piece
    Shape::from(Ellipse::new([128, 88].into(), (18, 10))),
  ]
}