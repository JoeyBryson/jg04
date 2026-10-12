//! Forwards tracing's standard single-line output to the native application.
//!
//! Register a [`LogListener`] once during application startup, before another
//! global tracing subscriber is installed. Active spans are included in each
//! formatted line. The `tracing-log` subscriber feature also forwards existing
//! `log` records while the codebase is migrated.

use std::io::{self, Write};
use std::sync::Arc;

use tracing::Metadata;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;

use crate::ffi_error::FfiError;

#[derive(Clone, Copy, uniffi::Enum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[uniffi::export(callback_interface)]
pub trait LogListener: Send + Sync {
    fn log(&self, level: LogLevel, line: String);
}

struct NativeMakeWriter {
    listener: Arc<dyn LogListener>,
}

impl<'a> MakeWriter<'a> for NativeMakeWriter {
    type Writer = NativeWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.make_writer_for_level(LogLevel::Info)
    }

    fn make_writer_for(&'a self, metadata: &Metadata<'_>) -> Self::Writer {
        let level = match *metadata.level() {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::TRACE => LogLevel::Trace,
        };
        self.make_writer_for_level(level)
    }
}

impl NativeMakeWriter {
    fn make_writer_for_level(&self, level: LogLevel) -> NativeWriter {
        NativeWriter {
            listener: Arc::clone(&self.listener),
            level,
            buffer: Vec::new(),
        }
    }
}

struct NativeWriter {
    listener: Arc<dyn LogListener>,
    level: LogLevel,
    buffer: Vec<u8>,
}

impl Write for NativeWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let line = String::from_utf8_lossy(&self.buffer)
            .trim_end_matches(['\r', '\n'])
            .to_string();
        self.buffer.clear();

        if !line.is_empty() {
            self.listener.log(self.level, line);
        }

        Ok(())
    }
}

impl Drop for NativeWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[uniffi::export]
pub fn register_native_log_listener(listener: Box<dyn LogListener>) -> Result<(), FfiError> {
    let layer = tracing_subscriber::fmt::layer()
        .with_writer(NativeMakeWriter {
            listener: Arc::from(listener),
        })
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(layer)
        .try_init()
        .map_err(anyhow::Error::from)?;

    Ok(())
}
