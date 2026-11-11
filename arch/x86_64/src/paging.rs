//! Page tables, reached through the loader's mapping of all physical
//! memory. Nothing here makes a reference to a table: the CPU's page walker
//! sets Accessed and Dirty bits in the tables at any time, so each access is
//! one atomic operation on one entry.

use core::marker::PhantomData;
use core::ops::Range;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::{mem, ptr, slice};

use thisos_frames::Frame;
use thisos_hal::{MapError, MapErrorKind, Protection, Translation};
use x86_64::instructions::{interrupts, tlb};
use x86_64::registers::control::{Cr3, Cr4, Cr4Flags};
use x86_64::structures::paging::{Page, PageTableIndex, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

use crate::heap::{self, Heap};
use crate::once::Once;

// Page-table entry bits.
pub const PRESENT: u64 = 1 << 0;
pub const WRITABLE: u64 = 1 << 1;
pub const USER: u64 = 1 << 2;
pub const PWT: u64 = 1 << 3;
pub const PCD: u64 = 1 << 4;
/// A large page in a level 3 or 2 entry; the PAT bit in a level 1 entry.
pub const PS_OR_PAT: u64 = 1 << 7;
/// In a leaf: kept in the TLB when CR3 changes, with CR4.PGE on.
pub const GLOBAL: u64 = 1 << 8;
pub const NO_EXECUTE: u64 = 1 << 63;
pub const ADDRESS: u64 = 0x000f_ffff_ffff_f000;

/// The first level 4 entry of the upper half.
const UPPER_HALF: u16 = 256;

/// Where user mappings may go: the lower half above its first 64 KiB and
/// below its last page, so that no user instruction ends at the canonical
/// boundary, where SYSCALL would save a RIP that SYSRET can't return to.
pub const USER_SPACE: Range<u64> = 0x1_0000..0x0000_7fff_ffff_f000;

/// The page tables at `level_4`. Whoever makes one is the only user of the
/// tables while it lives, apart from the CPU's page walker, and apart from
/// the upper half of a user space's, which only the kernel's changes.
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

    /// Whether CR3 points at these tables.
    fn active(&self) -> bool {
        Cr3::read().0.start_address().as_u64() == self.level_4
    }

    /// Makes `frame` an empty table, and returns its physical address. The
    /// frame belongs to the tables from here on.
    fn new_table(&self, frame: Frame) -> u64 {
        let start = frame.into_start();
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
    /// with the global bit added, and the tables below are copied in turn.
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
                value | GLOBAL
            } else {
                (value & !ADDRESS) | self.copy(value & ADDRESS, level - 1, 0..512, alloc)
            };
            // SAFETY: The new table isn't in use yet: CR3 doesn't point at
            // its level 4 table.
            unsafe { self.entry(to, index).store(value) };
        }
        to
    }

    /// Maps `page` to `frame` with leaf bits `flags`, and links any new
    /// table on the way with `table_flags`, from `alloc`. Flushes the page
    /// from the TLB if `flush`.
    fn map_page(
        &self,
        page: Page<Size4KiB>,
        frame: Frame,
        flags: u64,
        table_flags: u64,
        flush: bool,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError> {
        let fail = |kind, frame| Err(MapError { kind, frame });
        let mut table = self.level_4;
        for index in [page.p4_index(), page.p3_index(), page.p2_index()] {
            let entry = self.entry(table, index);
            let mut value = entry.load();
            if value & PRESENT == 0 {
                let Some(new) = alloc() else {
                    return fail(MapErrorKind::NoFrames, frame);
                };
                value = self.new_table(new) | PRESENT | table_flags;
                // SAFETY: The entry isn't present, so nothing is reached
                // through it yet.
                unsafe { entry.store(value) };
            }
            // Only the loader's mappings have large pages, and none is in
            // kernel space, the heap's range or the lower half, so this is
            // a kernel bug.
            assert!(
                value & PS_OR_PAT == 0,
                "a large page on the way to {:#x}",
                page.start_address()
            );
            table = value & ADDRESS;
        }
        let leaf = self.entry(table, page.p1_index());
        if leaf.load() & PRESENT != 0 {
            return fail(MapErrorKind::Mapped, frame);
        }
        // SAFETY: The entry isn't present, so nothing is reached through it
        // yet, and the frame is the caller's to give: the entry holds it
        // until unmap reclaims it.
        unsafe { leaf.store(frame.into_start() | PRESENT | flags) };
        if flush {
            tlb::flush(page.start_address());
        }
        Ok(())
    }

    /// Unmaps `page` and returns its frame, or `None` if it isn't mapped as
    /// a 4 KiB page. Flushes the page from the TLB if `flush`.
    ///
    /// # Safety
    ///
    /// The page's frame must have come from `map_page`, nothing may reach
    /// other memory through the page once it is gone, and `flush` must be
    /// set if the TLB may hold the page.
    unsafe fn unmap_page(&self, page: Page<Size4KiB>, flush: bool) -> Option<Frame> {
        let leaf = self.leaf(page)?;
        // SAFETY: The caller guarantees nothing reaches other memory through
        // the page once it is gone.
        let value = unsafe { leaf.clear() };
        if flush {
            tlb::flush(page.start_address());
        }
        // SAFETY: map_page stored into_start of the frame there, and the
        // entry, now clear, was its only holder.
        Some(unsafe { Frame::reclaim(value & ADDRESS) })
    }

    /// Where `addr` is mapped, or `None` if it isn't.
    pub fn translate(&self, addr: u64) -> Option<Translation> {
        let addr = VirtAddr::try_new(addr).ok()?;
        let page = Page::<Size4KiB>::containing_address(addr);
        let indices = [
            page.p4_index(),
            page.p3_index(),
            page.p2_index(),
            page.p1_index(),
        ];
        let (mut table, mut writable, mut executable, mut user) = (self.level_4, true, true, true);
        for (level, index) in (1..=4).rev().zip(indices) {
            let value = self.entry(table, index).load();
            if value & PRESENT == 0 {
                return None;
            }
            writable &= value & WRITABLE != 0;
            executable &= value & NO_EXECUTE == 0;
            user &= value & USER != 0;
            if is_leaf(level, value) {
                let size = 1u64 << (12 + 9 * (level - 1));
                return Some(Translation {
                    phys: (value & ADDRESS & !(size - 1)) | (addr.as_u64() & (size - 1)),
                    writable,
                    executable,
                    user,
                });
            }
            table = value & ADDRESS;
        }
        None
    }

    /// Calls `mapped` with the frame of each leaf under entries `indices`
    /// of the level `level` table at `table`, and `free` with each table
    /// below them.
    ///
    /// # Safety
    ///
    /// Nothing may use the tables or the pages they map any more, and every
    /// frame in them must have come from `map_page` or `new_table`.
    unsafe fn release(
        &self,
        table: u64,
        level: u8,
        indices: Range<u16>,
        mapped: &mut dyn FnMut(Frame),
        free: &mut dyn FnMut(Frame),
    ) {
        for index in indices.map(PageTableIndex::new) {
            let value = self.entry(table, index).load();
            if value & PRESENT == 0 {
                continue;
            }
            let frame = value & ADDRESS;
            if level == 1 {
                // SAFETY: map_page stored into_start of the frame there,
                // and the caller guarantees nothing uses it any more.
                mapped(unsafe { Frame::reclaim(frame) });
            } else {
                // SAFETY: As for this call.
                unsafe { self.release(frame, level - 1, 0..512, mapped, free) };
                // SAFETY: new_table took the frame with into_start, and
                // nothing uses the table any more.
                free(unsafe { Frame::reclaim(frame) });
            }
        }
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

    /// Replaces the entry with `value`. The store is Release, so that what
    /// was written before it, such as the zeroes of a table it links, can't
    /// be moved after it; on x86_64 that costs nothing over a plain store.
    ///
    /// # Safety
    ///
    /// The entry must not be present, or its table must not be in use, so
    /// that nothing the kernel reaches changes underneath it.
    unsafe fn store(self, value: u64) {
        // SAFETY: As in load.
        unsafe { AtomicU64::from_ptr(self.0) }.store(value, Ordering::Release);
    }

    /// Clears the entry, and returns what it held, Accessed and Dirty bits
    /// included.
    ///
    /// # Safety
    ///
    /// Nothing may reach other memory through the entry's page once it is
    /// gone.
    unsafe fn clear(self) -> u64 {
        // SAFETY: As in load.
        unsafe { AtomicU64::from_ptr(self.0) }.swap(0, Ordering::Relaxed)
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

/// Makes CR3 point at the level 4 table at `level_4`, unless it does
/// already: a write flushes the TLB but for global pages.
///
/// # Safety
///
/// The table's upper half must map everything the kernel reaches as the
/// tables CR3 points at now do.
unsafe fn load(level_4: u64) {
    let (current, flags) = Cr3::read();
    let frame = PhysFrame::containing_address(PhysAddr::new(level_4));
    if frame == current {
        return;
    }
    // SAFETY: The caller guarantees the kernel's memory stays where it is.
    unsafe { Cr3::write(frame, flags) };
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

/// [`thisos_hal::PageTables::kernel_space`], which `take` records for
/// checks that have no `KernelPageTables` to ask.
pub static KERNEL_SPACE: Once<Range<u64>> = Once::new();

/// The kernel's level 4 table, which `take` records for `activate_kernel`.
static KERNEL_LEVEL_4: Once<u64> = Once::new();

/// Makes the CPU use the kernel's own tables; see
/// `Platform::activate_kernel_tables`.
pub fn activate_kernel() {
    let Some(&level_4) = KERNEL_LEVEL_4.get() else {
        panic!("the kernel's page tables aren't taken yet");
    };
    // SAFETY: Every user space's upper half is a copy of the kernel tables'
    // level 4 entries, which never change after take.
    unsafe { load(level_4) };
}

/// Copies the loader's upper half into new tables made from `alloc`'s
/// frames, switches to them, turns on global pages, SMEP and SMAP, and
/// returns them. Returns `None` after the first call.
///
/// # Safety
///
/// `phys_offset` must be where the loader mapped all physical memory, and
/// the descriptor tables the CPU uses must be the kernel's own, not the
/// loader's, which it mapped in the lower half: `trap::init`, which loads
/// them, has also checked that the CPU has SMEP and SMAP.
pub unsafe fn take(
    phys_offset: VirtAddr,
    alloc: &mut dyn FnMut() -> Option<Frame>,
) -> Option<KernelPageTables> {
    static TAKEN: AtomicBool = AtomicBool::new(false);
    if TAKEN.swap(true, Ordering::Relaxed) {
        return None;
    }
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
    let tables = PageTables {
        phys_offset,
        level_4,
    };
    // Its level 3 table now, so that no upper-half level 4 entry changes
    // after this, and every user space's copy of them stays right.
    let Some(frame) = alloc() else {
        panic!("no frames left for the kernel's page tables");
    };
    let level_3 = tables.new_table(frame);
    // SAFETY: The new tables aren't in use yet.
    unsafe {
        tables
            .entry(level_4, space)
            .store(level_3 | PRESENT | WRITABLE);
    }
    // SAFETY: The new tables map every upper-half address to the same frame
    // with the same flags as the loader's, global bit aside, and the
    // kernel's code, stack, statics, boot information, framebuffer and
    // physical-memory mapping are all in the upper half, so everything the
    // kernel reaches stays where it was. The lower half held only the
    // loader's identity mappings, which the caller guarantees nothing uses.
    unsafe { load(level_4) };
    // Turning PGE off flushes global entries too, in case the loader made
    // any, so none of the loader's lower half survives.
    // SAFETY: Global pages only change what the TLB keeps.
    unsafe { Cr4::update(|flags| flags.remove(Cr4Flags::PAGE_GLOBAL)) };
    // SAFETY: Global pages only change what the TLB keeps, and the upper
    // half, the only one with global leaves, is the same in every space.
    // SMEP and SMAP only stop the kernel reaching user pages, of which
    // there are none yet; the kernel never reaches one but through user
    // copies, which will set RFLAGS.AC.
    unsafe {
        Cr4::update(|flags| {
            flags.insert(
                Cr4Flags::PAGE_GLOBAL
                    | Cr4Flags::SUPERVISOR_MODE_EXECUTION_PROTECTION
                    | Cr4Flags::SUPERVISOR_MODE_ACCESS_PREVENTION,
            );
        });
    }
    let heap_base = space_start(space);
    KERNEL_SPACE.init(kernel_space(space));
    KERNEL_LEVEL_4.init(level_4);
    Some(KernelPageTables {
        tables,
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

/// Where `map` maps in level 4 entry `space`: all of it above the heap.
fn kernel_space(space: PageTableIndex) -> Range<u64> {
    let start = space_start(space).as_u64();
    start + heap::SIZE..start + (1 << 39)
}

impl thisos_hal::PageTables for KernelPageTables {
    type AddressSpace = UserSpace;

    fn kernel_space(&self) -> Range<u64> {
        kernel_space(self.space)
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
        let write = if writable { WRITABLE } else { 0 };
        self.tables.map_page(
            Page::containing_address(VirtAddr::new(page)),
            frame,
            GLOBAL | NO_EXECUTE | write,
            WRITABLE,
            true,
            alloc,
        )
    }

    fn unmap(&mut self, page: u64) -> Option<Frame> {
        if page % 4096 != 0 || !self.kernel_space().contains(&page) {
            return None;
        }
        // SAFETY: Only map maps in kernel space, through map_page, and
        // kernel space's addresses are handed out once, so code still using
        // the page faults. Every space shares the upper half, so the TLB
        // may hold the page whichever is active.
        unsafe {
            self.tables
                .unmap_page(Page::containing_address(VirtAddr::new(page)), true)
        }
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
        self.tables.map_page(
            Page::containing_address(next),
            frame,
            GLOBAL | NO_EXECUTE | WRITABLE,
            WRITABLE,
            true,
            alloc,
        )?;
        // SAFETY: map_page mapped the page writable to the frame, and
        // nothing unmaps in the heap's range, maps there or reaches the
        // frame otherwise.
        unsafe { self.heap.grow() };
        Ok(())
    }

    fn translate(&self, addr: u64) -> Option<Translation> {
        self.tables.translate(addr)
    }

    fn lower_half_mapped(&self) -> bool {
        (0..UPPER_HALF)
            .map(PageTableIndex::new)
            .any(|i| self.tables.entry(self.tables.level_4, i).load() & PRESENT != 0)
    }

    fn new_space(&self, alloc: &mut dyn FnMut() -> Option<Frame>) -> Option<UserSpace> {
        let level_4 = self.tables.new_table(alloc()?);
        for index in (UPPER_HALF..512).map(PageTableIndex::new) {
            let value = self.tables.entry(self.tables.level_4, index).load();
            // SAFETY: The new table isn't in use yet.
            unsafe { self.tables.entry(level_4, index).store(value) };
        }
        Some(UserSpace {
            tables: PageTables {
                phys_offset: self.tables.phys_offset,
                level_4,
            },
            kernel: self.tables.level_4,
        })
    }

    fn activate(&self) {
        // SAFETY: Every user space's upper half is a copy of these tables'
        // level 4 entries, which never change after take.
        unsafe { load(self.tables.level_4) };
    }
}

/// A user address space. Only `KernelPageTables::new_space` makes one.
#[derive(Debug)]
pub struct UserSpace {
    tables: PageTables,
    /// The kernel's level 4 table, which `destroy` switches to.
    kernel: u64,
}

impl thisos_hal::AddressSpace for UserSpace {
    fn user_space(&self) -> Range<u64> {
        USER_SPACE
    }

    fn map(
        &mut self,
        page: u64,
        frame: Frame,
        protection: Protection,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError> {
        let fail = |kind, frame| Err(MapError { kind, frame });
        if page % 4096 != 0 {
            return fail(MapErrorKind::Unaligned, frame);
        }
        if !USER_SPACE.contains(&page) {
            return fail(MapErrorKind::OutsideUserSpace, frame);
        }
        let flags = match protection {
            Protection::ReadOnly => NO_EXECUTE,
            Protection::ReadWrite => NO_EXECUTE | WRITABLE,
            Protection::ReadExecute => 0,
        };
        // Tables let everything through; each leaf says what it allows.
        self.tables.map_page(
            Page::containing_address(VirtAddr::new(page)),
            frame,
            USER | flags,
            USER | WRITABLE,
            self.tables.active(),
            alloc,
        )
    }

    fn unmap(&mut self, page: u64) -> Option<Frame> {
        if page % 4096 != 0 || !USER_SPACE.contains(&page) {
            return None;
        }
        // SAFETY: Only map maps in the lower half, through map_page. The
        // kernel reaches user pages' frames only in copy_from_user, which
        // runs with interrupts off on the only CPU, so never while this
        // runs, and holds no reference to them afterwards. User pages
        // aren't global, so loading CR3 to leave this space flushed them,
        // and the TLB holds them only while it is active.
        unsafe {
            self.tables.unmap_page(
                Page::containing_address(VirtAddr::new(page)),
                self.tables.active(),
            )
        }
    }

    fn translate(&self, addr: u64) -> Option<Translation> {
        self.tables.translate(addr)
    }

    fn activate(&self) {
        // SAFETY: new_space copied the kernel's upper-half level 4 entries,
        // which never change after take.
        unsafe { load(self.tables.level_4) };
    }

    fn destroy(self, mapped: &mut dyn FnMut(Frame), free: &mut dyn FnMut(Frame)) {
        if self.tables.active() {
            // SAFETY: The kernel's tables have the same upper half as this
            // space.
            unsafe { load(self.kernel) };
        }
        // SAFETY: self is gone after this, CR3 doesn't point at its tables,
        // and every lower-half frame in them came from map_page or
        // new_table. The upper half is the kernel's, and is left alone.
        unsafe {
            self.tables
                .release(self.tables.level_4, 4, 0..UPPER_HALF, mapped, free);
        }
        // SAFETY: new_space took the level 4 frame with into_start, and
        // nothing uses the table any more.
        free(unsafe { Frame::reclaim(self.tables.level_4) });
    }
}

/// Where `addr` is mapped in the tables CR3 points at.
pub fn translate_active(phys_offset: VirtAddr, addr: u64) -> Option<Translation> {
    // Only reads entries, one atomic load each, so it can share the tables
    // with their owner.
    let (frame, _) = Cr3::read();
    PageTables {
        phys_offset,
        level_4: frame.start_address().as_u64(),
    }
    .translate(addr)
}

/// Writes `bytes` into `frame` from `offset` and zeroes the rest of it.
pub fn fill_frame(phys_offset: VirtAddr, frame: &mut Frame, offset: usize, bytes: &[u8]) {
    let Some(end) = offset.checked_add(bytes.len()).filter(|&end| end <= 4096) else {
        panic!(
            "{:#x} bytes at offset {offset:#x} don't fit in a frame",
            bytes.len()
        );
    };
    // A physical address, reached through the physical-memory mapping.
    let start = (phys_offset + frame.start()).as_mut_ptr::<u8>();
    // SAFETY: The caller holds the frame mutably, and page tables hold
    // every frame they map, so nothing maps it but the physical-memory
    // mapping, and no other reference to its 4 KiB exists.
    let page = unsafe { slice::from_raw_parts_mut(start, 4096) };
    page[..offset].fill(0);
    page[offset..end].copy_from_slice(bytes);
    page[end..].fill(0);
}

/// Copies the bytes at `addr` in the tables CR3 points at into `buf`; see
/// `Platform::copy_from_user`.
pub fn copy_from_user(phys_offset: VirtAddr, addr: u64, buf: &mut [u8]) -> bool {
    assert!(
        !interrupts::are_enabled(),
        "copying from user memory with interrupts on"
    );
    let (mut at, mut rest) = (addr, buf);
    while !rest.is_empty() {
        // Under a page, so it fits a usize.
        let room = usize::try_from(4096 - at % 4096).unwrap_or(4096);
        let len = room.min(rest.len());
        let (chunk, tail) = mem::take(&mut rest).split_at_mut(len);
        rest = tail;
        let Some(translation) = translate_active(phys_offset, at) else {
            return false;
        };
        if !translation.user || !USER_SPACE.contains(&at) {
            return false;
        }
        // A physical address, reached through the physical-memory mapping.
        let source = (phys_offset + translation.phys).as_ptr::<u8>();
        // SAFETY: chunk stays inside the page at, which is mapped for user
        // code, so its frame is RAM that the physical-memory mapping maps.
        // Interrupts are off on the only CPU, so no other code runs to
        // write the frame or unmap it meanwhile, and the kernel holds no
        // reference into user pages.
        unsafe { ptr::copy_nonoverlapping(source, chunk.as_mut_ptr(), len) };
        // USER_SPACE ends below the last page, so this doesn't overflow.
        at += u64::try_from(len).unwrap_or(4096);
    }
    true
}

/// Writes `ret` at the start of `frame`.
pub fn write_return(phys_offset: VirtAddr, frame: &Frame) {
    // A physical address, reached through the physical-memory mapping.
    let byte = (phys_offset + frame.start()).as_mut_ptr::<u8>();
    // SAFETY: The caller holds the frame, and page tables hold every frame
    // they map, so nothing maps it but the physical-memory mapping, and no
    // Rust reference to its memory exists.
    unsafe { byte.write_volatile(0xc3) };
}
