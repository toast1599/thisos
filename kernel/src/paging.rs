//! The kernel's page tables, which the architecture builds from the
//! loader's at boot. Trap handlers never touch them: their lock is a spin
//! lock, and a handler that found it held by the code it interrupted would
//! never return.

use core::ops::Range;

use spin::Mutex;
use thisos_frames::Frame;
#[cfg(test)]
use thisos_hal::Translation;
use thisos_hal::{MapError, PageTables as _, Platform};

use crate::{Arch, frames};

type PageTables = <Arch as Platform>::PageTables;

static PAGE_TABLES: Mutex<Option<Kernel>> = Mutex::new(None);

/// The kernel's page tables, and what of kernel space is taken.
struct Kernel {
    tables: PageTables,
    /// The lowest kernel-space address no stack has taken, which
    /// [`map_stack`] moves up.
    stacks: u64,
}

/// Switches to page tables of the kernel's own. `kernel::init` calls this
/// once, after `frames::init`.
pub fn init() {
    let Some(tables) = Arch::take_page_tables(&mut frames::alloc) else {
        panic!("the page tables were taken already");
    };
    let space = tables.kernel_space();
    log::info!(
        "page tables: the kernel's own, kernel space {:#x}..{:#x}",
        space.start,
        space.end
    );
    *PAGE_TABLES.lock() = Some(Kernel {
        tables,
        stacks: space.start,
    });
}

fn with<T>(f: impl FnOnce(&mut Kernel) -> T) -> T {
    let mut kernel = PAGE_TABLES.lock();
    let Some(kernel) = kernel.as_mut() else {
        panic!("paging::init didn't run");
    };
    f(kernel)
}

/// The virtual addresses [`map`] maps into.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests maps into kernel space yet"
    )
)]
pub fn kernel_space() -> Range<u64> {
    with(|kernel| kernel.tables.kernel_space())
}

/// Maps the page at `page` to `frame` for good, no-execute, and writable if
/// `writable`.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests maps into kernel space yet"
    )
)]
pub fn map(page: u64, frame: Frame, writable: bool) -> Result<(), MapError> {
    with(|kernel| kernel.tables.map(page, frame, writable, &mut frames::alloc))
}

/// Maps a stack of `size` bytes, rounded up to whole pages, in kernel space
/// for good, writable and no-execute, with an unmapped page below it that
/// an overflow runs into. Returns its addresses. Panics if kernel space or
/// the frames run out.
pub fn map_stack(size: usize) -> Range<u64> {
    let Ok(size) = u64::try_from(size.next_multiple_of(4096)) else {
        panic!("a stack of {size:#x} bytes doesn't fit a u64");
    };
    with(|kernel| {
        let stack = kernel.stacks + 4096..kernel.stacks + 4096 + size;
        for page in stack.clone().step_by(4096) {
            let Some(frame) = frames::alloc() else {
                panic!("no free frame for a stack");
            };
            if let Err(e) = kernel.tables.map(page, frame, true, &mut frames::alloc) {
                panic!("mapping a stack page at {page:#x} failed: {:?}", e.kind);
            }
        }
        kernel.stacks = stack.end;
        stack
    })
}

/// Maps `frame` at the end of the heap and adds it to the heap.
pub fn grow_heap(frame: Frame) -> Result<(), MapError> {
    with(|kernel| kernel.tables.grow_heap(frame, &mut frames::alloc))
}

/// Where `addr` is mapped, or `None` if it isn't.
#[cfg(test)]
pub fn translate(addr: u64) -> Option<Translation> {
    with(|kernel| kernel.tables.translate(addr))
}

/// Whether anything is mapped in the lower half.
#[cfg(test)]
fn lower_half_mapped() -> bool {
    with(|kernel| kernel.tables.lower_half_mapped())
}

#[cfg(test)]
mod tests {
    use core::ptr;
    use core::sync::atomic::AtomicU64;

    use thisos_hal::{Access, MapErrorKind, Probe, Trap};

    use super::*;
    use crate::testing;

    static PROBE: AtomicU64 = AtomicU64::new(0);

    /// Whether the memory at `p` is writable and executable, or `None` if
    /// it isn't mapped.
    fn access<T>(p: *const T) -> Option<(bool, bool)> {
        let t = translate(u64::try_from(p.addr()).ok()?)?;
        Some((t.writable, t.executable))
    }

    fn code() {}

    #[test_case]
    fn code_is_executable_and_read_only() {
        let f: fn() = code;
        #[expect(
            clippy::as_conversions,
            reason = "a function pointer becomes an address only through as"
        )]
        let p = f as *const ();
        assert_eq!(access(p), Some((false, true)));
    }

    #[test_case]
    fn statics_and_the_stack_are_writable_and_no_execute() {
        let local = 0u64;
        assert_eq!(access(ptr::from_ref(&PROBE)), Some((true, false)));
        assert_eq!(access(ptr::from_ref(&local)), Some((true, false)));
    }

    #[test_case]
    fn the_guard_page_is_unmapped() {
        assert_eq!(translate(Arch::guard_page()), None);
    }

    #[test_case]
    fn reading_the_guard_page_faults() {
        testing::expect_trap(
            |trap| {
                trap == Trap::PageFault {
                    address: Arch::guard_page(),
                    access: Access::Read,
                }
            },
            Probe::ReadGuardPage,
        );
    }

    #[test_case]
    fn the_lower_half_is_unmapped() {
        assert!(!lower_half_mapped());
    }

    /// Whether `trap` is the page fault on `access` that ends `probe`.
    fn faults_on_target(probe: Probe, access: Access, trap: Trap) -> bool {
        Arch::probe_target(probe).is_some_and(|address| trap == Trap::PageFault { address, access })
    }

    /// Checks that `probe`'s target is mapped, writable and executable as
    /// `expected` says.
    fn check_target(probe: Probe, expected: (bool, bool)) {
        let Some(address) = Arch::probe_target(probe) else {
            panic!("{probe:?} has no target");
        };
        let mapped = translate(address).map(|t| (t.writable, t.executable));
        assert_eq!(mapped, Some(expected), "{probe:?} target {address:#x}");
    }

    // The entry says read-only; the fault shows the CPU enforces it, which
    // it does in ring 0 only with CR0.WP on.
    #[test_case]
    fn writing_a_read_only_page_faults() {
        check_target(Probe::WriteReadOnly, (false, false));
        testing::expect_trap(
            |trap| faults_on_target(Probe::WriteReadOnly, Access::Write, trap),
            Probe::WriteReadOnly,
        );
    }

    // The entry says no-execute; the fault shows the CPU enforces it, which
    // it does only with EFER.NXE on.
    #[test_case]
    fn calling_a_no_execute_page_faults() {
        check_target(Probe::ExecuteNoExecute, (false, false));
        testing::expect_trap(
            |trap| faults_on_target(Probe::ExecuteNoExecute, Access::Execute, trap),
            Probe::ExecuteNoExecute,
        );
    }

    #[test_case]
    fn map_puts_a_frame_in_kernel_space() {
        let space = kernel_space();
        // The last page, out of the way of anything that maps from the start.
        let page = space.end - 4096;
        let Some(frame) = frames::alloc() else {
            panic!("no free frame");
        };
        let phys = frame.start();
        if let Err(e) = map(page, frame, true) {
            panic!("mapping {page:#x} failed: {:?}", e.kind);
        }
        assert_eq!(
            translate(page + 8),
            Some(Translation {
                phys: phys + 8,
                writable: true,
                executable: false,
            })
        );
        let Some(frame) = frames::alloc() else {
            panic!("no free frame");
        };
        let Err(e) = map(page, frame, false) else {
            panic!("{page:#x} was mapped twice");
        };
        assert_eq!(e.kind, MapErrorKind::Mapped);
        let Err(e) = map(space.start - 4096, e.frame, false) else {
            panic!("a page below kernel space was mapped");
        };
        assert_eq!(e.kind, MapErrorKind::OutsideKernelSpace);
        frames::free(e.frame);
    }
}
