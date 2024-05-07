mod device_config;

#[cfg(feature = "btleplug")]
pub mod btleplug;

pub use device_config::*;

pub use xrc_commons::Result;
