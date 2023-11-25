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

  // Hardlight VR Suit
  {
    hardlight_vest_chest_front().iter().for_each(|geometry| {
      plane.insert_no_recalc(geometry.clone(), ());
    });
  }

  // // AbdulC render
  // {
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([26, 25].into(), [64, 15].into(), [71, 27].into())),
  //     Shape::from(Triangle::new([26, 25].into(), [71, 27].into(), [31, 36].into())),
  //     Shape::from(Triangle::new([81, 39].into(), [71, 27].into(), [31, 36].into())),
  //     Shape::from(Triangle::new([81, 39].into(), [36, 52].into(), [31, 36].into())),
  //     Shape::from(Triangle::new([81, 39].into(), [36, 52].into(), [37, 66].into())),
  //     Shape::from(Triangle::new([81, 39].into(), [36, 52].into(), [35, 80].into())),
  //     Shape::from(Triangle::new([81, 39].into(), [120, 110].into(), [35, 80].into())),
  //     Shape::from(Triangle::new([81, 39].into(), [120, 110].into(), [119, 72].into())),
  //     Shape::from(Triangle::new([81, 39].into(), [98, 48].into(), [119, 72].into())),
  //     Shape::from(Triangle::new([121, 53].into(), [98, 48].into(), [119, 72].into())),
  //     Shape::from(Triangle::new([35, 80].into(), [30, 89].into(), [120, 110].into())),
  //     Shape::from(Triangle::new([18, 102].into(), [30, 89].into(), [120, 110].into())),
  //     Shape::from(Triangle::new([18, 102].into(), [122, 122].into(), [120, 110].into())),
  //   ])), ());
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([16, 114].into(), [120, 131].into(), [22, 139].into())),
  //     Shape::from(Triangle::new([83, 169].into(), [120, 131].into(), [22, 139].into())),
  //     Shape::from(Triangle::new([83, 169].into(), [24, 156].into(), [22, 139].into())),
  //   ])), ());
  //
  //   plane.insert_no_recalc(Shape::from(Triangle::new([27, 167].into(), [77, 175].into(), [39, 197].into())), ());
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([115, 147].into(), [119, 147].into(), [122, 161].into())),
  //     Shape::from(Triangle::new([115, 147].into(), [92, 170].into(), [122, 161].into())),
  //     Shape::from(Triangle::new([121, 175].into(), [92, 170].into(), [122, 161].into())),
  //   ])), ());
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([53, 201].into(), [89, 180].into(), [90, 205].into())),
  //     Shape::from(Triangle::new([125, 184].into(), [89, 180].into(), [90, 205].into())),
  //     Shape::from(Triangle::new([125, 184].into(), [117, 210].into(), [90, 205].into())),
  //   ])), ());
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([53, 201].into(), [89, 180].into(), [90, 205].into())),
  //     Shape::from(Triangle::new([125, 184].into(), [89, 180].into(), [90, 205].into())),
  //     Shape::from(Triangle::new([125, 184].into(), [117, 210].into(), [90, 205].into())),
  //   ])), ());
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([51, 213].into(), [51, 220].into(), [81, 216].into())),
  //     Shape::from(Triangle::new([53, 226].into(), [51, 220].into(), [81, 216].into())),
  //     Shape::from(Triangle::new([53, 226].into(), [56, 231].into(), [81, 216].into())),
  //     Shape::from(Triangle::new([61, 234].into(), [56, 231].into(), [81, 216].into())),
  //     Shape::from(Triangle::new([61, 234].into(), [73, 238].into(), [81, 216].into())),
  //     Shape::from(Triangle::new([117, 240].into(), [73, 238].into(), [81, 216].into())),
  //     Shape::from(Triangle::new([117, 240].into(), [118, 218].into(), [81, 216].into())),
  //   ])), ());
  //
  //   // Right
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([255 - 26, 25].into(), [255 - 64, 15].into(), [255 - 71, 27].into())),
  //     Shape::from(Triangle::new([255 - 26, 25].into(), [255 - 71, 27].into(), [255 - 31, 36].into())),
  //     Shape::from(Triangle::new([255 - 81, 39].into(), [255 - 71, 27].into(), [255 - 31, 36].into())),
  //     Shape::from(Triangle::new([255 - 81, 39].into(), [255 - 36, 52].into(), [255 - 31, 36].into())),
  //     Shape::from(Triangle::new([255 - 81, 39].into(), [255 - 36, 52].into(), [255 - 37, 66].into())),
  //     Shape::from(Triangle::new([255 - 81, 39].into(), [255 - 36, 52].into(), [255 - 35, 80].into())),
  //     Shape::from(Triangle::new([255 - 81, 39].into(), [255 - 120, 110].into(), [255 - 35, 80].into())),
  //     Shape::from(Triangle::new([255 - 81, 39].into(), [255 - 120, 110].into(), [255 - 119, 72].into())),
  //     Shape::from(Triangle::new([255 - 81, 39].into(), [255 - 98, 48].into(), [255 - 119, 72].into())),
  //     Shape::from(Triangle::new([255 - 121, 53].into(), [255 - 98, 48].into(), [255 - 119, 72].into())),
  //     Shape::from(Triangle::new([255 - 35, 80].into(), [255 - 30, 89].into(), [255 - 120, 110].into())),
  //     Shape::from(Triangle::new([255 - 18, 102].into(), [255 - 30, 89].into(), [255 - 120, 110].into())),
  //     Shape::from(Triangle::new([255 - 18, 102].into(), [255 - 122, 122].into(), [255 - 120, 110].into())),
  //   ])), ());
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([255 - 16, 114].into(), [255 - 120, 131].into(), [255 - 22, 139].into())),
  //     Shape::from(Triangle::new([255 - 83, 169].into(), [255 - 120, 131].into(), [255 - 22, 139].into())),
  //     Shape::from(Triangle::new([255 - 83, 169].into(), [255 - 24, 156].into(), [255 - 22, 139].into())),
  //   ])), ());
  //
  //   plane.insert_no_recalc(Shape::from(Triangle::new([255 - 27, 167].into(), [255 - 77, 175].into(), [255 - 39, 197].into())), ());
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([255 - 115, 147].into(), [255 - 119, 147].into(), [255 - 122, 161].into())),
  //     Shape::from(Triangle::new([255 - 115, 147].into(), [255 - 92, 170].into(), [255 - 122, 161].into())),
  //     Shape::from(Triangle::new([255 - 121, 175].into(), [255 - 92, 170].into(), [255 - 122, 161].into())),
  //   ])), ());
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([255 - 53, 201].into(), [255 - 89, 180].into(), [255 - 90, 205].into())),
  //     Shape::from(Triangle::new([255 - 125, 184].into(), [255 - 89, 180].into(), [255 - 90, 205].into())),
  //     Shape::from(Triangle::new([255 - 125, 184].into(), [255 - 117, 210].into(), [255 - 90, 205].into())),
  //   ])), ());
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([255 - 53, 201].into(), [255 - 89, 180].into(), [255 - 90, 205].into())),
  //     Shape::from(Triangle::new([255 - 125, 184].into(), [255 - 89, 180].into(), [255 - 90, 205].into())),
  //     Shape::from(Triangle::new([255 - 125, 184].into(), [255 - 117, 210].into(), [255 - 90, 205].into())),
  //   ])), ());
  //
  //   plane.insert_no_recalc(Shape::Collection(ShapeCollection::new(vec![
  //     Shape::from(Triangle::new([255 - 51, 213].into(), [255 - 51, 220].into(), [255 - 81, 216].into())),
  //     Shape::from(Triangle::new([255 - 53, 226].into(), [255 - 51, 220].into(), [255 - 81, 216].into())),
  //     Shape::from(Triangle::new([255 - 53, 226].into(), [255 - 56, 231].into(), [255 - 81, 216].into())),
  //     Shape::from(Triangle::new([255 - 61, 234].into(), [255 - 56, 231].into(), [255 - 81, 216].into())),
  //     Shape::from(Triangle::new([255 - 61, 234].into(), [255 - 73, 238].into(), [255 - 81, 216].into())),
  //     Shape::from(Triangle::new([255 - 117, 240].into(), [255 - 73, 238].into(), [255 - 81, 216].into())),
  //     Shape::from(Triangle::new([255 - 117, 240].into(), [255 - 118, 218].into(), [255 - 81, 216].into())),
  //   ])), ());
  // }

  // plane.insert_no_recalc(Shape::from(Circle::new([10 , 10 ].into(), 10)), ());
  // plane.insert_no_recalc(Shape::from(Circle::new([10 , 245].into(), 10)), ());
  // plane.insert_no_recalc(Shape::from(Circle::new([245, 10 ].into(), 10)), ());
  // plane.insert_no_recalc(Shape::from(Circle::new([245, 245].into(), 10)), ());

  // plane.insert_no_recalc(Shape::from(Circle::new([100, 100].into(), 10)), ());
  // plane.insert_no_recalc(Shape::from(Circle::new([100, 155].into(), 10)), ());
  // plane.insert_no_recalc(Shape::from(Circle::new([155, 100].into(), 10)), ());
  // plane.insert_no_recalc(Shape::from(Circle::new([155, 155].into(), 10)), ());

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