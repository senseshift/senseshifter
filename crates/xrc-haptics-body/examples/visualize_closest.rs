use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use num::traits::NumOps;
use xrc_geometry::*;
use xrc_haptics_body::*;

fn accurate_map<T>(x: T, in_min: T, in_max: T, out_min: T, out_max: T) -> T
  where
    T: NumOps + Copy + From<f64>,
    f64: From<T>,
{
  let run = in_max - in_min;

  let rise = out_max - out_min;
  let delta = x - in_min;

  return ((f64::from(delta) * f64::from(rise)) / f64::from(run) + f64::from(out_min)).into()
}

pub fn main() {
  let mut plane = HapticPlaneU8::<()>::default();

  // Actual bHaptics X40
  // for x in [44, 99, 157, 212].iter() {
  //   for y in [85, 118, 151, 184, 217].iter() {
  //     let center = Point2::new(*x, *y);
  //
  //     let geometry = Circle::new(center, 10);
  //     plane.insert(ActuatorGeometry::from(geometry), ());
  //   }
  // }

  // Hardlight VR Suit
  {
    // Center Piece
    plane.insert(ActuatorGeometry::from(Ellipse::new([128, 88].into(), (18, 10))), ());

    // Top Left
    plane.insert(
      ActuatorGeometry::Collection(GeometryCollection {
        geometry: vec![
          ActuatorGeometry::from(Triangle::new([29, 79].into(), [35, 44].into(), [86, 41].into())),
          ActuatorGeometry::from(Triangle::new([29, 79].into(), [86, 41].into(), [97, 68].into())),
          ActuatorGeometry::from(Triangle::new([29, 79].into(), [97, 68].into(), [85, 73].into())),
          ActuatorGeometry::from(Triangle::new([29, 79].into(), [85, 73].into(), [79, 78].into())),
          ActuatorGeometry::from(Triangle::new([29, 79].into(), [79, 78].into(), [76, 83].into())),
          ActuatorGeometry::from(Triangle::new([29, 79].into(), [76, 83].into(), [75, 89].into())),
          ActuatorGeometry::from(Triangle::new([29, 79].into(), [75, 89].into(), [39, 89].into())),
        ]
      }),
      (),
    );
    plane.insert(
      ActuatorGeometry::Collection(GeometryCollection {
        geometry: vec![
          ActuatorGeometry::from(Triangle::new([45, 107].into(), [84, 107].into(), [44, 113].into())),
          ActuatorGeometry::from(Triangle::new([44, 113].into(), [84, 107].into(), [83, 132].into())),
          ActuatorGeometry::from(Triangle::new([84, 107].into(), [93, 112].into(), [83, 132].into())),
          ActuatorGeometry::from(Triangle::new([93, 112].into(), [100, 114].into(), [83, 132].into())),
          ActuatorGeometry::from(Triangle::new([83, 132].into(), [100, 114].into(), [100, 132].into())),
        ]
      }),
      ()
    );
    plane.insert(
      ActuatorGeometry::Collection(GeometryCollection {
        geometry: vec![
          ActuatorGeometry::from(Triangle::new([41, 134].into(), [76, 151].into(), [45, 143].into())),
          ActuatorGeometry::from(Triangle::new([45, 143].into(), [76, 151].into(), [85, 169].into())),
          ActuatorGeometry::from(Triangle::new([76, 151].into(), [85, 169].into(), [99, 151].into())),
          ActuatorGeometry::from(Triangle::new([85, 169].into(), [99, 151].into(), [99, 169].into())),
        ]
      }),
      ()
    );
    plane.insert(
      ActuatorGeometry::Collection(GeometryCollection {
        geometry: vec![
          ActuatorGeometry::from(Triangle::new([47, 168].into(), [75, 184].into(), [52, 179].into())),
          ActuatorGeometry::from(Triangle::new([52, 179].into(), [75, 184].into(), [81, 202].into())),
          ActuatorGeometry::from(Triangle::new([75, 184].into(), [99, 186].into(), [81, 202].into())),
          ActuatorGeometry::from(Triangle::new([81, 202].into(), [99, 186].into(), [99, 206].into())),
        ]
      }),
      ()
    );

    // Top Right
    plane.insert(
      ActuatorGeometry::Collection(GeometryCollection {
        geometry: vec![
          ActuatorGeometry::from(Triangle::new([255 - 29, 79].into(), [255 - 35, 44].into(), [255 - 86, 41].into())),
          ActuatorGeometry::from(Triangle::new([255 - 29, 79].into(), [255 - 86, 41].into(), [255 - 97, 68].into())),
          ActuatorGeometry::from(Triangle::new([255 - 29, 79].into(), [255 - 97, 68].into(), [255 - 85, 73].into())),
          ActuatorGeometry::from(Triangle::new([255 - 29, 79].into(), [255 - 85, 73].into(), [255 - 79, 78].into())),
          ActuatorGeometry::from(Triangle::new([255 - 29, 79].into(), [255 - 79, 78].into(), [255 - 76, 83].into())),
          ActuatorGeometry::from(Triangle::new([255 - 29, 79].into(), [255 - 76, 83].into(), [255 - 75, 89].into())),
          ActuatorGeometry::from(Triangle::new([255 - 29, 79].into(), [255 - 75, 89].into(), [255 - 39, 89].into())),
        ]
      }),
      (),
    );
    plane.insert(
      ActuatorGeometry::Collection(GeometryCollection {
        geometry: vec![
          ActuatorGeometry::from(Triangle::new([255 - 45, 107].into(), [255 - 84, 107].into(), [255 - 44, 113].into())),
          ActuatorGeometry::from(Triangle::new([255 - 44, 113].into(), [255 - 84, 107].into(), [255 - 83, 132].into())),
          ActuatorGeometry::from(Triangle::new([255 - 84, 107].into(), [255 - 93, 112].into(), [255 - 83, 132].into())),
          ActuatorGeometry::from(Triangle::new([255 - 93, 112].into(), [255 - 100, 114].into(), [255 - 83, 132].into())),
          ActuatorGeometry::from(Triangle::new([255 - 83, 132].into(), [255 - 100, 114].into(), [255 - 100, 132].into())),
        ]
      }),
      ()
    );
    plane.insert(
      ActuatorGeometry::Collection(GeometryCollection {
        geometry: vec![
          ActuatorGeometry::from(Triangle::new([255 - 41, 134].into(), [255 - 76, 151].into(), [255 - 45, 143].into())),
          ActuatorGeometry::from(Triangle::new([255 - 45, 143].into(), [255 - 76, 151].into(), [255 - 85, 169].into())),
          ActuatorGeometry::from(Triangle::new([255 - 76, 151].into(), [255 - 85, 169].into(), [255 - 99, 151].into())),
          ActuatorGeometry::from(Triangle::new([255 - 85, 169].into(), [255 - 99, 151].into(), [255 - 99, 169].into())),
        ]
      }),
      ()
    );
    plane.insert(
      ActuatorGeometry::Collection(GeometryCollection {
        geometry: vec![
          ActuatorGeometry::from(Triangle::new([255 - 47, 168].into(), [255 - 75, 184].into(), [255 - 52, 179].into())),
          ActuatorGeometry::from(Triangle::new([255 - 52, 179].into(), [255 - 75, 184].into(), [255 - 81, 202].into())),
          ActuatorGeometry::from(Triangle::new([255 - 75, 184].into(), [255 - 99, 186].into(), [255 - 81, 202].into())),
          ActuatorGeometry::from(Triangle::new([255 - 81, 202].into(), [255 - 99, 186].into(), [255 - 99, 206].into())),
        ]
      }),
      ()
    );
  }

  let mut data = vec![0; 256 * 256 * 3];

  let now = std::time::Instant::now();

  for (x, row) in plane.state().0.iter().enumerate() {
    for (y, _) in row.iter().enumerate() {
      let index = (y * 256 + x) * 3;
      let point = Point2::new(x as u8, y as u8);
      let closest = plane.search_closest(&point);

      data[index + 0] = closest.x;
      data[index + 1] = closest.y;

      for entry in plane.actuators() {
        let geometry = entry.key();
        let center = geometry.center();

        if center == point {
          data[index + 0] = 255;
          data[index + 1] = 255;
          data[index + 2] = 255;
        } else if geometry.within(&point) == PointWithin::Inside {
          data[index + 2] = (data[index + 2] as u16 + 255) as u8;
        }
      }
    }
  }

  println!("elapsed: {:?}", now.elapsed());

  let path = Path::new(file!()).with_extension("png");
  println!("writing to {:?}", path);

  let file = File::create(path).unwrap();
  let w = &mut BufWriter::new(file);

  let mut encoder = png::Encoder::new(w, 256, 256);
  encoder.set_color(png::ColorType::Rgb);
  encoder.set_depth(png::BitDepth::Eight);

  let mut writer = encoder.write_header().unwrap();
  writer.write_image_data(&data).unwrap();
}