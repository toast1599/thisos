//! The `log` backend: writes each record to the serial port, when there is
//! one, and to the framebuffer console.

use core::fmt::{self, Write};

use log::{Level, LevelFilter, Log, Metadata, Record};
use spin::Mutex;
use thisos_console::{Color, Console};
use thisos_hal::{Cpu, Framebuffer, Platform};

use crate::Arch;

type Serial = <Arch as Platform>::Serial;
type Pixels = <Arch as Platform>::Pixels;

struct Logger {
    // One lock for both, so that records reach them in the same order.
    sinks: Mutex<Sinks>,
}

struct Sinks {
    serial: Option<Serial>,
    console: Console<Pixels>,
}

static LOGGER: Logger = Logger {
    sinks: Mutex::new(Sinks {
        serial: None,
        console: Console::new(),
    }),
};

// The lock is held only with interrupts off, so an interrupt handler that
// logs never finds it held by the code it interrupted. An exception, NMI or
// machine check inside the logger can still find it held, which is what
// the panic path's try_lock is for.
fn with_sinks<R>(f: impl FnOnce(&mut Sinks) -> R) -> R {
    Arch::without_interrupts(|| f(&mut LOGGER.sinks.lock()))
}

fn try_with_sinks<R>(f: impl FnOnce(&mut Sinks) -> R) -> Option<R> {
    Arch::without_interrupts(|| LOGGER.sinks.try_lock().map(|mut sinks| f(&mut sinks)))
}

/// Installs the logger. Only `kernel::init` calls this, once.
pub fn init(serial: Option<Serial>, framebuffer: Option<Framebuffer<Pixels>>) {
    with_sinks(|sinks| {
        sinks.serial = serial;
        if let Some(framebuffer) = framebuffer {
            sinks.console.attach(framebuffer);
        }
    });
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
    let written = try_with_sinks(|Sinks { serial, console }| {
        if let Some(serial) = serial {
            write_serial(serial, &record, body);
        }
        write_console(console, &record, body);
    });
    if written.is_none()
        && let Some(mut serial) = Arch::panic_serial()
    {
        write_serial(&mut serial, &record, body);
    }
}

/// Writes the test summary, `body`, to both sinks, then PASS or FAIL: four
/// times the font's size on the console, plain on serial. When the lock is
/// held, a panic came from inside the logger, and its report on serial is
/// all there is.
#[cfg(test)]
pub fn test_summary(body: &dyn fmt::Display, pass: bool) {
    let (word, color) = if pass {
        ("PASS", Color::Green)
    } else {
        ("FAIL", Color::Red)
    };
    try_with_sinks(|Sinks { serial, console }| {
        if let Some(serial) = serial {
            let _ = writeln!(serial, "{body}{word}");
        }
        console.set_color(Color::Gray);
        let _ = write!(console, "{body}");
        console.set_color(color);
        console.write_large(word, 4);
    });
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

fn write_console(console: &mut Console<Pixels>, record: &Record, body: &dyn fmt::Display) {
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
        with_sinks(|Sinks { serial, console }| {
            if let Some(serial) = serial {
                write_serial(serial, record, &"");
            }
            write_console(console, record, &"");
        });
    }

    fn flush(&self) {}
}
