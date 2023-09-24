use std::fmt::Debug;
use async_trait::async_trait;
use xrconnect_proto::devices::v1alpha1::device::{Info, Status};

#[async_trait]
pub trait Device: Send + Sync + Debug {
  fn id(&self) -> &String;

  fn name(&self) -> Option<&String>;

  fn connected(&self) -> bool {
    false
  }

  fn connectible(&self) -> bool {
    false
  }

  async fn update(&mut self) {
    // Do nothing by default
  }
}

#[cfg(feature = "proto-v1alpha1")]
impl Into<xrconnect_proto::devices::v1alpha1::Device> for &dyn Device {
  fn into(self) -> xrconnect_proto::devices::v1alpha1::Device {
    use xrconnect_proto::devices::v1alpha1::{
      Device as ProtoDevice,
      device::{
        Status,
      },
    };

    ProtoDevice {
      device_id: self.id().to_string(),
      name: self.name().cloned(),
      status: match self.connected() {
        true => Status::Connected.into(),
        false => Status::Disconnected.into(),
      },
      connectible: self.connectible(),
      info: None,
      properties: None,
    }
  }
}
