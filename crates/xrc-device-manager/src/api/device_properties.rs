use derivative::Derivative;

pub trait DeviceProperties {
  fn manufacturer(&self) -> &String;

  fn product(&self) -> &String;

  fn serial_number(&self) -> &String;

  fn hardware_version(&self) -> &String;

  fn firmware_version(&self) -> &String;
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative, Debug, Clone)]
pub struct GenericDeviceProperties {
  manufacturer: String,
  product: String,
  serial_number: String,
  hardware_version: String,
  firmware_version: String,
}

impl GenericDeviceProperties {
  #[inline(always)]
  pub fn new(
    manufacturer: String,
    product: String,
    serial_number: String,
    hardware_version: String,
    firmware_version: String,
  ) -> Self {
    Self {
      manufacturer,
      product,
      serial_number,
      hardware_version,
      firmware_version,
    }
  }
}

impl DeviceProperties for GenericDeviceProperties {
  #[inline(always)]
  fn manufacturer(&self) -> &String {
    &self.manufacturer
  }

  #[inline(always)]
  fn product(&self) -> &String {
    &self.product
  }

  #[inline(always)]
  fn serial_number(&self) -> &String {
    &self.serial_number
  }

  #[inline(always)]
  fn hardware_version(&self) -> &String {
    &self.hardware_version
  }

  #[inline(always)]
  fn firmware_version(&self) -> &String {
    &self.firmware_version
  }
}
