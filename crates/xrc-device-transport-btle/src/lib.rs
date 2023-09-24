#[cfg(feature = "btleplug")]
pub mod btleplug;

#[cfg(feature = "btleplug")]
pub use btleplug::{
  BtlePlugManager as BtleManager,
  BtlePlugManagerBuilder as BtleManagerBuilder,
};