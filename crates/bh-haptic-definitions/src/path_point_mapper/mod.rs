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
}
