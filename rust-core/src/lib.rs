mod network;
mod database;
mod ui;
mod logging;
mod notifications;
mod ffi_error;
pub mod testing_utilities;

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))] compile_error!("Unsupported target");


uniffi::setup_scaffolding!();