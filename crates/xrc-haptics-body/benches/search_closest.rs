use criterion::{criterion_group, criterion_main, Criterion};
use xrc_geometry::Point2;
use xrc_geometry_test_fixtures::hardlight_vest_chest_front;

use xrc_haptics_body::HapticPlaneU8;

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("search closest point", |b| {
    let mut plane = HapticPlaneU8::<()>::default();
    hardlight_vest_chest_front().iter().for_each(|geometry| {
      plane.insert(geometry.clone(), ());
    });

    let point = Point2::new(145, 96);
    let center_point = Point2::new(128, 88);

    b.iter(|| assert_eq!(plane.search_closest(&point), center_point));
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);