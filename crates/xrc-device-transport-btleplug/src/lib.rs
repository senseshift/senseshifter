pub mod api;

#[cfg(feature = "manager")]
mod manager;

#[cfg(feature = "manager")]
pub use manager::*;

pub type Result<T> = anyhow::Result<T>;
