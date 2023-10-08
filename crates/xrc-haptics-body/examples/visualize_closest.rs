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

  // // Actual bHaptics X40
  // for x in [44, 99, 157, 212].iter() {
  //   for y in [85, 118, 151, 184, 217].iter() {
  //     let center = Point2::new(*x, *y);
  //
  //     let geometry = Circle::new(center, 10);
  //     plane.insert(Shape::from(geometry), ());
  //   }
  // }

  // // Hardlight VR Suit
  // {
  //   hardlight_vest_chest_front().iter().for_each(|geometry| {
  //     plane.insert_no_recalc(geometry.clone(), ());
  //   });
  // }

  plane.insert(Shape::from(Circle::new([10 , 10 ].into(), 10)), ());
  plane.insert(Shape::from(Circle::new([10 , 245].into(), 10)), ());
  plane.insert(Shape::from(Circle::new([245, 10 ].into(), 10)), ());
  plane.insert(Shape::from(Circle::new([245, 245].into(), 10)), ());

  plane.insert(Shape::from(Circle::new([100, 100].into(), 10)), ());
  plane.insert(Shape::from(Circle::new([100, 155].into(), 10)), ());
  plane.insert(Shape::from(Circle::new([155, 100].into(), 10)), ());
  plane.insert(Shape::from(Circle::new([155, 155].into(), 10)), ());

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

  let mut data = [0; 256 * 256 * 3];

  let now = std::time::Instant::now();
  for (x, row) in plane.state().0.iter().enumerate() {
    for (y, _) in row.iter().enumerate() {
      let index = (y * 256 + x) * 3;
      let point = Point2::new(x as u8, y as u8);

      // let circle = Circle::new(point, 10);
      // let closests: Vec<_> = circle.points_inside().iter()
      //   .map(|point| plane.get_closest(&point))
      //   .collect();
      //
      // // calc average
      // let circle_closest: (u32, u32) = closests.iter()
      //   .fold((0, 0), |acc, point| {
      //     (acc.0 + point.x as u32, acc.1 + point.y as u32)
      //   });
      //
      // let (r, g) = (
      //   (circle_closest.0 / closests.len() as u32) as u8,
      //   (circle_closest.1 / closests.len() as u32) as u8,
      // );
      //
      // data[index + 0] = r;
      // data[index + 1] = g;

      let closest = plane.get_closest(&point);
      data[index + 0] = closest.x;
      data[index + 1] = closest.y;
    }
  }
  let elapsed = now.elapsed();
  println!("total mapping elapsed: {:?} (~{:?} per point)", elapsed, elapsed / (256 * 256));

  let now = std::time::Instant::now();
  for entry in plane.actuators() {
    for point in entry.key().points_inside() {
      let index = (point.y as usize * 256 + point.x as usize) * 3;
      data[index + 2] = 255;
    }

    let center = entry.key().center();
    let index = (center.y as usize * 256 + center.x as usize) * 3;
    data[index + 0] = 255;
    data[index + 1] = 255;
    data[index + 2] = 255;

    // // outline bbox
    // {
    //   let bbox = entry.key().bbox();
    //
    //   // Top line
    //   for x in bbox.min().x..bbox.max().x {
    //     let index = (bbox.min().y as usize * 256 + x as usize) * 3;
    //     data[index + 0] = 255;
    //     data[index + 1] = 255;
    //     data[index + 2] = 255;
    //   }
    //   // Bottom line
    //   for x in bbox.min().x..bbox.max().x {
    //     let index = (bbox.max().y as usize * 256 + x as usize) * 3;
    //     data[index + 0] = 255;
    //     data[index + 1] = 255;
    //     data[index + 2] = 255;
    //   }
    //   // Left line
    //   for y in bbox.min().y..bbox.max().y {
    //     let index = (y as usize * 256 + bbox.min().x as usize) * 3;
    //     data[index + 0] = 255;
    //     data[index + 1] = 255;
    //     data[index + 2] = 255;
    //   }
    //   // Right line
    //   for y in bbox.min().y..bbox.max().y {
    //     let index = (y as usize * 256 + bbox.max().x as usize) * 3;
    //     data[index + 0] = 255;
    //     data[index + 1] = 255;
    //     data[index + 2] = 255;
    //   }
    //
    //   // let center = bbox.center();
    //   // let index = (center.y as usize * 256 + center.x as usize) * 3;
    //   // data[index + 2] = 255;
    // }
  }

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