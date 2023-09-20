pub mod api;

#[cfg(feature = "btle-manager")]
mod btleplug;
#[cfg(feature = "serialport-manager")]
mod serialport;

#[cfg(feature = "btle-manager")]
pub(crate) use btleplug::{
  BtlePlugManager as BtleManager,
  BtlePlugManagerBuilder as BtleManagerBuilder,
};
#[cfg(feature = "serialport-manager")]
pub(crate) use serialport::{
  SerialPortManager,
  SerialPortManagerBuilder,
};

