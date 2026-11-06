//! The kernel's trap handler.

use thisos_hal::{Platform, Trap};

use crate::{Arch, panic, timer};

/// The kernel's trap handler, which every entry point passes to `entry!`.
/// A breakpoint returns, and a timer interrupt counts a tick. Until the
/// kernel has user space, any other trap is a kernel bug, and panics.
pub fn trap(trap: Trap, frame: &mut <Arch as Platform>::TrapFrame) {
    #[cfg(test)]
    if crate::testing::caught(trap, frame) {
        return;
    }
    match trap {
        Trap::Breakpoint => {}
        Trap::Timer => {
            #[cfg(test)]
            crate::testing::watchdog(frame);
            timer::tick();
        }
        _ => panic::trap(trap, frame),
    }
}

#[cfg(test)]
mod tests {
    use thisos_hal::{Cpu, Platform, Probe, Trap};

    use crate::{Arch, testing};

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

    // The CPU can't push the page fault's frame onto a full stack, so the
    // overflow reaches the handler as a double fault, on a stack of its own.
    // The tests after this one show the kernel carries on after it.
    #[test_case]
    fn a_stack_overflow_double_faults() {
        testing::expect_trap(|trap| trap == Trap::DoubleFault, Probe::OverflowStack);
    }
}
