//! Page tables, reached through the loader's mapping of all physical
//! memory. Nothing here makes a reference to a table: the CPU's page walker
//! sets Accessed and Dirty bits in the tables at any time, so each access is
//! one atomic operation on one entry.

use core::marker::PhantomData;
use core::ops::Range;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use thisos_frames::Frame;
use thisos_hal::{MapError, MapErrorKind, Translation};
use x86_64::instructions::tlb;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{Page, PageTableIndex, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

use crate::heap::{self, Heap};

// Page-table entry bits.
pub const PRESENT: u64 = 1 << 0;
pub const WRITABLE: u64 = 1 << 1;
pub const PWT: u64 = 1 << 3;
pub const PCD: u64 = 1 << 4;
/// A large page in a level 3 or 2 entry; the PAT bit in a level 1 entry.
pub const PS_OR_PAT: u64 = 1 << 7;
pub const NO_EXECUTE: u64 = 1 << 63;
pub const ADDRESS: u64 = 0x000f_ffff_ffff_f000;

/// The first level 4 entry of the upper half.
const UPPER_HALF: u16 = 256;

/// The page tables at `level_4`. Whoever makes one is the only user of the
/// tables while it lives, apart from the CPU's page walker.
#[derive(Debug)]
pub struct PageTables {
    phys_offset: VirtAddr,
    level_4: u64,
}

impl PageTables {
    /// The page tables CR3 points at.
    ///
    /// # Safety
    ///
    /// `phys_offset` must be where the loader mapped all physical memory,
    /// and nothing else may use the page tables while this exists.
    pub unsafe fn current(phys_offset: VirtAddr) -> Self {
        let (frame, _) = Cr3::read();
        Self {
            phys_offset,
            level_4: frame.start_address().as_u64(),
        }
    }

    /// Entry `index` of the table at physical address `table`.
    fn entry(&self, table: u64, index: PageTableIndex) -> Entry<'_> {
        // The table's address comes from CR3 or a table entry, and reaches
        // memory through the physical-memory mapping.
        let address = self.phys_offset + table + 8 * u64::from(index);
        Entry(address.as_mut_ptr(), PhantomData)
    }

    /// The level 1 entry that maps `page`, when it is mapped as a 4 KiB
    /// page.
    pub fn leaf(&self, page: Page) -> Option<Entry<'_>> {
        let mut table = self.level_4;
        for index in [page.p4_index(), page.p3_index(), page.p2_index()] {
            let value = self.entry(table, index).load();
            if value & PRESENT == 0 || value & PS_OR_PAT != 0 {
                return None;
            }
            table = value & ADDRESS;
        }
        let leaf = self.entry(table, page.p1_index());
        (leaf.load() & PRESENT != 0).then_some(leaf)
    }

    /// The frame `page` maps, when it is mapped as a 4 KiB page.
    pub fn frame(&self, page: Page) -> Option<u64> {
        self.leaf(page).map(|leaf| leaf.load() & ADDRESS)
    }

    /// Makes `frame` an empty table, and returns its physical address. The
    /// frame belongs to the tables from here on.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "taking the Frame stops anyone else from using it"
    )]
    fn new_table(&self, frame: Frame) -> u64 {
        let start = frame.start();
        // A physical address, reached through the physical-memory mapping.
        let table = (self.phys_offset + start).as_mut_ptr::<u64>();
        // SAFETY: The allocator handed over the frame, so nothing else
        // reads or writes it, and it isn't in any table yet, so the CPU's
        // walker doesn't either. Its 512 entries are its 4 KiB, which the
        // physical-memory mapping maps.
        unsafe { ptr::write_bytes(table, 0, 512) };
        start
    }

    /// Copies entries `indices` of the level `level` table at `from` into
    /// a new table, and returns the new table's address. Leaves are copied
    /// as they are, and the tables below are copied in turn.
    fn copy(
        &self,
        from: u64,
        level: u8,
        indices: Range<u16>,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> u64 {
        let Some(frame) = alloc() else {
            panic!("no frames left for the kernel's page tables");
        };
        let to = self.new_table(frame);
        for index in indices.map(PageTableIndex::new) {
            let value = self.entry(from, index).load();
            if value & PRESENT == 0 {
                continue;
            }
            let value = if is_leaf(level, value) {
                value
            } else {
                (value & !ADDRESS) | self.copy(value & ADDRESS, level - 1, 0..512, alloc)
            };
            // SAFETY: The new table isn't in use yet: CR3 doesn't point at
            // its level 4 table.
            unsafe { self.entry(to, index).store(value) };
        }
        to
    }
}

/// Whether `value`, a present entry in a level `level` table, maps a page
/// rather than pointing at a table.
fn is_leaf(level: u8, value: u64) -> bool {
    level == 1 || (level < 4 && value & PS_OR_PAT != 0)
}

/// A page-table entry. Only `PageTables::entry` makes one, from a table it
/// reached through the physical-memory mapping, so the pointer is 8-byte
/// aligned and mapped, and the CPU's page walker, which writes entries
/// atomically, is its only other user while the `PageTables` lives.
#[derive(Clone, Copy)]
pub struct Entry<'t>(*mut u64, PhantomData<&'t PageTables>);

impl Entry<'_> {
    pub fn load(self) -> u64 {
        // SAFETY: As the type says: aligned, mapped, and accessed only
        // atomically. The reference lasts this one load.
        unsafe { AtomicU64::from_ptr(self.0) }.load(Ordering::Relaxed)
    }

    /// Replaces the entry with `value`.
    ///
    /// # Safety
    ///
    /// The entry must not be present, or its table must not be in use, so
    /// that nothing the kernel reaches changes underneath it.
    unsafe fn store(self, value: u64) {
        // SAFETY: As in load.
        unsafe { AtomicU64::from_ptr(self.0) }.store(value, Ordering::Relaxed);
    }

    /// Replaces the entry's value `v` with `f(v)` in one atomic step, so
    /// that an Accessed or Dirty bit the CPU sets meanwhile isn't lost.
    ///
    /// # Safety
    ///
    /// The new value must keep valid everything the kernel reaches through
    /// the entry's pages.
    pub unsafe fn update(self, f: impl Fn(u64) -> u64) {
        // SAFETY: As in load. The reference lasts this function.
        let entry = unsafe { AtomicU64::from_ptr(self.0) };
        let mut current = entry.load(Ordering::Relaxed);
        while let Err(seen) =
            entry.compare_exchange_weak(current, f(current), Ordering::Relaxed, Ordering::Relaxed)
        {
            current = seen;
        }
    }
}

/// The page tables the kernel built and runs on. Only `take` makes one,
/// once.
#[derive(Debug)]
pub struct KernelPageTables {
    tables: PageTables,
    /// The level 4 entry the heap and `map` map into.
    space: PageTableIndex,
    heap: Heap,
}

/// Copies the loader's upper half into new tables made from `alloc`'s
/// frames, switches to them, and returns them. Returns `None` after the
/// first call.
///
/// # Safety
///
/// `phys_offset` must be where the loader mapped all physical memory, and
/// the descriptor tables the CPU uses must be the kernel's own, not the
/// loader's, which it mapped in the lower half.
pub unsafe fn take(
    phys_offset: VirtAddr,
    alloc: &mut dyn FnMut() -> Option<Frame>,
) -> Option<KernelPageTables> {
    static TAKEN: AtomicBool = AtomicBool::new(false);
    if TAKEN.swap(true, Ordering::Relaxed) {
        return None;
    }
    let (_, flags) = Cr3::read();
    // SAFETY: The caller guarantees phys_offset. TAKEN makes this the only
    // call, and only it changes the tables until the KernelPageTables it
    // returns exists.
    let loader = unsafe { PageTables::current(phys_offset) };
    let level_4 = loader.copy(loader.level_4, 4, UPPER_HALF..512, alloc);
    let Some(space) = (UPPER_HALF..512)
        .map(PageTableIndex::new)
        .find(|&i| loader.entry(loader.level_4, i).load() & PRESENT == 0)
    else {
        panic!("the loader left no level 4 entry free in the upper half");
    };
    let frame = PhysFrame::containing_address(PhysAddr::new(level_4));
    // SAFETY: The new tables map every upper-half address to the same frame
    // with the same flags as the loader's, and the kernel's code, stack,
    // statics, boot information, framebuffer and physical-memory mapping
    // are all in the upper half, so everything the kernel reaches stays
    // where it was. The lower half held only the loader's identity
    // mappings, which the caller guarantees nothing uses.
    unsafe { Cr3::write(frame, flags) };
    let heap_base = space_start(space);
    Some(KernelPageTables {
        tables: PageTables {
            phys_offset,
            level_4,
        },
        space,
        // SAFETY: The loader mapped nothing in the space entry, TAKEN makes
        // this the only Heap, and only grow_heap maps in its range, since
        // map stays inside kernel_space, which starts above it.
        heap: unsafe { Heap::new(heap_base) },
    })
}

/// The first address that level 4 entry `index` maps.
fn space_start(index: PageTableIndex) -> VirtAddr {
    let zero = PageTableIndex::new(0);
    Page::<Size4KiB>::from_page_table_indices(index, zero, zero, zero).start_address()
}

impl thisos_hal::PageTables for KernelPageTables {
    fn kernel_space(&self) -> Range<u64> {
        let start = space_start(self.space).as_u64();
        start + heap::SIZE..start + (1 << 39)
    }

    fn map(
        &mut self,
        page: u64,
        frame: Frame,
        writable: bool,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError> {
        let fail = |kind, frame| Err(MapError { kind, frame });
        if page % 4096 != 0 {
            return fail(MapErrorKind::Unaligned, frame);
        }
        if !self.kernel_space().contains(&page) {
            return fail(MapErrorKind::OutsideKernelSpace, frame);
        }
        self.map_page(
            Page::containing_address(VirtAddr::new(page)),
            frame,
            writable,
            alloc,
        )
    }

    fn grow_heap(
        &mut self,
        frame: Frame,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError> {
        let next = self.heap.next();
        assert!(
            next < space_start(self.space) + heap::SIZE,
            "the heap's {} GiB are full",
            heap::SIZE >> 30
        );
        self.map_page(Page::containing_address(next), frame, true, alloc)?;
        // SAFETY: map_page mapped the page writable to the frame for good,
        // and nothing else maps in the heap's range or reaches the frame.
        unsafe { self.heap.grow() };
        Ok(())
    }

    fn translate(&self, addr: u64) -> Option<Translation> {
        let addr = VirtAddr::try_new(addr).ok()?;
        let page = Page::<Size4KiB>::containing_address(addr);
        let indices = [
            page.p4_index(),
            page.p3_index(),
            page.p2_index(),
            page.p1_index(),
        ];
        let (mut table, mut writable, mut executable) = (self.tables.level_4, true, true);
        for (level, index) in (1..=4).rev().zip(indices) {
            let value = self.tables.entry(table, index).load();
            if value & PRESENT == 0 {
                return None;
            }
            writable &= value & WRITABLE != 0;
            executable &= value & NO_EXECUTE == 0;
            if is_leaf(level, value) {
                let size = 1u64 << (12 + 9 * (level - 1));
                return Some(Translation {
                    phys: (value & ADDRESS & !(size - 1)) | (addr.as_u64() & (size - 1)),
                    writable,
                    executable,
                });
            }
            table = value & ADDRESS;
        }
        None
    }
}

impl KernelPageTables {
    /// Maps `page` to `frame` as `map` does, wherever `page` is.
    fn map_page(
        &mut self,
        page: Page<Size4KiB>,
        frame: Frame,
        writable: bool,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError> {
        let fail = |kind, frame| Err(MapError { kind, frame });
        let mut table = self.tables.level_4;
        for index in [page.p4_index(), page.p3_index(), page.p2_index()] {
            let entry = self.tables.entry(table, index);
            let mut value = entry.load();
            if value & PRESENT == 0 {
                let Some(new) = alloc() else {
                    return fail(MapErrorKind::NoFrames, frame);
                };
                value = self.tables.new_table(new) | PRESENT | WRITABLE;
                // SAFETY: The entry isn't present, so nothing is reached
                // through it yet.
                unsafe { entry.store(value) };
            }
            table = value & ADDRESS;
        }
        let leaf = self.tables.entry(table, page.p1_index());
        if leaf.load() & PRESENT != 0 {
            return fail(MapErrorKind::Mapped, frame);
        }
        let write = if writable { WRITABLE } else { 0 };
        // SAFETY: The entry isn't present, so nothing is reached through it
        // yet, and the frame is the caller's to give: it stays mapped here
        // for good.
        unsafe { leaf.store(frame.start() | PRESENT | NO_EXECUTE | write) };
        tlb::flush(page.start_address());
        Ok(())
    }
}
