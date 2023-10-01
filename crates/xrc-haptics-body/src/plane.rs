use dashmap::DashMap;
use xrc_geometry::Point;
use crate::{ActuatorSender, ActuatorGeometry};

pub struct HapticPlaneU8 {
  actuators: DashMap<ActuatorGeometry<u8>, ActuatorSender>,
  centers: DashMap<Point<u8, 2>, ActuatorGeometry<u8>>,
  intensities: DashMap<ActuatorGeometry<u8>, u8>,

  state: [u8; 256 * 256],
}

impl Default for HapticPlaneU8 {
  fn default() -> Self {
    Self {
      actuators: DashMap::new(),
      centers: DashMap::new(),
      intensities: DashMap::new(),
      
      state: [0; 256 * 256],
    }
  }
}

impl HapticPlaneU8 {
  pub fn insert(&self, geometry: ActuatorGeometry<u8>, sender: ActuatorSender) {
    self.actuators.insert(geometry.clone(), sender.clone());
    self.centers.insert(geometry.center(), geometry.clone());
    self.intensities.insert(geometry, 0);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn test_plane_u8_insert()
  {
    use tokio::sync::mpsc;
    use xrc_geometry::Circle;

    let plane = HapticPlaneU8::default();

    let (sender, _) = mpsc::channel(1);
    plane.insert(Circle::new(Point::from([5, 83]), 10).into(), sender);

    assert_eq!(plane.actuators.len(), 1);
    assert_eq!(plane.centers.len(), 1);
    assert_eq!(plane.intensities.len(), 1);

    let center = Point::from([5, 83]);

    assert!(plane.centers.contains_key(&center));
  }
}