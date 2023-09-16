#![crate_type = "lib"]
#![crate_name = "xrconnect_proto"]

/// Re-export to ensure that users of this crate can access the same version.
pub use prost;
pub use async_trait::async_trait;

pub mod devices {
  pub mod v1alpha1 {
    include!("codegen/xrconnect.devices.v1alpha1.rs");

    impl From<DeviceMessage> for EventStreamResponse {
      fn from(message: DeviceMessage) -> Self {
        Self {
          message: Some(message),
        }
      }
    }
  }
}
