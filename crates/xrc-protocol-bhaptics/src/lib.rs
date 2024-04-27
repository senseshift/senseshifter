mod device_config;

#[cfg(feature = "btleplug")]
pub mod btleplug;

pub use device_config::*;

pub type Result<T> = anyhow::Result<T>;
