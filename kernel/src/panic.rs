//! The panic handler: reports the panic with a backtrace to both log sinks,
//! then halts.

use core::fmt;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use thisos_hal::{BacktraceEnd, Cpu, MAX_FRAMES};

use crate::{Arch, logger};

static PANICKING: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // A panic while reporting one would recurse, and the first report is
    // the one worth reading.
    if PANICKING.swap(true, Ordering::Relaxed) {
        Arch::halt();
    }
    let mut backtrace = Backtrace {
        frames: [0; MAX_FRAMES],
        len: 0,
        end: BacktraceEnd::Done,
    };
    // Walked once, so both sinks show the same frames.
    backtrace.end = Arch::backtrace(|address| {
        if let Some(slot) = backtrace.frames.get_mut(backtrace.len) {
            *slot = address;
            backtrace.len += 1;
        }
    });
    match info.location() {
        Some(location) => logger::panic(
            format_args!("PANIC at {location}: {}", info.message()),
            &backtrace,
        ),
        None => logger::panic(format_args!("PANIC: {}", info.message()), &backtrace),
    }
    Arch::halt()
}

struct Backtrace {
    frames: [u64; MAX_FRAMES],
    len: usize,
    end: BacktraceEnd,
}

impl fmt::Display for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "backtrace (link addresses):")?;
        for (i, address) in self.frames.iter().take(self.len).enumerate() {
            writeln!(f, "  #{i} {address:#x}")?;
        }
        match self.end {
            BacktraceEnd::Done => Ok(()),
            BacktraceEnd::TooDeep => writeln!(f, "  stopped: after {MAX_FRAMES} frames"),
            BacktraceEnd::OutsideStack(fp) => {
                writeln!(f, "  stopped: frame pointer {fp:#x} outside the stack")
            }
            BacktraceEnd::Misaligned(fp) => {
                writeln!(f, "  stopped: frame pointer {fp:#x} misaligned")
            }
            BacktraceEnd::NotIncreasing(fp) => {
                writeln!(f, "  stopped: frame pointer {fp:#x} not above the last")
            }
        }
    }
}
