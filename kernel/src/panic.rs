//! The panic handler: reports the panic with a backtrace to both log sinks,
//! then halts. In a test build it ends the test run with failure instead.
//! Traps the kernel can't handle end the same way.

use core::fmt;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use thisos_hal::{BacktraceEnd, Cpu, MAX_FRAMES, Platform, Trap, TrapFrame};

use crate::{Arch, logger};

static PANICKING: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    enter();
    let mut backtrace = Backtrace::new();
    // Walked once, so both sinks show the same frames.
    backtrace.end = Arch::backtrace(|address| backtrace.frames.push(address));
    match info.location() {
        Some(location) => logger::panic(
            format_args!("PANIC at {location}: {}", info.message()),
            &backtrace,
        ),
        None => logger::panic(format_args!("PANIC: {}", info.message()), &backtrace),
    }
    end()
}

/// Reports `trap` as a panic, with the registers in `frame` and a backtrace
/// of the trapped code, then ends as a panic does. Not through `panic!`,
/// whose backtrace would start in the handler, and for a trap on a stack of
/// its own stop there at once.
pub fn trap(trap: Trap, frame: &<Arch as Platform>::TrapFrame) -> ! {
    trapped(format_args!("{trap}"), frame)
}

/// As [`trap`], with `head` in place of the trap's name.
pub fn trapped(head: fmt::Arguments, frame: &<Arch as Platform>::TrapFrame) -> ! {
    enter();
    let mut backtrace = Backtrace::new();
    backtrace.end = frame.backtrace(|address| backtrace.frames.push(address));
    logger::panic(
        format_args!("PANIC: {head}"),
        &format_args!("{frame}{backtrace}"),
    );
    end()
}

fn enter() {
    // A panic while reporting one would recurse, and the first report is
    // the one worth reading.
    if PANICKING.swap(true, Ordering::Relaxed) {
        stop();
    }
}

fn end() -> ! {
    #[cfg(feature = "testing")]
    crate::testing::fail();
    #[cfg(not(feature = "testing"))]
    Arch::halt();
}

fn stop() -> ! {
    if cfg!(feature = "testing") {
        Arch::exit_qemu(false);
    }
    Arch::halt()
}

struct Backtrace {
    frames: Frames,
    end: BacktraceEnd,
}

impl Backtrace {
    fn new() -> Self {
        Backtrace {
            frames: Frames {
                addresses: [0; MAX_FRAMES],
                len: 0,
            },
            end: BacktraceEnd::Done,
        }
    }
}

/// The first [`MAX_FRAMES`] addresses pushed. A trap's backtrace reports
/// one more, and the last is dropped.
struct Frames {
    addresses: [u64; MAX_FRAMES],
    len: usize,
}

impl Frames {
    fn push(&mut self, address: u64) {
        if let Some(slot) = self.addresses.get_mut(self.len) {
            *slot = address;
            self.len += 1;
        }
    }
}

impl fmt::Display for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "backtrace (link addresses):")?;
        for (i, address) in self
            .frames
            .addresses
            .iter()
            .take(self.frames.len)
            .enumerate()
        {
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

#[cfg(test)]
mod tests {
    use core::hint::black_box;

    use thisos_hal::{BacktraceEnd, Cpu};

    use crate::Arch;

    #[test_case]
    fn backtrace_reaches_the_entry() {
        let (frames, end) = outer();
        assert_eq!(end, BacktraceEnd::Done);
        assert!(frames >= 4, "{frames} frames");
    }

    // black_box keeps each call from becoming a tail call, which would
    // reuse the caller's frame and drop it from the chain.
    #[inline(never)]
    fn outer() -> (usize, BacktraceEnd) {
        black_box(middle())
    }

    #[inline(never)]
    fn middle() -> (usize, BacktraceEnd) {
        black_box(inner())
    }

    #[inline(never)]
    fn inner() -> (usize, BacktraceEnd) {
        let mut frames = 0;
        let end = Arch::backtrace(|_| frames += 1);
        (frames, end)
    }
}
