use bh_haptic_definitions::path_point_mapper::{
  InterpolatingMapperEvenGrid, InterpolatingMapperWithCoordinates, Mapper, spread_points_evenly,
};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn bench_mappers(c: &mut Criterion) {
  let mut group = c.benchmark_group("PathPointMapper");

  let interpolating_mapper = InterpolatingMapperEvenGrid::new(4, 4);
  let interpolating_mapper_with_coordinates =
    InterpolatingMapperWithCoordinates::new(spread_points_evenly(4, 4));

  for x in 0..2 {
    for y in 0..5 {
      let x = x as f32 / 2.0;
      let y = y as f32 / 5.0;

      group.bench_with_input(
        BenchmarkId::new("InterpolatingMapper", format!("{x:.3}x{y:.3}")),
        &(x, y),
        |b, &(x, y)| {
          b.iter(|| interpolating_mapper.map(x, y, 3));
        },
      );

      group.bench_with_input(
        BenchmarkId::new(
          "InterpolatingMapperWithCoordinates",
          format!("{x:.3}x{y:.3}"),
        ),
        &(x, y),
        |b, &(x, y)| {
          b.iter(|| interpolating_mapper_with_coordinates.map(x, y, 3));
        },
      );
    }
  }

  group.finish();
}

criterion_group!(benches, bench_mappers);
criterion_main!(benches);
