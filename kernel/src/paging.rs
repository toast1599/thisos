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
type AddressSpace = <PageTables as thisos_hal::PageTables>::AddressSpace;

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

/// Unmaps the page at `page` in kernel space, and returns its frame, or
/// `None` if nothing was mapped there. Its address is never mapped again.
pub fn unmap(page: u64) -> Option<Frame> {
    with(|kernel| kernel.tables.unmap(page))
}

/// A new address space with an empty lower half, or `None` if no frame is
/// free.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests makes a space until init runs"
    )
)]
pub fn new_space() -> Option<AddressSpace> {
    with(|kernel| kernel.tables.new_space(&mut frames::alloc))
}

/// Makes the CPU use the kernel's own tables, with an empty lower half.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests switches spaces until threads"
    )
)]
pub fn activate() {
    with(|kernel| kernel.tables.activate());
}

/// Maps a stack of `size` bytes, rounded up to whole pages, in kernel space
/// for good, writable and no-execute, with an unmapped page below it that
/// an overflow runs into. Its start is aligned to the power of two at or
/// above twice its size, so that an architecture can tell an overflow by
/// one bit of the stack pointer. Returns its addresses. Panics if kernel
/// space or the frames run out.
pub fn map_stack(size: usize) -> Range<u64> {
    let Ok(size) = u64::try_from(size.next_multiple_of(4096)) else {
        panic!("a stack of {size:#x} bytes doesn't fit a u64");
    };
    let align = (2 * size).next_power_of_two();
    with(|kernel| {
        let start = (kernel.stacks + 4096).next_multiple_of(align);
        let stack = start..start + size;
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
    use alloc::vec::Vec;
    use core::ptr;
    use core::sync::atomic::AtomicU64;

    use thisos_hal::{Access, AddressSpace as _, MapErrorKind, Probe, Protection, Trap};

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
                user: false,
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

    fn translate_active(addr: u64) -> Option<Translation> {
        Arch::translate_active(addr)
    }

    fn frame() -> Frame {
        let Some(frame) = frames::alloc() else {
            panic!("no free frame");
        };
        frame
    }

    fn space() -> AddressSpace {
        let Some(space) = new_space() else {
            panic!("no free frame for a space");
        };
        space
    }

    fn map_user(space: &mut AddressSpace, page: u64, protection: Protection) -> u64 {
        let frame = frame();
        let phys = frame.start();
        if let Err(e) = space.map(page, frame, protection, &mut frames::alloc) {
            panic!("mapping {page:#x} failed: {:?}", e.kind);
        }
        phys
    }

    /// Destroys `space`, frees every frame, and returns the addresses of
    /// those it still mapped.
    fn destroy(space: AddressSpace) -> Vec<u64> {
        let mut mapped = Vec::new();
        space.destroy(
            &mut |frame| {
                mapped.push(frame.start());
                frames::free(frame);
            },
            &mut frames::free,
        );
        mapped
    }

    #[test_case]
    fn a_space_maps_translates_and_unmaps() {
        let mut space = space();
        let pages = [
            (Protection::ReadOnly, false, false),
            (Protection::ReadWrite, true, false),
            (Protection::ReadExecute, false, true),
        ];
        let mut mapped = Vec::new();
        for (i, (protection, writable, executable)) in (0u64..).zip(pages) {
            let page = 0x40_0000 + 4096 * i;
            let phys = map_user(&mut space, page, protection);
            assert_eq!(
                space.translate(page + 8),
                Some(Translation {
                    phys: phys + 8,
                    writable,
                    executable,
                    user: true,
                }),
                "{protection:?}"
            );
            mapped.push((page, phys));
        }
        for (page, phys) in mapped {
            let Some(frame) = space.unmap(page) else {
                panic!("{page:#x} wasn't mapped");
            };
            assert_eq!(frame.start(), phys);
            frames::free(frame);
            assert_eq!(space.translate(page), None);
            assert!(space.unmap(page).is_none(), "{page:#x} unmapped twice");
        }
        assert_eq!(destroy(space), []);
    }

    #[test_case]
    fn user_maps_stay_in_user_space() {
        let mut space = space();
        let lowest = space.user_space().start;
        for (page, kind) in [
            (0x40_0001, MapErrorKind::Unaligned),
            (lowest - 4096, MapErrorKind::OutsideUserSpace),
            (kernel_space().start, MapErrorKind::OutsideUserSpace),
        ] {
            let Err(e) = space.map(page, frame(), Protection::ReadOnly, &mut frames::alloc) else {
                panic!("{page:#x} was mapped");
            };
            assert_eq!(e.kind, kind, "{page:#x}");
            frames::free(e.frame);
        }
        let phys = map_user(&mut space, lowest, Protection::ReadOnly);
        let Err(e) = space.map(lowest, frame(), Protection::ReadOnly, &mut frames::alloc) else {
            panic!("{lowest:#x} was mapped twice");
        };
        assert_eq!(e.kind, MapErrorKind::Mapped);
        frames::free(e.frame);
        assert_eq!(destroy(space), [phys]);
    }

    #[test_case]
    fn each_space_sees_its_own_lower_half() {
        let page = 0x40_0000;
        let (mut a, mut b) = (space(), space());
        let phys_a = map_user(&mut a, page, Protection::ReadWrite);
        let phys_b = map_user(&mut b, page, Protection::ReadWrite);
        let Ok(statics) = u64::try_from(ptr::from_ref(&PROBE).addr()) else {
            panic!("a static's address doesn't fit a u64");
        };
        let kernel = translate_active(statics);
        assert!(kernel.is_some(), "the kernel's statics aren't mapped");
        for (space, phys) in [(&a, phys_a), (&b, phys_b)] {
            space.activate();
            assert_eq!(translate_active(page).map(|t| t.phys), Some(phys));
            assert_eq!(translate_active(statics), kernel);
        }
        activate();
        assert_eq!(translate_active(page), None);
        assert_eq!(destroy(a), [phys_a]);
        assert_eq!(destroy(b), [phys_b]);
    }

    // The pages sit under three different level 3 entries, so destroy has
    // a table of each level to free for each.
    #[test_case]
    fn destroying_the_active_space_gives_back_every_frame() {
        let before = frames::free_frames();
        let mut space = space();
        let pages = [0x1_0000, 0x40_0000_0000, 0x7fff_ffff_f000];
        let mut phys: Vec<u64> = pages
            .iter()
            .map(|&page| map_user(&mut space, page, Protection::ReadWrite))
            .collect();
        space.activate();
        let mut mapped = destroy(space);
        assert_eq!(translate_active(pages[0]), None, "still in the space");
        phys.sort_unstable();
        mapped.sort_unstable();
        assert_eq!(mapped, phys);
        assert_eq!(frames::free_frames(), before);
    }

    #[test_case]
    fn unmap_gives_back_a_kernel_space_frame() {
        // Below the page map_puts_a_frame_in_kernel_space leaves mapped.
        let page = kernel_space().end - 2 * 4096;
        let frame = frame();
        let phys = frame.start();
        if let Err(e) = map(page, frame, true) {
            panic!("mapping {page:#x} failed: {:?}", e.kind);
        }
        let Some(frame) = unmap(page) else {
            panic!("{page:#x} wasn't mapped");
        };
        assert_eq!(frame.start(), phys);
        frames::free(frame);
        assert_eq!(translate(page), None);
        assert!(unmap(page).is_none(), "{page:#x} unmapped twice");
    }

    /// Maps a frame at `probe`'s user target in a new space, with
    /// `protection`, after `fill` has written it, and runs `probe` there,
    /// expecting a trap for which `expected` is true.
    fn probe_user(
        probe: Probe,
        protection: Protection,
        fill: fn(&Frame),
        expected: fn(Trap) -> bool,
    ) {
        let Some(target) = Arch::probe_target(probe) else {
            panic!("{probe:?} has no target");
        };
        let mut space = space();
        let frame = frame();
        fill(&frame);
        let phys = frame.start();
        if let Err(e) = space.map(target, frame, protection, &mut frames::alloc) {
            panic!("mapping {target:#x} failed: {:?}", e.kind);
        }
        space.activate();
        // Mapped for user code, so only SMEP or SMAP can make it fault.
        assert_eq!(
            translate_active(target).map(|t| (t.phys, t.user)),
            Some((phys, true))
        );
        testing::expect_trap(expected, probe);
        activate();
        assert_eq!(destroy(space), [phys]);
    }

    #[test_case]
    fn the_kernel_cant_read_user_pages() {
        probe_user(
            Probe::ReadUser,
            Protection::ReadOnly,
            |_| {},
            |trap| faults_on_target(Probe::ReadUser, Access::Read, trap),
        );
    }

    // The frame holds a ret, so if SMEP let the call through it would
    // return, and the test would time out instead of running garbage.
    #[test_case]
    fn the_kernel_cant_run_user_pages() {
        probe_user(
            Probe::ExecuteUser,
            Protection::ReadExecute,
            Arch::write_return,
            |trap| faults_on_target(Probe::ExecuteUser, Access::Execute, trap),
        );
    }
}
