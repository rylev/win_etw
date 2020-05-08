//! Provides a `log::Log` implementation that sends events to Event Tracing for Windows (ETW).

#![no_std]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::useless_let_if_seq)]

extern crate alloc;
use alloc::string::{String, ToString};
use core::sync::atomic::{AtomicBool, Ordering};

/// Provides a `log::Log` implementation that sends events to Event Tracing for Windows (ETW).
pub struct TraceLogger {
    provider: RustLogProvider,
    log_module_path: AtomicBool,
    log_file_path: AtomicBool,
}

impl TraceLogger {
    /// Registers the `TraceLogger` with ETW.
    pub fn new() -> Result<Self, win_etw_provider::Error> {
        let provider = RustLogProvider::new()?;
        Ok(TraceLogger {
            provider,
            log_module_path: AtomicBool::new(true),
            log_file_path: AtomicBool::new(true),
        })
    }

    /// Controls whether Rust module paths are included in event records.
    /// The default is `true` (module paths are included).
    /// This is provided to give control over privacy and to control the size of event records.
    pub fn set_log_module_path(&self, value: bool) {
        self.log_module_path.store(value, Ordering::Release);
    }

    /// Controls whether source file names and line numbers are included in event
    /// records. The default is `true` (source file names and line numbers are included).
    /// This is provided to give control over privacy and to control the size of event records.
    pub fn set_log_file_path(&self, value: bool) {
        self.log_file_path.store(value, Ordering::Release);
    }

    /// Returns `true` if this logger will include Rust module paths in event records.
    pub fn log_module_path(&self) -> bool {
        self.log_module_path.load(Ordering::Acquire)
    }

    /// Returns `true` if this logger will include source file names and line numbers in event
    /// records.
    pub fn log_file_path(&self) -> bool {
        self.log_file_path.load(Ordering::Acquire)
    }
}

macro_rules! impl_log_levels {
    (
        $( $snake_level:ident, $camel_level:ident; )*
    ) => {

        impl log::Log for TraceLogger {
            fn enabled(&self, _metadata: &log::Metadata) -> bool {
                true // self.provider.log_is_enabled()
            }

            fn log(&self, record: &log::Record) {
                let module_path = if self.log_module_path() {
                    record.module_path().unwrap_or("")
                } else {
                    ""
                };

                let file_path;
                let file_line;
                if self.log_file_path() {
                    file_path = record.file().unwrap_or("");
                    file_line = record.line().unwrap_or(0);
                } else {
                    file_path = "";
                    file_line = 0;
                }

                let message: String = record.args().to_string();

                let metadata = record.metadata();

                match metadata.level() {
                    $(
                        log::Level::$camel_level => {
                            self.provider.$snake_level(None, module_path, file_path, file_line, &message);
                        }
                    )*
                }
            }

            fn flush(&self) {}
        }

    }
}

#[win_etw_macros::trace_logging_events(guid = "7f006a22-73fb-4c17-b1eb-0a3070f9f187")]
trait RustLogProvider {
    // $( fn $snake_level(module_path: &str, file: &str, line: u32, message: &str); )*
    fn error(module_path: &str, file: &str, line: u32, message: &str);
    fn warn(module_path: &str, file: &str, line: u32, message: &str);
    fn info(module_path: &str, file: &str, line: u32, message: &str);
    fn debug(module_path: &str, file: &str, line: u32, message: &str);
    fn trace(module_path: &str, file: &str, line: u32, message: &str);
}

impl_log_levels! {
    error, Error;
    warn, Warn;
    info, Info;
    debug, Debug;
    trace, Trace;
}
