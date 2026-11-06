//! The kernel's trap handler.

use thisos_hal::{Platform, Trap};

use crate::{Arch, panic, timer};

/// The kernel's trap handler, which every entry point passes to `entry!`.
/// A breakpoint returns, and a timer interrupt counts a tick. Until the
/// kernel has user space, any other trap is a kernel bug, and panics.
pub fn trap(trap: Trap, frame: &mut <Arch as Platform>::TrapFrame) {
    match trap {
        Trap::Breakpoint => {}
        Trap::Timer => {
            // Before the tick, so that a test's own handler, which runs
            // first, sees the same count the watchdog does.
            #[cfg(feature = "testing")]
            crate::testing::watchdog(frame);
            timer::tick();
        }
        _ => panic::trap(trap, frame),
    }
}

#[cfg(test)]
mod tests {
    use thisos_hal::{Cpu, Platform};

    use crate::Arch;

    // Passes by returning: with no IDT entry the CPU triple-faults, and a
    // handler that panics fails the run.
    #[test_case]
    fn breakpoint_returns() {
        Arch::breakpoint();
    }

    #[test_case]
    fn legacy_interrupts_stay_masked() {
        assert!(Arch::legacy_interrupts_masked());
    }
}
