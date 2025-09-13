mod mapper;

use crate::{DevicePosition, DotPoint, PathPoint};
pub use mapper::*;

use std::collections::HashMap;
use std::sync::Arc;

pub struct PositionMapper {
  device_position_mappers: HashMap<DevicePosition, Arc<Box<dyn Mapper>>>,
}

impl PositionMapper {
  pub fn new(device_position_mapper: HashMap<DevicePosition, Arc<Box<dyn Mapper>>>) -> Self {
    Self {
      device_position_mappers: device_position_mapper,
    }
  }
}

impl Default for PositionMapper {
  fn default() -> Self {
    let vest_mapper: Arc<Box<dyn Mapper>> =
      Arc::new(Box::new(InterpolatingMapperEvenGrid::new(4, 4)));

    PositionMapper::new(HashMap::from([
      (DevicePosition::Vest, vest_mapper.clone()),
      (DevicePosition::VestFront, vest_mapper.clone()),
      (DevicePosition::VestBack, vest_mapper.clone()),
    ]))
  }
}

impl PositionMapper {
  pub fn map_point(
    &self,
    device: &DevicePosition,
    point: &PathPoint,
  ) -> anyhow::Result<Vec<DotPoint>> {
    let mapper = self
      .device_position_mappers
      .get(device)
      .ok_or_else(|| anyhow::anyhow!("No mapper found for device position: {:?}", device))?;

    let points = mapper.map(*point.x() as f32, *point.y() as f32, *point.motor_count());

    let result = points
      .into_iter()
      .map(|(index, intensity)| {
        let intensity = *point.intensity() as f64 * intensity;

        DotPoint::new(index, (intensity as u32).min(DotPoint::MAX_INTENSITY))
      })
      .collect();

    Ok(result)
  }

  pub fn map_points(
    &self,
    device: &DevicePosition,
    points: &Vec<PathPoint>,
  ) -> anyhow::Result<Vec<DotPoint>> {
    let mapper = self
      .device_position_mappers
      .get(device)
      .ok_or_else(|| anyhow::anyhow!("No mapper found for device position: {:?}", device))?;

    let mut mapped_points: Vec<DotPoint> = Vec::new();

    for point in points {
      let local_mapped_points =
        mapper.map(*point.x() as f32, *point.y() as f32, *point.motor_count());

      for (index, intensity) in local_mapped_points {
        let existing_point = mapped_points.iter_mut().find(|p| p.index() == &index);

        if existing_point.is_none() {
          let intensity = *point.intensity() as f64 * intensity;
          mapped_points.push(DotPoint::new(index, intensity as u32));

          continue;
        }

        let existing_point = existing_point.unwrap();

        *existing_point.intensity_mut() =
          (*existing_point.intensity() + intensity as u32).min(DotPoint::MAX_INTENSITY);
      }
    }

    Ok(mapped_points)
  }
}
