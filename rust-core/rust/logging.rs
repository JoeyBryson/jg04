use std::sync::{Arc, OnceLock};

static LOGGER_CALLBACK: OnceLock<Arc<dyn RustLogger>> = OnceLock::new();

#[derive(uniffi::Enum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[uniffi::export(callback_interface)]
pub trait RustLogger: Send + Sync {
    fn log(
        &self,
        level: LogLevel,
        target: String,
        file: Option<String>,
        line: Option<u32>,
        msg: String,
    );
}

struct LoggerBridge {
    callback: Arc<dyn RustLogger>,
}

impl log::Log for LoggerBridge {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let level = match record.level() {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn  => LogLevel::Warn,
            log::Level::Info  => LogLevel::Info,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Trace => LogLevel::Trace,
        };

        self.callback.log(
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
pub fn init_native_logger(callback: Box<dyn RustLogger>) {
    let arc: Arc<dyn RustLogger> = Arc::from(callback);

    LOGGER_CALLBACK.set(arc.clone()).ok();

    let bridge = LoggerBridge { callback: arc };

    if let Err(e) = log::set_boxed_logger(Box::new(bridge)) {
        eprintln!(
            "Warning: Failed to set native logger (likely already set): {}",
            e
        );
        return;
    }

    log::set_max_level(log::LevelFilter::Trace);
}