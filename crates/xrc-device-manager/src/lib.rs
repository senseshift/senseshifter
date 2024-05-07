pub mod api;
#[cfg(feature = "manager")]
mod manager;

pub type Result<T> = xrc_commons::Result<T>;
