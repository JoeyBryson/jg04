mod database;
mod ffi_error;
mod logging;
mod network;
mod notifications;
pub mod testing_utilities;
mod ui;

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
compile_error!("Unsupported target");

uniffi::setup_scaffolding!();
