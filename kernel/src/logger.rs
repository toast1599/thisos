//! The `log` backend: writes each record to the serial port, when there is
//! one, and to the framebuffer console.

use core::fmt::{self, Write};

use log::{Level, LevelFilter, Log, Metadata, Record};
use spin::Mutex;
use thisos_console::{Color, Console};
use thisos_hal::{Framebuffer, Platform};

use crate::Arch;

type Serial = <Arch as Platform>::Serial;

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

/// Writes a panic report: `head` as an Error record, then `body` as it is.
/// When the lock is already held, the panic came from inside the logger, so
/// the report goes to a fresh serial port only: the console may be half
/// updated.
pub fn panic(head: fmt::Arguments, body: &dyn fmt::Display) {
    let record = Record::builder()
        .level(Level::Error)
        .target(env!("CARGO_CRATE_NAME"))
        .args(head)
        .build();
    if let Some(mut sinks) = LOGGER.sinks.try_lock() {
        let Sinks { serial, console } = &mut *sinks;
        if let Some(serial) = serial {
            write_serial(serial, &record, body);
        }
        write_console(console, &record, body);
    } else if let Some(mut serial) = Arch::panic_serial() {
        write_serial(&mut serial, &record, body);
    }
}

// A record that fails to format has nowhere else to be reported, so this
// and write_console drop the results of their writes.
fn write_serial(serial: &mut Serial, record: &Record, body: &dyn fmt::Display) {
    let _ = write!(
        serial,
        "{} {}: {}\n{body}",
        record.level(),
        record.target(),
        record.args()
    );
}

fn write_console(console: &mut Console, record: &Record, body: &dyn fmt::Display) {
    console.set_color(color(record.level()));
    let _ = write!(console, "{}", record.level());
    console.set_color(Color::Gray);
    let _ = write!(console, " {}: {}\n{body}", record.target(), record.args());
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

    fn log(&self, record: &Record) {
        let mut sinks = self.sinks.lock();
        let Sinks { serial, console } = &mut *sinks;
        if let Some(serial) = serial {
            write_serial(serial, record, &"");
        }
        write_console(console, record, &"");
    }

    fn flush(&self) {}
}
