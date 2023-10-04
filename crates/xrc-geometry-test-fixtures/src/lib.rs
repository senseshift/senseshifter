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

pub fn owo_skin_front() -> Vec<Shape<u8, u8>>
{
  vec![
    // Left Side
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([8 + 90, 53 + 5].into(), [8 + 95, 53 + 32].into(), [8 + 44, 53 + 32].into())),
        Shape::from(Triangle::new([8 + 103, 53 + 28].into(), [8 + 95, 53 + 32].into(), [8 + 104, 53 + 12].into())),
        Shape::from(Triangle::new([8 + 95, 53 + 32].into(), [8 + 103, 53 + 28].into(), [8 + 99, 53 + 31].into())),
        Shape::from(Triangle::new([8 + 12, 53 + 1].into(), [8 + 90, 53 + 5].into(), [8 + 44, 53 + 32].into())),
        Shape::from(Triangle::new([8 + 34, 53 + 31].into(), [8 + 12, 53 + 1].into(), [8 + 44, 53 + 32].into())),
        Shape::from(Triangle::new([8 + 95, 53 + 32].into(), [8 + 101, 53 + 9].into(), [8 + 104, 53 + 12].into())),
        Shape::from(Triangle::new([8 + 101, 53 + 9].into(), [8 + 95, 53 + 32].into(), [8 + 90, 53 + 5].into())),
        Shape::from(Triangle::new([8 + 12, 53 + 1].into(), [8 + 25, 53 + 28].into(), [8 + 17, 53 + 24].into())),
        Shape::from(Triangle::new([8 + 25, 53 + 28].into(), [8 + 12, 53 + 1].into(), [8 + 34, 53 + 31].into())),
        Shape::from(Triangle::new([8 + 12, 53 + 20].into(), [8 + 12, 53 + 1].into(), [8 + 17, 53 + 24].into())),
        Shape::from(Triangle::new([8 + 12, 53 + 1].into(), [8 + 12, 53 + 20].into(), [8 + 6, 53 + 15].into())),
        Shape::from(Triangle::new([8 + 4, 53 + 5].into(), [8 + 6, 53 + 15].into(), [8 + 3, 53 + 12].into())),
        Shape::from(Triangle::new([8 + 4, 53 + 5].into(), [8 + 12, 53 + 1].into(), [8 + 6, 53 + 15].into())),
        Shape::from(Triangle::new([8 + 4, 53 + 5].into(), [8 + 3, 53 + 12].into(), [8 + 1, 53 + 9].into())),
        Shape::from(Triangle::new([8 + 97, 53 + 6].into(), [8 + 101, 53 + 9].into(), [8 + 90, 53 + 5].into())),
      ]
    }),
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([48 + 32, 166 + 72].into(), [48 + 10, 166 + 58].into(), [48 + 52, 166 + 67].into())),
        Shape::from(Triangle::new([48 + 51, 166 + 10].into(), [48 + 10, 166 + 58].into(), [48 + 2, 166 + 42].into())),
        Shape::from(Triangle::new([48 + 10, 166 + 58].into(), [48 + 51, 166 + 10].into(), [48 + 52, 166 + 67].into())),
        Shape::from(Triangle::new([48 + 32, 166 + 1].into(), [48 + 51, 166 + 10].into(), [48 + 2, 166 + 42].into())),
        Shape::from(Triangle::new([48 + 3, 166 + 6].into(), [48 + 32, 166 + 1].into(), [48 + 2, 166 + 42].into())),
        Shape::from(Triangle::new([48 + 32, 166 + 1].into(), [48 + 3, 166 + 6].into(), [48 + 10, 166 + 1].into())),
        Shape::from(Triangle::new([48 + 3, 166 + 6].into(), [48 + 6, 166 + 3].into(), [48 + 10, 166 + 1].into())),
        Shape::from(Triangle::new([48 + 10, 166 + 58].into(), [48 + 4, 166 + 50].into(), [48 + 2, 166 + 42].into())),
        Shape::from(Triangle::new([48 + 37, 166 + 74].into(), [48 + 32, 166 + 72].into(), [48 + 52, 166 + 67].into())),
        Shape::from(Triangle::new([48 + 48, 166 + 72].into(), [48 + 37, 166 + 74].into(), [48 + 52, 166 + 67].into())),
        Shape::from(Triangle::new([48 + 37, 166 + 74].into(), [48 + 48, 166 + 72].into(), [48 + 44, 166 + 74].into())),
        Shape::from(Triangle::new([48 + 32, 166 + 1].into(), [48 + 41, 166 + 2].into(), [48 + 51, 166 + 10].into())),
        Shape::from(Triangle::new([48 + 7, 166 + 55].into(), [48 + 4, 166 + 50].into(), [48 + 10, 166 + 58].into())),
        Shape::from(Triangle::new([48 + 51, 166 + 69].into(), [48 + 48, 166 + 72].into(), [48 + 52, 166 + 67].into())),
        Shape::from(Triangle::new([48 + 41, 166 + 2].into(), [48 + 49, 166 + 6].into(), [48 + 51, 166 + 10].into())),
      ]
    }),

    // Right Side
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([255 - 8 - 90, 53 + 5].into(), [255 - 8 - 95, 53 + 32].into(), [255 - 8 - 44, 53 + 32].into())),
        Shape::from(Triangle::new([255 - 8 - 103, 53 + 28].into(), [255 - 8 - 95, 53 + 32].into(), [255 - 8 - 104, 53 + 12].into())),
        Shape::from(Triangle::new([255 - 8 - 95, 53 + 32].into(), [255 - 8 - 103, 53 + 28].into(), [255 - 8 - 99, 53 + 31].into())),
        Shape::from(Triangle::new([255 - 8 - 12, 53 + 1].into(), [255 - 8 - 90, 53 + 5].into(), [255 - 8 - 44, 53 + 32].into())),
        Shape::from(Triangle::new([255 - 8 - 34, 53 + 31].into(), [255 - 8 - 12, 53 + 1].into(), [255 - 8 - 44, 53 + 32].into())),
        Shape::from(Triangle::new([255 - 8 - 95, 53 + 32].into(), [255 - 8 - 101, 53 + 9].into(), [255 - 8 - 104, 53 + 12].into())),
        Shape::from(Triangle::new([255 - 8 - 101, 53 + 9].into(), [255 - 8 - 95, 53 + 32].into(), [255 - 8 - 90, 53 + 5].into())),
        Shape::from(Triangle::new([255 - 8 - 12, 53 + 1].into(), [255 - 8 - 25, 53 + 28].into(), [255 - 8 - 17, 53 + 24].into())),
        Shape::from(Triangle::new([255 - 8 - 25, 53 + 28].into(), [255 - 8 - 12, 53 + 1].into(), [255 - 8 - 34, 53 + 31].into())),
        Shape::from(Triangle::new([255 - 8 - 12, 53 + 20].into(), [255 - 8 - 12, 53 + 1].into(), [255 - 8 - 17, 53 + 24].into())),
        Shape::from(Triangle::new([255 - 8 - 12, 53 + 1].into(), [255 - 8 - 12, 53 + 20].into(), [255 - 8 - 6, 53 + 15].into())),
        Shape::from(Triangle::new([255 - 8 - 4, 53 + 5].into(), [255 - 8 - 6, 53 + 15].into(), [255 - 8 - 3, 53 + 12].into())),
        Shape::from(Triangle::new([255 - 8 - 4, 53 + 5].into(), [255 - 8 - 12, 53 + 1].into(), [255 - 8 - 6, 53 + 15].into())),
        Shape::from(Triangle::new([255 - 8 - 4, 53 + 5].into(), [255 - 8 - 3, 53 + 12].into(), [255 - 8 - 1, 53 + 9].into())),
        Shape::from(Triangle::new([255 - 8 - 97, 53 + 6].into(), [255 - 8 - 101, 53 + 9].into(), [255 - 8 - 90, 53 + 5].into())),
      ]
    }),
    Shape::from(ShapeCollection {
      shapes: vec![
        Shape::from(Triangle::new([255 - 48 -32, 166 + 72].into(), [255 - 48 -10, 166 + 58].into(), [255 - 48 -52, 166 + 67].into())),
        Shape::from(Triangle::new([255 - 48 -51, 166 + 10].into(), [255 - 48 -10, 166 + 58].into(), [255 - 48 -2, 166 + 42].into())),
        Shape::from(Triangle::new([255 - 48 -10, 166 + 58].into(), [255 - 48 -51, 166 + 10].into(), [255 - 48 -52, 166 + 67].into())),
        Shape::from(Triangle::new([255 - 48 -32, 166 + 1].into(), [255 - 48 -51, 166 + 10].into(), [255 - 48 -2, 166 + 42].into())),
        Shape::from(Triangle::new([255 - 48 -3, 166 + 6].into(), [255 - 48 -32, 166 + 1].into(), [255 - 48 -2, 166 + 42].into())),
        Shape::from(Triangle::new([255 - 48 -32, 166 + 1].into(), [255 - 48 -3, 166 + 6].into(), [255 - 48 -10, 166 + 1].into())),
        Shape::from(Triangle::new([255 - 48 -3, 166 + 6].into(), [255 - 48 -6, 166 + 3].into(), [255 - 48 -10, 166 + 1].into())),
        Shape::from(Triangle::new([255 - 48 -10, 166 + 58].into(), [255 - 48 -4, 166 + 50].into(), [255 - 48 -2, 166 + 42].into())),
        Shape::from(Triangle::new([255 - 48 -37, 166 + 74].into(), [255 - 48 -32, 166 + 72].into(), [255 - 48 -52, 166 + 67].into())),
        Shape::from(Triangle::new([255 - 48 -48, 166 + 72].into(), [255 - 48 -37, 166 + 74].into(), [255 - 48 -52, 166 + 67].into())),
        Shape::from(Triangle::new([255 - 48 -37, 166 + 74].into(), [255 - 48 -48, 166 + 72].into(), [255 - 48 -44, 166 + 74].into())),
        Shape::from(Triangle::new([255 - 48 -32, 166 + 1].into(), [255 - 48 -41, 166 + 2].into(), [255 - 48 -51, 166 + 10].into())),
        Shape::from(Triangle::new([255 - 48 -7, 166 + 55].into(), [255 - 48 -4, 166 + 50].into(), [255 - 48 -10, 166 + 58].into())),
        Shape::from(Triangle::new([255 - 48 -51, 166 + 69].into(), [255 - 48 -48, 166 + 72].into(), [255 - 48 -52, 166 + 67].into())),
        Shape::from(Triangle::new([255 - 48 -41, 166 + 2].into(), [255 - 48 -49, 166 + 6].into(), [255 - 48 -51, 166 + 10].into())),
      ]
    }),
  ]
}