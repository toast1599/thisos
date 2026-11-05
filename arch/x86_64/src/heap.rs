//! The kernel heap: talc, as the global allocator, over pages that
//! `KernelPageTables::grow_heap` maps for it at the start of the reserved
//! level 4 entry. Nothing else maps there, and only talc reads or writes
//! the pages.

use core::ptr::NonNull;

use talc::TalcLock;
use talc::source::Manual;
use thisos_hal::HeapStats;
use x86_64::VirtAddr;

/// The heap's share of the reserved level 4 entry, from its start.
pub const SIZE: u64 = 64 << 30;

#[global_allocator]
static TALC: TalcLock<spin::Mutex<()>, Manual> = TalcLock::new(Manual);

/// The heap's pages, mapped from `base` up to `mapped` bytes above it, and
/// where talc's heap over them ends. There is one, in the kernel's page
/// tables.
#[derive(Debug)]
pub struct Heap {
    base: *mut u8,
    mapped: usize,
    /// Where talc's heap ends, or `None` before its first claim. It lies
    /// at most a chunk below `base + mapped`, for alignment.
    end: Option<NonNull<u8>>,
}

// SAFETY: A Heap's pointers are into the heap's pages, and it hands them
// only to talc, under talc's lock, so the thread that holds the Heap makes
// no difference to them.
unsafe impl Send for Heap {}

impl Heap {
    /// The heap at `base`, with nothing mapped yet.
    ///
    /// # Safety
    ///
    /// `base` must be page-aligned, nothing may be mapped in the [`SIZE`]
    /// bytes from it, and nothing but this Heap's owner may map there
    /// afterwards. There may be only one Heap.
    pub unsafe fn new(base: VirtAddr) -> Self {
        Self {
            // The one integer the heap's pointers come from: a kernel-space
            // address that only the Heap's owner maps.
            base: base.as_mut_ptr(),
            mapped: 0,
            end: None,
        }
    }

    /// The address of the page the next [`Heap::grow`] adds.
    pub fn next(&self) -> VirtAddr {
        VirtAddr::from_ptr(self.base.wrapping_add(self.mapped))
    }

    /// Adds the page at [`Heap::next`] to talc's heap.
    ///
    /// # Safety
    ///
    /// The page must be mapped writable, for good, to a frame that nothing
    /// else reads or writes.
    pub unsafe fn grow(&mut self) {
        self.mapped += 4096;
        let end = self.base.wrapping_add(self.mapped);
        let mut talc = TALC.lock();
        self.end = match self.end {
            // A failed claim, too small for talc's metadata, takes the
            // pages so far again on the next grow.
            // SAFETY: The caller mapped every page from base to end for
            // good, and only talc reads or writes them from here on.
            None => unsafe { talc.claim(self.base, self.mapped) },
            // SAFETY: As for claim; talc's heap ends at end, and the pages
            // between it and the new end are mapped and talc's alone.
            Some(heap_end) => Some(unsafe { talc.extend(heap_end, end) }),
        };
    }
}

pub fn stats() -> HeapStats {
    let talc = TALC.lock();
    let counters = talc.counters();
    HeapStats {
        allocated: counters.allocated_bytes,
        claimed: counters.claimed_bytes,
    }
}
