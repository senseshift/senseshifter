pub mod api;

#[cfg(target_os = "windows")]
mod windows;

pub use windows::manager::WindowsRfcommTransportManager as RfcommTransportManager;
#[cfg(target_os = "windows")]
pub use windows::manager::WindowsRfcommTransportManagerBuilder as RfcommTransportManagerBuilder;
