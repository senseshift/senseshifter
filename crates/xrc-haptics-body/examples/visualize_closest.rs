use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use num::traits::NumOps;
use xrc_geometry::*;
use xrc_geometry_test_fixtures::{hardlight_vest_chest_front, owo_skin_front};
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
  //     plane.insert(Shape::from(geometry), ());
  //   }
  // }

  // Hardlight VR Suit
  {
    hardlight_vest_chest_front().iter().for_each(|geometry| {
      plane.insert_no_recalc(geometry.clone(), ());
    });
  }

  // // OWO Skin
  // {
  //   owo_skin_front().iter().for_each(|geometry| {
  //     plane.insert(geometry.clone(), ());
  //   });
  // }

  let now = std::time::Instant::now();
  plane.recalc_closest();
  let duration = now.elapsed();
  println!("recalc elapsed: {:?} (~{:?} per point)", duration, duration / (256 * 256));

  let mut data = vec![0; 256 * 256 * 3];

  let now = std::time::Instant::now();
  for (x, row) in plane.state().0.iter().enumerate() {
    for (y, _) in row.iter().enumerate() {
      let index = (y * 256 + x) * 3;
      let point = Point2::new(x as u8, y as u8);
      let closest = plane.get_closest(&point);

      data[index + 0] = closest.x;
      data[index + 1] = closest.y;
    }
  }
  let elapsed = now.elapsed();
  println!("total mapping elapsed: {:?} (~{:?} per point)", elapsed, elapsed / (256 * 256));

  let now = std::time::Instant::now();
  for entry in plane.actuators() {
    let geometry = entry.key();
    let center = geometry.center();

    for point in geometry.points_inside() {
      let index = (point.y as usize * 256 + point.x as usize) * 3;
      data[index + 2] = 255;
    }

    let index = (center.y as usize * 256 + center.x as usize) * 3;
    data[index + 0] = 255;
    data[index + 1] = 255;
    data[index + 2] = 255;
  }
  // for (x, row) in plane.state().0.iter().enumerate() {
  //   for (y, _) in row.iter().enumerate() {
  //     let index = (y * 256 + x) * 3;
  //     let point = Point2::new(x as u8, y as u8);
  //
  //     for entry in plane.actuators() {
  //       let geometry = entry.key();
  //       let center = geometry.center();
  //
  //       if center == point {
  //         data[index + 0] = 255;
  //         data[index + 1] = 255;
  //         data[index + 2] = 255;
  //       } else if geometry.within(&point) == true {
  //         data[index + 2] = (data[index + 2] as u16 + 255) as u8;
  //       }
  //     }
  //   }
  // }
  let elapsed = now.elapsed();
  println!("mapping shapes withins elapsed: {:?}", elapsed);

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