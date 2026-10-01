//! Previously, the log crate would route all logs to the stout
//! Mobile apps implement their own logging systems in which these logs may not appear
//! in order to maintain coherence so that app framework logging and internal rust logging can be viewed together 
//! we implement a listener pattern similar to ./notifications.rs. Read documentation there for more explanation of
//! how dynamic trait objects are used to call unknown foreign language code from a rust library.
//! The difference is our `LogListener` trait object is owned by a `Logger` object which owned by the the log crate itself

use crate::ffi_error::FfiError;

#[derive(uniffi::Enum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[uniffi::export(callback_interface)]

pub trait LogListener: Send + Sync {
    fn log(
        &self,
        level: LogLevel,
        target: String,
        file: Option<String>,
        line: Option<u32>,
        msg: String,
    );
}

struct Logger {
    listener: Box<dyn LogListener>,
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let level = match record.level() {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Info => LogLevel::Info,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Trace => LogLevel::Trace,
        };

        self.listener.log(
            level,
            record.target().to_string(),
            record.file().map(|s| s.to_string()),
            record.line(),
            format!("{}", record.args()),
        );
    }

    fn flush(&self) {}
}

#[uniffi::export]
pub fn register_native_log_listener(listener: Box<dyn LogListener>) -> Result<(), FfiError> {

    let logger = Logger { listener};

    log::set_boxed_logger(Box::new(logger)).map_err(anyhow::Error::from)?;
    log::set_max_level(log::LevelFilter::Trace);
    Ok(())
}
