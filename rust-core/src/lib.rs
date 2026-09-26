//! The rust library for the ____ app.  

pub mod database;
pub mod ffi_error;
pub mod logging;
pub mod network;
pub mod notifications;
pub mod testing_utilities;
pub mod ui;

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
compile_error!("Unsupported target");

uniffi::setup_scaffolding!();
