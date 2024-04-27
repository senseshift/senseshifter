pub mod api;
#[cfg(feature = "manager")]
mod manager;

pub type Result<T> = anyhow::Result<T>;
