//! The timer tick, which counts time for the test watchdog.

use core::sync::atomic::{AtomicU64, Ordering};

use thisos_hal::{Cpu, Platform};

use crate::Arch;

/// Ticks per second.
pub const HZ: u32 = 100;

static TICKS: AtomicU64 = AtomicU64::new(0);

/// Starts the timer and turns interrupts on.
pub fn init() {
    let frequency = Arch::start_timer(HZ);
    log::info!(
        "timer ticking at {HZ} Hz, counting at {} Hz from {}",
        frequency.hz,
        frequency.source
    );
    Arch::enable_interrupts();
}

/// The ticks since [`init`]. Ticks that come while interrupts are off,
/// as they are while a record is logged, count as one.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "only the test watchdog reads the ticks so far")
)]
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Counts a tick. The kernel's trap handler calls this on every timer
/// interrupt.
pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
    #[cfg(test)]
    tests::tick();
}

#[cfg(test)]
mod tests {
    use core::hint::spin_loop;
    use core::sync::atomic::{AtomicBool, Ordering};

    use thisos_hal::Platform;

    use crate::Arch;

    /// Far more spins than 100 ms takes under TCG, and far fewer than the
    /// watchdog's 10 s on real hardware.
    const MAX_SPINS: u64 = 1 << 30;

    static LOG_NEXT: AtomicBool = AtomicBool::new(false);
    static LOGGED: AtomicBool = AtomicBool::new(false);

    pub(super) fn tick() {
        if LOG_NEXT.swap(false, Ordering::Relaxed) {
            log::info!("logged from the timer interrupt at tick {}", super::ticks());
            LOGGED.store(true, Ordering::Relaxed);
        }
    }

    /// Spins until `done` holds, and panics after [`MAX_SPINS`].
    fn spin_until(what: &str, done: impl Fn() -> bool) {
        for _ in 0..MAX_SPINS {
            if done() {
                return;
            }
            spin_loop();
        }
        panic!("{what} didn't happen within {MAX_SPINS} spins");
    }

    #[test_case]
    fn ticks_advance() {
        let start = super::ticks();
        spin_until("10 ticks", || super::ticks() >= start + 10);
    }

    #[test_case]
    fn handler_can_log() {
        LOG_NEXT.store(true, Ordering::Relaxed);
        spin_until("a record from the timer interrupt", || {
            LOGGED.load(Ordering::Relaxed)
        });
    }

    // Runs only where the CPU reports the timer's frequency, which QEMU
    // doesn't.
    #[test_case]
    fn reported_frequency_matches_a_measurement() {
        // Interrupts are off while it measures, so the watchdog can't stop
        // it, and this line shows where a hang is.
        log::info!("measuring the timer against another clock");
        let frequencies = Arch::timer_frequencies();
        let Some(reported) = frequencies.reported else {
            log::info!("the CPU reports no timer frequency to check");
            return;
        };
        let Some(measured) = frequencies.measured else {
            panic!("the CPU reports {reported} Hz, but measuring it failed");
        };
        let tolerance = reported / 20;
        assert!(
            measured.hz.abs_diff(reported) <= tolerance,
            "the CPU reports {reported} Hz, but {} measures {} Hz",
            measured.source,
            measured.hz
        );
    }
}
