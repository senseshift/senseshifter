use std::collections::HashMap;

pub trait Mapper {
  fn map(&self, x: f32, y: f32) -> HashMap<usize, f64>;
}

/// Interpolates 0.0..1.0 to the maximum of `max_points` points or less on the X*Y even unpadded grid.
///
/// ### Generated index positions
///
/// ```text
/// +-------------------+
/// |0     1     2     3|
/// |                   |
/// |4     5     6     7|
/// |                   |
/// |8     9    10    11|
/// |                   |
/// |12   13    14    15|
/// +-------------------+
///
/// ### Behaviour
///
/// - If the mapped point is exactly on a grid point, return that point.
/// - If the mapped point is between two grid points on the X axis, return two points with intensity, interpolated by distance.
/// - If the mapped point is between two grid points on the Y axis, return two points with intensity, interpolated by distance.
/// - Otherwise, return N closest points with intensity, interpolated by distance.
/// ```
pub struct InterpolatingMapper {
  pub max_points: usize,

  pub x_points: usize,
  pub y_points: usize,
}

impl InterpolatingMapper {
  pub fn new(max_points: usize, x_points: usize, y_points: usize) -> Self {
    Self {
      max_points,
      x_points,
      y_points,
    }
  }
}

impl Mapper for InterpolatingMapper {
  /// Selects max N points on the grid.
  fn map(&self, x: f32, y: f32) -> HashMap<usize, f64> {
    let mut points = HashMap::new();

    let x = x.clamp(0.0, 1.0);
    let y = y.clamp(0.0, 1.0);

    // Convert from [0,1] to grid coordinates
    let grid_x = x * (self.x_points - 1) as f32;
    let grid_y = y * (self.y_points - 1) as f32;

    // Find closest grid coordinates
    let closest_x = grid_x.round() as usize;
    let closest_y = grid_y.round() as usize;

    const EPSILON: f32 = 1e-6;

    // Check if we're exactly on a grid point
    if (grid_x - closest_x as f32).abs() < EPSILON && (grid_y - closest_y as f32).abs() < EPSILON {
      let index = closest_y * self.x_points + closest_x;
      points.insert(index, 1.0);
      return points;
    }

    // Check if we're on a horizontal line (y is exact)
    if (grid_y - closest_y as f32).abs() < EPSILON {
      // Interpolate only in X direction - select 2 closest points on this Y line
      let x_left = grid_x.floor() as usize;
      let x_right = (x_left + 1).min(self.x_points - 1);

      if x_left == x_right {
        // At edge
        points.insert(closest_y * self.x_points + x_left, 1.0);
      } else {
        // Between two points
        let fx = grid_x - x_left as f32;
        points.insert(closest_y * self.x_points + x_left, (1.0 - fx) as f64);
        points.insert(closest_y * self.x_points + x_right, fx as f64);
      }
      return points;
    }

    // Check if we're on a vertical line (x is exact)
    if (grid_x - closest_x as f32).abs() < EPSILON {
      // Interpolate only in Y direction - select 2 closest points on this X line
      let y_top = grid_y.floor() as usize;
      let y_bottom = (y_top + 1).min(self.y_points - 1);

      if y_top == y_bottom {
        // At edge
        points.insert(y_top * self.x_points + closest_x, 1.0);
      } else {
        // Between two points
        let fy = grid_y - y_top as f32;
        points.insert(y_top * self.x_points + closest_x, (1.0 - fy) as f64);
        points.insert(y_bottom * self.x_points + closest_x, fy as f64);
      }
      return points;
    }

    // Interior point - find closest points up to max_points and interpolate
    let mut distances: Vec<(usize, f32)> = Vec::new();

    for gy in 0..self.y_points {
      for gx in 0..self.x_points {
        let index = gy * self.x_points + gx;
        let dx = grid_x - gx as f32;
        let dy = grid_y - gy as f32;
        let distance = (dx * dx + dy * dy).sqrt();
        distances.push((index, distance));
      }
    }

    // Sort by distance and take max_points closest
    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let closest = &distances[..self.max_points.min(distances.len())];

    // Calculate weights using inverse distance
    let mut total_weight = 0.0;
    let mut weights: Vec<(usize, f64)> = Vec::new();

    for &(index, distance) in closest {
      let weight = 1.0 / (distance as f64);
      weights.push((index, weight));
      total_weight += weight;
    }

    // Normalize weights
    for (index, weight) in weights {
      points.insert(index, weight / total_weight);
    }

    points
  }
}

pub struct InterpolatingMapperWithCoordinates {
  pub max_points: usize,

  /// All grid points in (x, y) format, 0.0..1.0
  pub grid_points: Vec<(f32, f32)>,
}

impl InterpolatingMapperWithCoordinates {
  pub fn new(max_points: usize, grid_points: Vec<(f32, f32)>) -> Self {
    Self {
      max_points,
      grid_points,
    }
  }
}

impl Mapper for InterpolatingMapperWithCoordinates {
  fn map(&self, x: f32, y: f32) -> HashMap<usize, f64> {
    let mut points = HashMap::new();

    let x = x.clamp(0.0, 1.0);
    let y = y.clamp(0.0, 1.0);

    // Calculate distances to all grid points
    let mut distances: Vec<(usize, f32)> = Vec::new();

    for (index, &(gx, gy)) in self.grid_points.iter().enumerate() {
      let dx = x - gx;
      let dy = y - gy;
      let distance = (dx * dx + dy * dy).sqrt();
      distances.push((index, distance));
    }

    // Sort by distance (closest first) and take max_points
    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let closest = &distances[..self.max_points.min(distances.len())];

    // If we're exactly on a grid point
    if closest[0].1 < 1e-6 {
      points.insert(closest[0].0, 1.0);
      return points;
    }

    // Check if we have exactly 2 points on the same axis
    if closest.len() >= 2 {
      let (idx1, _) = closest[0];
      let (idx2, _) = closest[1];
      let (x1, y1) = self.grid_points[idx1];
      let (x2, y2) = self.grid_points[idx2];

      const EPSILON: f32 = 1e-6;

      // Check if points are on same horizontal line and input point is between them
      if (y1 - y2).abs() < EPSILON
        && y >= y1.min(y2) - EPSILON
        && y <= y1.max(y2) + EPSILON
        && (y - y1).abs() < EPSILON
      {
        // On the same horizontal line, interpolate in X
        let total_distance = (x2 - x1).abs();
        if total_distance > EPSILON {
          let weight1 = ((x2 - x).abs() / total_distance) as f64;
          let weight2 = ((x - x1).abs() / total_distance) as f64;
          points.insert(idx1, weight1);
          points.insert(idx2, weight2);
          return points;
        }
      }

      // Check if points are on same vertical line and input point is between them
      if (x1 - x2).abs() < EPSILON
        && x >= x1.min(x2) - EPSILON
        && x <= x1.max(x2) + EPSILON
        && (x - x1).abs() < EPSILON
      {
        // On the same vertical line, interpolate in Y
        let total_distance = (y2 - y1).abs();
        if total_distance > EPSILON {
          let weight1 = ((y2 - y).abs() / total_distance) as f64;
          let weight2 = ((y - y1).abs() / total_distance) as f64;
          points.insert(idx1, weight1);
          points.insert(idx2, weight2);
          return points;
        }
      }
    }

    // General case: use inverse distance weighting for closest points
    let mut total_weight = 0.0;
    let mut weights: Vec<(usize, f64)> = Vec::new();

    for &(index, distance) in closest {
      let weight = 1.0 / (distance as f64);
      weights.push((index, weight));
      total_weight += weight;
    }

    // Normalize weights
    for (index, weight) in weights {
      points.insert(index, weight / total_weight);
    }

    points
  }
}

pub fn spread_points_evenly(x: usize, y: usize) -> Vec<(f32, f32)> {
  let mut points = Vec::new();

  for j in 0..y {
    for i in 0..x {
      let _index = j * x + i;

      let px = if x == 1 {
        0.5
      } else {
        i as f32 / (x - 1) as f32
      };

      let py = if y == 1 {
        0.5
      } else {
        j as f32 / (y - 1) as f32
      };

      points.push((px, py));
    }
  }

  points
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_map_interpolating() {
    let mapper = InterpolatingMapper {
      max_points: 3,
      x_points: 4,
      y_points: 4,
    }; // e.g. a vest with 4x4 grid per face (Pro, etc.)

    let expected = HashMap::from([(0, 1.0)]); // top-left corner
    assert_eq!(mapper.map(0.0, 0.0), expected);

    let expected = HashMap::from([(12, 1.0)]); // bottom-right corner
    assert_eq!(mapper.map(0.0, 1.0), expected);

    let expected = HashMap::from([(3, 1.0)]); // top-right corner
    assert_eq!(mapper.map(1.0, 0.0), expected);

    let expected = HashMap::from([(15, 1.0)]); // bottom-right corner
    assert_eq!(mapper.map(1.0, 1.0), expected);

    let expected = HashMap::from([(0, 0.625), (1, 0.375)]); // between grid points 0 and 1, X axis
    assert_eq!(mapper.map(0.125, 0.0), expected);

    let expected = HashMap::from([(0, 0.625), (4, 0.375)]); // between grid points 0 and 4, Y axis
    assert_eq!(mapper.map(0.0, 0.125), expected);

    let result = mapper.map(0.9, 0.9);
    // For (0.9, 0.9) which maps to grid position (2.7, 2.7), we expect 3 closest points
    // The actual weights will depend on inverse distance weighting
    assert_eq!(result.len(), 3);

    // The sum of all weights should be 1.0
    let total: f64 = result.values().sum();
    assert!((total - 1.0).abs() < 1e-10);
  }

  #[test]
  fn test_map_interpolating_with_coordinates() {
    let mapper = InterpolatingMapperWithCoordinates {
      max_points: 3,
      grid_points: spread_points_evenly(4, 4),
    };

    let expected = HashMap::from([(0, 1.0)]); // top-left corner
    assert_eq!(mapper.map(0.0, 0.0), expected);

    let expected = HashMap::from([(12, 1.0)]); // bottom-right corner
    assert_eq!(mapper.map(0.0, 1.0), expected);

    let expected = HashMap::from([(3, 1.0)]); // top-right corner
    assert_eq!(mapper.map(1.0, 0.0), expected);

    let expected = HashMap::from([(15, 1.0)]); // bottom-right corner
    assert_eq!(mapper.map(1.0, 1.0), expected);

    let expected = HashMap::from([(0, 0.625), (1, 0.375)]); // between grid points 0 and 1, X axis
    assert_eq!(mapper.map(0.125, 0.0), expected);

    let expected = HashMap::from([(0, 0.625), (4, 0.375)]); // between grid points 0 and 4, Y axis
    assert_eq!(mapper.map(0.0, 0.125), expected);

    let result = mapper.map(0.9, 0.9);
    // For (0.9, 0.9) which maps to grid position (2.7, 2.7), we expect 3 closest points
    // The actual weights will depend on inverse distance weighting
    assert_eq!(result.len(), 3);

    // The sum of all weights should be 1.0
    let total: f64 = result.values().sum();
    assert!((total - 1.0).abs() < 1e-10);
  }

  #[test]
  fn test_spread_points_evenly() {
    assert_eq!(spread_points_evenly(1, 1), vec![(0.5, 0.5)]);

    assert_eq!(
      spread_points_evenly(2, 2),
      vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)]
    );

    assert_eq!(
      spread_points_evenly(4, 4),
      vec![
        // row 0
        (0.0, 0.0),
        (0.33333334, 0.0),
        (0.6666667, 0.0),
        (1.0, 0.0),
        // row 1
        (0.0, 0.33333334),
        (0.33333334, 0.33333334),
        (0.6666667, 0.33333334),
        (1.0, 0.33333334),
        // row 2
        (0.0, 0.6666667),
        (0.33333334, 0.6666667),
        (0.6666667, 0.6666667),
        (1.0, 0.6666667),
        // row 3
        (0.0, 1.0),
        (0.33333334, 1.0),
        (0.6666667, 1.0),
        (1.0, 1.0),
      ]
    )
  }
}
