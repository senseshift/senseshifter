#![crate_type = "lib"]
#![crate_name = "xrconnect_proto"]

/// Re-export to ensure that users of this crate can access the same version.
pub use prost;
pub use prost_types;
pub use async_trait::async_trait;

pub mod devices {

  #[cfg(feature = "v1alpha1")]
  pub mod v1alpha1 {
    include!("codegen/xrconnect.devices.v1alpha1.rs");

    impl From<DeviceMessage> for EventStreamResponse {
      fn from(message: DeviceMessage) -> Self {
        Self {
          message: Some(message),
        }
      }
    }

    impl From<device_message::ScanStarted> for DeviceMessage {
      fn from(scan_started: device_message::ScanStarted) -> Self {
        Self {
          r#type: Some(device_message::Type::ScanStarted(scan_started)),
        }
      }
    }

    impl From<device_message::ScanStopped> for DeviceMessage {
      fn from(scan_stopped: device_message::ScanStopped) -> Self {
        Self {
          r#type: Some(device_message::Type::ScanStopped(scan_stopped)),
        }
      }
    }
  }
}
