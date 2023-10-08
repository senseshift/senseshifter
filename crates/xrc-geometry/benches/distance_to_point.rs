use criterion::{criterion_group, criterion_main, Criterion};

use xrc_geometry::{Distance, Point2, Ellipse};

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("point outside ellipse, but within bbox", |b| {
    let ellipse = Ellipse::new([128, 88].into(), (18, 10));
    let point = Point2::new(145, 96);

    b.iter(|| assert_eq!(ellipse.distance(&point).round(), 3));
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);