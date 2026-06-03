mod nw;
mod db;
mod ui;
mod notifications;
mod logging;
pub mod run;

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))] compile_error!("Unsupported target");


uniffi::setup_scaffolding!();