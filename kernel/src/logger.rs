//! The `log` backend: writes each record to the serial port, when there is
//! one.

use core::fmt::Write;

use log::{LevelFilter, Log, Metadata, Record};
use spin::Mutex;
use thisos_hal::Platform;

type Serial = <crate::Arch as Platform>::Serial;

struct Logger {
    serial: Mutex<Option<Serial>>,
}

static LOGGER: Logger = Logger {
    serial: Mutex::new(None),
};

/// Installs the logger. Only `kmain` calls this, once.
pub fn init(serial: Option<Serial>) {
    *LOGGER.serial.lock() = serial;
    // This fails only when a logger is already set, and this is the only
    // place that sets one.
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    });
}

impl Log for Logger {
    // The log macros have already checked the level against max_level.
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if let Some(serial) = self.serial.lock().as_mut() {
            // A record that fails to format has nowhere else to be reported.
            let _ = writeln!(
                serial,
                "{} {}: {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
