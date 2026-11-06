//! The `log` backend: writes each record to the serial port, when there is
//! one, and to the framebuffer console.

use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, Ordering};

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

/// Whether `init` has given the logger its sinks.
static READY: AtomicBool = AtomicBool::new(false);

/// The sinks a panic uses when it can't use the logger's: a second serial
/// handle and a console on a second framebuffer handle. The first panic
/// that needs them sets them up. A static, because the console's grid is
/// too large for the stack.
static PANIC_SINKS: Mutex<Sinks> = Mutex::new(Sinks {
    serial: None,
    console: Console::new(),
});

static PANIC_SINKS_READY: AtomicBool = AtomicBool::new(false);

// The lock is held only with interrupts off, so an interrupt handler that
// logs never finds it held by the code it interrupted. An exception, NMI or
// machine check inside the logger can still find it held, which is what
// report_sinks is for.
fn with_sinks<R>(f: impl FnOnce(&mut Sinks) -> R) -> R {
    Arch::without_interrupts(|| f(&mut LOGGER.sinks.lock()))
}

/// Runs `f` on the logger's sinks, or on the panic sinks when the logger
/// has none yet or its lock is held: then the panic came before
/// `logger::init`, or from inside the logger, where the console may be half
/// updated. Once a panic has used the panic sinks, every later report of
/// that panic finds the logger's lock still held and uses them too.
fn report_sinks<R>(f: impl FnOnce(&mut Sinks) -> R) -> Option<R> {
    Arch::without_interrupts(|| {
        if READY.load(Ordering::Acquire)
            && let Some(mut sinks) = LOGGER.sinks.try_lock()
        {
            return Some(f(&mut sinks));
        }
        let mut sinks = PANIC_SINKS.try_lock()?;
        if !PANIC_SINKS_READY.swap(true, Ordering::Relaxed) {
            sinks.serial = Arch::panic_serial();
            if let Some(framebuffer) = Arch::panic_framebuffer() {
                sinks.console.attach(framebuffer);
            }
        }
        Some(f(&mut sinks))
    })
}

/// Installs the logger. Only `kernel::init` calls this, once.
pub fn init(serial: Option<Serial>, framebuffer: Option<Framebuffer<Pixels>>) {
    with_sinks(|sinks| {
        sinks.serial = serial;
        if let Some(framebuffer) = framebuffer {
            sinks.console.attach(framebuffer);
        }
    });
    READY.store(true, Ordering::Release);
    // This fails only when a logger is already set, and this is the only
    // place that sets one.
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    });
}

/// Writes a panic report: `head` as an Error record, then `body` as it is,
/// to the logger's sinks or the panic sinks (see `report_sinks`).
pub fn panic(head: fmt::Arguments, body: &dyn fmt::Display) {
    let record = Record::builder()
        .level(Level::Error)
        .target(env!("CARGO_CRATE_NAME"))
        .args(head)
        .build();
    report_sinks(|Sinks { serial, console }| {
        if let Some(serial) = serial {
            write_serial(serial, &record, body);
        }
        write_console(console, &record, body);
    });
}

/// Writes the test summary, `body`, then PASS or FAIL: four times the
/// font's size on the console, plain on serial. It goes where the panic
/// report went, when there was one.
#[cfg(test)]
pub fn test_summary(body: &dyn fmt::Display, pass: bool) {
    let (word, color) = if pass {
        ("PASS", Color::Green)
    } else {
        ("FAIL", Color::Red)
    };
    report_sinks(|Sinks { serial, console }| {
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

#[cfg(test)]
mod tests {
    use core::fmt;

    use crate::testing::ExpectPanic;

    struct PanicsWhenShown;

    impl fmt::Display for PanicsWhenShown {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            panic!("panicked inside the logger");
        }
    }

    // The record is formatted with the logger's lock held, so the report
    // has to go through the panic sinks. On the laptop it is the report on
    // the cleared screen above the summary.
    #[test_case]
    const A_PANIC_INSIDE_THE_LOGGER_IS_REPORTED: ExpectPanic = ExpectPanic {
        name: "thisos_kernel::logger::tests::a_panic_inside_the_logger_is_reported",
        run: || log::info!("{PanicsWhenShown}"),
        message: "panicked inside the logger",
    };
}
