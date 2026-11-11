//! The kernel's trap handler.

use thisos_hal::{Platform, Trap};

use crate::{Arch, paging, panic, thread, timer};

/// Moves the stacks the CPU takes double faults, NMIs and machine checks
/// on to stacks in kernel space with an unmapped page below each.
/// `kernel::init` calls this once, after `paging::init`.
pub fn init() {
    let mut count = 0;
    Arch::move_trap_stacks(&mut |size| {
        count += 1;
        paging::map_stack(size).end
    });
    log::info!("trap stacks: {count}, each with a guard page");
}

/// The kernel's trap handler, which every entry point passes to `entry!`.
/// A breakpoint returns, and a timer interrupt counts a tick and switches
/// to the next ready thread. Until the kernel has user space, any other
/// trap is a kernel bug, and panics.
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
            thread::tick();
        }
        _ => panic::trap(trap, frame),
    }
}

#[cfg(test)]
mod tests {
    use thisos_hal::{Cpu, Platform, Probe, Trap};

    use crate::{Arch, paging, testing, timer};

    // Passes by returning: with no IDT entry the CPU triple-faults, and a
    // handler that panics fails the run.
    #[test_case]
    fn breakpoint_returns() {
        Arch::breakpoint();
    }

    // A trap from code that opened user access, as a user copy does, runs
    // the handler with it blocked, and the catch blocks it again for the
    // code after it.
    #[test_case]
    fn traps_block_user_access() {
        testing::expect_timeout(1, Probe::SpinUserAccessOpen);
        assert!(
            testing::user_access_blocked_at_timeout(),
            "the timer handler ran with user access open"
        );
        assert!(
            Arch::user_access_blocked(),
            "user access stayed open after the catch"
        );
    }

    #[test_case]
    fn legacy_interrupts_stay_masked() {
        assert!(Arch::legacy_interrupts_masked());
    }

    #[test_case]
    fn trap_stacks_have_guard_pages() {
        let mut count = 0;
        for stack in Arch::trap_stacks() {
            count += 1;
            let align = (2 * (stack.end - stack.start)).next_power_of_two();
            assert_eq!(stack.start % align, 0, "stack {stack:#x?} alignment");
            let guard = stack.start - 4096;
            assert_eq!(paging::translate(guard), None, "guard page {guard:#x}");
            for page in stack.step_by(4096) {
                let Some(t) = paging::translate(page) else {
                    panic!("trap stack page {page:#x} isn't mapped");
                };
                assert!(t.writable && !t.executable, "{page:#x}: {t:?}");
            }
        }
        assert!(count > 0, "no trap stacks");
    }

    // The CPU can't push the page fault's frame onto a full stack, so the
    // overflow reaches the handler as a double fault, on a stack of its own.
    // The tests after this one show the kernel carries on after it. The
    // probe runs with interrupts off, and the timer ticking after it shows
    // they came back on.
    #[test_case]
    fn a_stack_overflow_double_faults() {
        testing::expect_trap(|trap| trap == Trap::DoubleFault, Probe::OverflowStack);
        let start = timer::ticks();
        testing::spin_until("a tick after the overflow", || timer::ticks() > start);
    }
}
