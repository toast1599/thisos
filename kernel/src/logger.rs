//! The `log` backend: writes each record to the serial port, when there is
//! one, and to the framebuffer console.

use core::fmt::Write;

use log::{Level, LevelFilter, Log, Metadata, Record};
use spin::Mutex;
use thisos_console::{Color, Console};
use thisos_hal::{Framebuffer, Platform};

type Serial = <crate::Arch as Platform>::Serial;

struct Logger {
    // One lock for both, so that records reach them in the same order.
    sinks: Mutex<Sinks>,
}

struct Sinks {
    serial: Option<Serial>,
    console: Console<'static>,
}

static LOGGER: Logger = Logger {
    sinks: Mutex::new(Sinks {
        serial: None,
        console: Console::new(),
    }),
};

/// Installs the logger. Only `kmain` calls this, once.
pub fn init(serial: Option<Serial>, framebuffer: Option<Framebuffer<'static>>) {
    {
        let mut sinks = LOGGER.sinks.lock();
        sinks.serial = serial;
        if let Some(framebuffer) = framebuffer {
            sinks.console.attach(framebuffer);
        }
    }
    // This fails only when a logger is already set, and this is the only
    // place that sets one.
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    });
}

fn color(level: Level) -> Color {
    match level {
        Level::Error => Color::Red,
        Level::Warn => Color::Yellow,
        Level::Info => Color::Green,
        Level::Debug => Color::Blue,
        Level::Trace => Color::DarkGray,
    }
}

impl Log for Logger {
    // The log macros have already checked the level against max_level.
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    // A record that fails to format has nowhere else to be reported, so
    // the results of the writes are dropped.
    fn log(&self, record: &Record) {
        let mut sinks = self.sinks.lock();
        let Sinks { serial, console } = &mut *sinks;
        if let Some(serial) = serial {
            let _ = writeln!(
                serial,
                "{} {}: {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
        console.set_color(color(record.level()));
        let _ = write!(console, "{}", record.level());
        console.set_color(Color::Gray);
        let _ = writeln!(console, " {}: {}", record.target(), record.args());
    }

    fn flush(&self) {}
}
