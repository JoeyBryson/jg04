mod nw;
mod db;
mod ui;
mod logging;
mod notifications;
pub mod run;

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))] compile_error!("Unsupported target");


uniffi::setup_scaffolding!();