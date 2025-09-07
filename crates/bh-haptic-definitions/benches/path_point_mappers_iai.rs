use bh_haptic_definitions::path_point_mapper::{
  InterpolatingMapper, InterpolatingMapperWithCoordinates, Mapper, spread_points_evenly,
};
use iai::{black_box, main};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
  static ref INTERPOLATING_MAPPER: InterpolatingMapper = InterpolatingMapper {
    max_points: 3,
    x_points: 4,
    y_points: 4,
  };
  static ref INTERPOLATING_MAPPER_WITH_COORDINATES: InterpolatingMapperWithCoordinates =
    InterpolatingMapperWithCoordinates {
      max_points: 3,
      grid_points: spread_points_evenly(4, 4),
    };
}

fn map_interpolating() -> HashMap<usize, f64> {
  INTERPOLATING_MAPPER.map(black_box(0.0), black_box(0.0))
}

fn map_interpolating_with_coordinates() -> HashMap<usize, f64> {
  INTERPOLATING_MAPPER_WITH_COORDINATES.map(black_box(0.0), black_box(0.0))
}

main!(map_interpolating, map_interpolating_with_coordinates);
