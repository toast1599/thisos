//! Page tables, reached through the loader's mapping of all physical
//! memory. Nothing here makes a reference to a table: the CPU's page walker
//! sets Accessed and Dirty bits in the tables at any time, so each access is
//! one atomic operation on one entry.

use core::marker::PhantomData;
use core::ops::{Deref, Range};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::{mem, ptr, slice};

use spin::Mutex;
use thisos_frames::Frame;
use thisos_hal::{MapError, MapErrorKind, Protection, Translation};
use thisos_regions::Regions;
use x86_64::VirtAddr;
use x86_64::registers::control::{Cr4, Cr4Flags};
use x86_64::structures::paging::{Page, PageTableIndex, Size4KiB};

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

/// The CPU state the tables depend on: CR3, the TLB and the interrupt flag.
/// Host tests swap in `tests::cpu`, a fake per test thread.
#[cfg(not(test))]
mod cpu {
    use x86_64::instructions::{interrupts, tlb};
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::PhysFrame;
    use x86_64::{PhysAddr, VirtAddr};

    /// The level 4 table CR3 points at.
    pub fn level_4() -> u64 {
        Cr3::read().0.start_address().as_u64()
    }

    /// Points CR3 at the level 4 table at `level_4`, keeping its flags.
    ///
    /// # Safety
    ///
    /// The table must map everything the kernel reaches as the tables CR3
    /// points at now do.
    pub unsafe fn set_level_4(level_4: u64) {
        let (_, flags) = Cr3::read();
        let frame = PhysFrame::containing_address(PhysAddr::new(level_4));
        // SAFETY: The caller guarantees the kernel's memory stays where it
        // is.
        unsafe { Cr3::write(frame, flags) };
    }

    pub fn flush(page: VirtAddr) {
        tlb::flush(page);
    }

    pub fn interrupts_enabled() -> bool {
        interrupts::are_enabled()
    }
}

#[cfg(test)]
use tests::cpu;

/// The page tables at `level_4`. Whoever makes one is the only user of the
/// tables while it lives, apart from the CPU's page walker, and apart from
/// the upper half of a user space's, which only the kernel's changes. A
/// `UserSpace` shares its tables between threads, and changes them only
/// under its lock.
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
        Self {
            phys_offset,
            level_4: cpu::level_4(),
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
        cpu::level_4() == self.level_4
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
    /// from the TLB if `flush`. The tables hold the frame until
    /// `unmap_page` reclaims it.
    fn map_page(
        &self,
        page: Page<Size4KiB>,
        frame: Frame,
        flags: u64,
        table_flags: u64,
        flush: bool,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError> {
        // SAFETY: The frame is the caller's to give, and on success the
        // entry holds it: into_start below gives up the Frame, and only
        // unmap_page or release makes it again.
        match unsafe { self.map_leaf(page, frame.start(), flags, table_flags, flush, alloc) } {
            Ok(()) => {
                let _ = frame.into_start();
                Ok(())
            }
            Err(kind) => Err(MapError { kind, given: frame }),
        }
    }

    /// Maps `page` to the frame at `phys` with leaf bits `flags`, and links
    /// any new table on the way with `table_flags`, from `alloc`. Flushes
    /// the page from the TLB if `flush`.
    ///
    /// # Safety
    ///
    /// The frame must stay allocated while the page maps it, and the
    /// kernel must hold no reference into it that writes through the page
    /// could break.
    unsafe fn map_leaf(
        &self,
        page: Page<Size4KiB>,
        phys: u64,
        flags: u64,
        table_flags: u64,
        flush: bool,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapErrorKind> {
        let mut table = self.level_4;
        for index in [page.p4_index(), page.p3_index(), page.p2_index()] {
            let entry = self.entry(table, index);
            let mut value = entry.load();
            if value & PRESENT == 0 {
                let Some(new) = alloc() else {
                    return Err(MapErrorKind::NoFrames);
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
            return Err(MapErrorKind::Mapped);
        }
        // SAFETY: The entry isn't present, so nothing is reached through it
        // yet, and the caller keeps the frame for it.
        unsafe { leaf.store(phys | PRESENT | flags) };
        if flush {
            cpu::flush(page.start_address());
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
        // SAFETY: As for this call.
        let phys = unsafe { self.clear_leaf(page, flush) }?;
        // SAFETY: map_page stored into_start of the frame there, and the
        // entry, now clear, was its only holder.
        Some(unsafe { Frame::reclaim(phys) })
    }

    /// Unmaps `page` and returns the address of the frame it mapped, or
    /// `None` if it isn't mapped as a 4 KiB page. Flushes the page from the
    /// TLB if `flush`.
    ///
    /// # Safety
    ///
    /// Nothing may reach other memory through the page once it is gone,
    /// and `flush` must be set if the TLB may hold the page.
    unsafe fn clear_leaf(&self, page: Page<Size4KiB>, flush: bool) -> Option<u64> {
        let leaf = self.leaf(page)?;
        // SAFETY: The caller guarantees nothing reaches other memory through
        // the page once it is gone.
        let value = unsafe { leaf.clear() };
        if flush {
            cpu::flush(page.start_address());
        }
        Some(value & ADDRESS)
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

    /// Calls `free` with each table under entries `indices` of the level
    /// `level` table at `table`. The frames that level 1 entries map are
    /// left alone: they are a user space's, whose mappings' objects hold
    /// them.
    ///
    /// # Safety
    ///
    /// Nothing may use the tables or the pages they map any more, and every
    /// table in them must have come from `new_table`.
    unsafe fn release(
        &self,
        table: u64,
        level: u8,
        indices: Range<u16>,
        free: &mut dyn FnMut(Frame),
    ) {
        if level == 1 {
            return;
        }
        for index in indices.map(PageTableIndex::new) {
            let value = self.entry(table, index).load();
            if value & PRESENT == 0 {
                continue;
            }
            let frame = value & ADDRESS;
            // SAFETY: As for this call.
            unsafe { self.release(frame, level - 1, 0..512, free) };
            // SAFETY: new_table took the frame with into_start, and nothing
            // uses the table any more.
            free(unsafe { Frame::reclaim(frame) });
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
    if cpu::level_4() == level_4 {
        return;
    }
    // SAFETY: The caller guarantees the kernel's memory stays where it is.
    unsafe { cpu::set_level_4(level_4) };
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
    type AddressSpace<O: Deref<Target = [Frame]> + Send + Sync> = UserSpace<O>;

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
        let fail = |kind, given| Err(MapError { kind, given });
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

    fn new_space<O: Deref<Target = [Frame]> + Send + Sync>(
        &self,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Option<UserSpace<O>> {
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
            mappings: Mutex::new(Regions::new()),
        })
    }

    fn activate(&self) {
        // SAFETY: Every user space's upper half is a copy of these tables'
        // level 4 entries, which never change after take.
        unsafe { load(self.tables.level_4) };
    }
}

/// A user address space. Only `KernelPageTables::new_space` makes one.
pub struct UserSpace<O> {
    tables: PageTables,
    /// The kernel's level 4 table, which `destroy` switches to.
    kernel: u64,
    /// Every page mapped in the lower half is in one of these, and maps a
    /// frame of its object. Map and unmap change the tables only while
    /// they hold this lock.
    mappings: Mutex<Regions<O>>,
}

impl<O: Deref<Target = [Frame]> + Send + Sync> thisos_hal::AddressSpace<O> for UserSpace<O> {
    fn user_space(&self) -> Range<u64> {
        USER_SPACE
    }

    fn map(
        &self,
        addr: u64,
        object: O,
        first: usize,
        pages: usize,
        protection: Protection,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError<O>> {
        let fail = |kind, given| Err(MapError { kind, given });
        if addr % 4096 != 0 {
            return fail(MapErrorKind::Unaligned, object);
        }
        if !USER_SPACE.contains(&addr) {
            return fail(MapErrorKind::OutsideUserSpace, object);
        }
        let end = u64::try_from(pages)
            .ok()
            .and_then(|pages| pages.checked_mul(4096))
            .and_then(|len| addr.checked_add(len))
            .filter(|&end| end <= USER_SPACE.end);
        let in_object = first
            .checked_add(pages)
            .is_some_and(|last| last <= object.len());
        let Some(end) = end.filter(|_| pages > 0 && in_object) else {
            return fail(MapErrorKind::OutOfRange, object);
        };
        let flags = match protection {
            Protection::ReadOnly => NO_EXECUTE,
            Protection::ReadWrite => NO_EXECUTE | WRITABLE,
            Protection::ReadExecute => 0,
        };
        let flush = self.tables.active();
        let mut mappings = self.mappings.lock();
        let object = match mappings.insert(addr..end, object) {
            Ok(object) => object,
            Err(object) => return fail(MapErrorKind::Mapped, object),
        };
        let mut done = addr;
        let mapped = object[first..first + pages].iter().try_for_each(|frame| {
            let page = Page::containing_address(VirtAddr::new(done));
            // Tables let everything through; each leaf says what it allows.
            // SAFETY: The mapping now holds the object, which holds the
            // frame, until unmap or destroy clears the page. The kernel
            // reaches user pages' frames only in copy_from_user, which
            // holds no reference afterwards.
            unsafe {
                self.tables.map_leaf(
                    page,
                    frame.start(),
                    USER | flags,
                    USER | WRITABLE,
                    flush,
                    alloc,
                )
            }?;
            done += 4096;
            Ok(())
        });
        let Err(kind) = mapped else {
            return Ok(());
        };
        for page in (addr..done).step_by(4096) {
            // SAFETY: This call mapped the page a moment ago, and returns no
            // address in it, so nothing reaches it.
            unsafe {
                self.tables
                    .clear_leaf(Page::containing_address(VirtAddr::new(page)), flush)
            };
        }
        let Some((_, object)) = mappings.remove(addr) else {
            panic!("the mapping at {addr:#x} went missing");
        };
        fail(kind, object)
    }

    fn unmap(&self, addr: u64) -> Option<O> {
        let mut mappings = self.mappings.lock();
        let (range, object) = mappings.remove(addr)?;
        let flush = self.tables.active();
        for page in range.step_by(4096) {
            // SAFETY: Only map maps in the lower half, every page of the
            // mapping, and the frames stay the object's, so nothing but
            // user code reaches memory through them, and the object goes
            // back only after the pages are gone. User pages aren't global,
            // so loading CR3 to leave this space flushed them, and the TLB
            // holds them only while it is active.
            let cleared = unsafe {
                self.tables
                    .clear_leaf(Page::containing_address(VirtAddr::new(page)), flush)
            };
            assert!(
                cleared.is_some(),
                "mapped page {page:#x} wasn't in the tables"
            );
        }
        Some(object)
    }

    fn translate(&self, addr: u64) -> Option<Translation> {
        self.tables.translate(addr)
    }

    fn activate(&self) {
        // SAFETY: new_space copied the kernel's upper-half level 4 entries,
        // which never change after take.
        unsafe { load(self.tables.level_4) };
    }

    fn destroy(self, unmapped: &mut dyn FnMut(O), free: &mut dyn FnMut(Frame)) {
        if self.tables.active() {
            // SAFETY: The kernel's tables have the same upper half as this
            // space.
            unsafe { load(self.kernel) };
        }
        // SAFETY: self is gone after this, CR3 doesn't point at its tables,
        // and every lower-half table in them came from new_table. The upper
        // half is the kernel's, and is left alone.
        unsafe {
            self.tables
                .release(self.tables.level_4, 4, 0..UPPER_HALF, free);
        }
        // SAFETY: new_space took the level 4 frame with into_start, and
        // nothing uses the table any more.
        free(unsafe { Frame::reclaim(self.tables.level_4) });
        // No table maps the objects' frames any more.
        for (_, object) in self.mappings.into_inner().take_all() {
            unmapped(object);
        }
    }
}

/// Where `addr` is mapped in the tables CR3 points at.
pub fn translate_active(phys_offset: VirtAddr, addr: u64) -> Option<Translation> {
    // Only reads entries, one atomic load each, so it can share the tables
    // with their owner.
    PageTables {
        phys_offset,
        level_4: cpu::level_4(),
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
    // SAFETY: The caller holds the frame mutably. Kernel page tables hold
    // the frames they map, and a user space holds an object that shares
    // the frames it maps, so neither maps this one: nothing maps it but the
    // physical-memory mapping, and no other reference to its 4 KiB exists.
    let page = unsafe { slice::from_raw_parts_mut(start, 4096) };
    page[..offset].fill(0);
    page[offset..end].copy_from_slice(bytes);
    page[end..].fill(0);
}

/// Copies the bytes at `addr` in the tables CR3 points at into `buf`; see
/// `Platform::copy_from_user`.
pub fn copy_from_user(phys_offset: VirtAddr, addr: u64, buf: &mut [u8]) -> bool {
    assert!(
        !cpu::interrupts_enabled(),
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
pub fn write_return(phys_offset: VirtAddr, frame: &mut Frame) {
    // A physical address, reached through the physical-memory mapping.
    let byte = (phys_offset + frame.start()).as_mut_ptr::<u8>();
    // SAFETY: The caller holds the frame mutably, so, as in fill_frame,
    // nothing maps it but the physical-memory mapping, and no Rust
    // reference to its memory exists.
    unsafe { byte.write_volatile(0xc3) };
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::vec::Vec;
    use std::{thread, vec};

    use thisos_frames::{FLOOR, Frames};
    use thisos_hal::{AddressSpace as _, PageTables as _};

    use super::*;

    /// The CPU as the tables see it, faked per test thread: CR3 starts at
    /// 0, which no table is at, the TLB is a list of flushed pages, and
    /// interrupts are always off.
    pub mod cpu {
        use core::cell::{Cell, RefCell};
        use std::vec::Vec;

        use x86_64::VirtAddr;

        std::thread_local! {
            static LEVEL_4: Cell<u64> = const { Cell::new(0) };
            static FLUSHED: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
        }

        pub fn level_4() -> u64 {
            LEVEL_4.get()
        }

        /// # Safety
        ///
        /// None; it is unsafe as the real one is.
        pub unsafe fn set_level_4(level_4: u64) {
            LEVEL_4.set(level_4);
        }

        pub fn flush(page: VirtAddr) {
            FLUSHED.with_borrow_mut(|flushed| flushed.push(page.as_u64()));
        }

        /// The pages flushed since the last call.
        pub fn flushed() -> Vec<u64> {
            FLUSHED.take()
        }

        pub fn interrupts_enabled() -> bool {
            false
        }
    }

    const ACCESSED: u64 = 1 << 5;
    const DIRTY: u64 = 1 << 6;

    /// Physical memory ends here; the frames from `FLOOR` up are free.
    const END: u64 = 0x14_0000;
    const PAGES: usize = 0x140;
    const _: () = assert!(END == 4096 * 0x140);

    #[derive(Clone)]
    #[repr(C, align(4096))]
    struct Phys([u8; 4096]);

    /// A buffer standing in for physical memory from 0 to `END`, and the
    /// frame allocator over it.
    struct Memory {
        frames: Frames<5>,
        phys_offset: VirtAddr,
        _buffer: Vec<Phys>,
    }

    impl Memory {
        fn new() -> Self {
            let mut buffer = vec![Phys([0; 4096]); PAGES];
            // The tables reach the buffer only through this address, as the
            // kernel reaches RAM through the physical-memory mapping, so the
            // buffer's provenance has to be exposed.
            let start = buffer.as_mut_ptr().expose_provenance();
            let Ok(start) = u64::try_from(start) else {
                panic!("a host address wider than 64 bits");
            };
            let mut frames = Frames::new();
            assert_eq!(frames.add(FLOOR..END), 0);
            Self {
                frames,
                phys_offset: VirtAddr::new(start),
                _buffer: buffer,
            }
        }

        fn frame(&mut self) -> Frame {
            let Some(frame) = self.frames.alloc() else {
                panic!("the test's physical memory is used up");
            };
            frame
        }

        fn object(&mut self, pages: usize) -> Object {
            Object(Arc::new((0..pages).map(|_| self.frame()).collect()))
        }

        fn free_object(&mut self, object: Object) {
            let Some(frames) = Arc::into_inner(object.0) else {
                panic!("an object is still shared");
            };
            for frame in frames {
                self.frames.free(frame);
            }
        }
    }

    /// A memory object as the kernel's `Mapping` is: frames that mappings
    /// share.
    #[derive(Clone)]
    struct Object(Arc<Vec<Frame>>);

    impl Deref for Object {
        type Target = [Frame];

        fn deref(&self) -> &[Frame] {
            &self.0
        }
    }

    /// Kernel page tables as `take` leaves them, without the loader's
    /// upper half: the reserved level 4 entry and its level 3 table.
    fn kernel(memory: &mut Memory) -> KernelPageTables {
        let space = PageTableIndex::new(300);
        let mut tables = PageTables {
            phys_offset: memory.phys_offset,
            level_4: 0,
        };
        tables.level_4 = tables.new_table(memory.frame());
        let level_3 = tables.new_table(memory.frame());
        // SAFETY: The tables aren't in use.
        unsafe {
            tables
                .entry(tables.level_4, space)
                .store(level_3 | PRESENT | WRITABLE);
        }
        KernelPageTables {
            tables,
            space,
            // SAFETY: Nothing is mapped in the space entry, and the test
            // never grows the heap.
            heap: unsafe { Heap::new(space_start(space)) },
        }
    }

    fn new_space(kernel: &KernelPageTables, memory: &mut Memory) -> UserSpace<Object> {
        let Some(space) = kernel.new_space(&mut || memory.frames.alloc()) else {
            panic!("no frame for a level 4 table");
        };
        space
    }

    /// Destroys `space`, frees every frame it held, and checks that the
    /// frame count is back to `free`.
    fn destroy(space: UserSpace<Object>, memory: &mut Memory, free: usize) {
        let mut objects = Vec::new();
        space.destroy(&mut |object| objects.push(object), &mut |frame| {
            memory.frames.free(frame);
        });
        for object in objects {
            memory.free_object(object);
        }
        assert_eq!(memory.frames.free_frames(), free);
    }

    fn page(addr: u64) -> Page {
        Page::containing_address(VirtAddr::new(addr))
    }

    #[test]
    fn user_mappings_translate_and_unmap() {
        let mut memory = Memory::new();
        let kernel = kernel(&mut memory);
        let free = memory.frames.free_frames();
        let space = new_space(&kernel, &mut memory);
        let data = memory.object(3);
        let code = memory.object(1);
        let (data_1, code_0) = (data[1].start(), code[0].start());
        let mut alloc = || memory.frames.alloc();
        let rw = space.map(
            0x40_0000,
            data.clone(),
            0,
            3,
            Protection::ReadWrite,
            &mut alloc,
        );
        let rx = space.map(0x40_3000, code, 0, 1, Protection::ReadExecute, &mut alloc);
        let ro = space.map(0x80_0000, data, 1, 2, Protection::ReadOnly, &mut alloc);
        assert!(rw.is_ok() && rx.is_ok() && ro.is_ok());
        let translation = |phys, writable, executable| {
            Some(Translation {
                phys,
                writable,
                executable,
                user: true,
            })
        };
        assert_eq!(
            space.translate(0x40_1234),
            translation(data_1 + 0x234, true, false)
        );
        assert_eq!(
            space.translate(0x40_3008),
            translation(code_0 + 8, false, true)
        );
        assert_eq!(
            space.translate(0x80_0010),
            translation(data_1 + 0x10, false, false)
        );
        assert_eq!(space.translate(0x40_4000), None);

        let other = memory.object(1);
        let Err(error) = space.map(0x40_2000, other, 0, 1, Protection::ReadOnly, &mut || {
            memory.frames.alloc()
        }) else {
            panic!("an overlapping mapping was made");
        };
        assert_eq!(error.kind, MapErrorKind::Mapped);
        memory.free_object(error.given);

        let Some(data) = space.unmap(0x40_0000) else {
            panic!("the mapping at 0x40_0000 is gone");
        };
        assert_eq!(space.translate(0x40_1234), None);
        assert!(space.unmap(0x40_0000).is_none());
        // The read-only mapping still shares the frames.
        assert_eq!(
            space.translate(0x80_0010),
            translation(data_1 + 0x10, false, false)
        );
        drop(data);
        assert!(
            cpu::flushed().is_empty(),
            "flushed an inactive space's pages"
        );
        destroy(space, &mut memory, free);
    }

    #[test]
    fn failed_map_takes_its_pages_back() {
        let mut memory = Memory::new();
        let kernel = kernel(&mut memory);
        let free = memory.frames.free_frames();
        let space = new_space(&kernel, &mut memory);
        let object = memory.object(3);
        // The third page needs a second level 1 table, and the frames run
        // out after the first one.
        let mut budget = 3;
        let mut alloc = || {
            budget -= 1;
            (budget >= 0).then(|| memory.frames.alloc()).flatten()
        };
        let Err(error) = space.map(0x1f_e000, object, 0, 3, Protection::ReadWrite, &mut alloc)
        else {
            panic!("mapped with too few frames");
        };
        assert_eq!(error.kind, MapErrorKind::NoFrames);
        assert_eq!(space.translate(0x1f_e000), None);
        assert_eq!(space.translate(0x1f_f000), None);
        // The range is free again.
        let mapped = space.map(
            0x1f_e000,
            error.given,
            0,
            3,
            Protection::ReadWrite,
            &mut || memory.frames.alloc(),
        );
        assert!(mapped.is_ok());
        destroy(space, &mut memory, free);
    }

    #[test]
    fn only_the_active_space_flushes() {
        let mut memory = Memory::new();
        let kernel = kernel(&mut memory);
        let free = memory.frames.free_frames();
        let space = new_space(&kernel, &mut memory);
        let object = memory.object(2);
        space.activate();
        let mapped = space.map(0x40_0000, object, 0, 2, Protection::ReadOnly, &mut || {
            memory.frames.alloc()
        });
        assert!(mapped.is_ok());
        assert_eq!(cpu::flushed(), [0x40_0000, 0x40_1000]);
        let Some(object) = space.unmap(0x40_0000) else {
            panic!("the mapping is gone");
        };
        assert_eq!(cpu::flushed(), [0x40_0000, 0x40_1000]);

        kernel.activate();
        let mapped = space.map(0x40_0000, object, 0, 2, Protection::ReadOnly, &mut || {
            memory.frames.alloc()
        });
        assert!(mapped.is_ok());
        assert!(space.unmap(0x40_0000).is_some_and(|object| {
            memory.free_object(object);
            true
        }));
        assert_eq!(cpu::flushed(), Vec::<u64>::new());

        // Destroying the active space leaves the kernel's tables active.
        space.activate();
        destroy(space, &mut memory, free);
        assert_eq!(cpu::level_4(), kernel.tables.level_4);
    }

    #[test]
    fn kernel_space_maps_in_every_space() {
        let mut memory = Memory::new();
        let mut kernel = kernel(&mut memory);
        let space = new_space(&kernel, &mut memory);
        let start = kernel.kernel_space().start;
        for (page, kind) in [
            (start + 8, MapErrorKind::Unaligned),
            (start - 4096, MapErrorKind::OutsideKernelSpace),
        ] {
            let frame = memory.frame();
            let Err(error) = kernel.map(page, frame, true, &mut || memory.frames.alloc()) else {
                panic!("mapped {page:#x}");
            };
            assert_eq!(error.kind, kind);
            memory.frames.free(error.given);
        }

        let frame = memory.frame();
        let phys = frame.start();
        let mapped = kernel.map(start, frame, true, &mut || memory.frames.alloc());
        assert!(mapped.is_ok());
        assert_eq!(cpu::flushed(), [start]);
        let expected = Some(Translation {
            phys: phys + 0x10,
            writable: true,
            executable: false,
            user: false,
        });
        assert_eq!(kernel.translate(start + 0x10), expected);
        // The space was made before the page was mapped, and shares the
        // level 3 table under the reserved entry.
        assert_eq!(space.translate(start + 0x10), expected);
        assert!(!kernel.lower_half_mapped());

        let Some(frame) = kernel.unmap(start) else {
            panic!("the page at {start:#x} is gone");
        };
        assert_eq!(frame.start(), phys);
        assert!(kernel.unmap(start).is_none());
        assert_eq!(space.translate(start + 0x10), None);
        memory.frames.free(frame);
    }

    #[test]
    fn user_copies_cross_pages() {
        let mut memory = Memory::new();
        let kernel = kernel(&mut memory);
        let space = new_space(&kernel, &mut memory);
        let phys_offset = memory.phys_offset;
        let mut frames: Vec<Frame> = (0..3).map(|_| memory.frame()).collect();
        fill_frame(phys_offset, &mut frames[0], 4093, b"abc");
        fill_frame(phys_offset, &mut frames[1], 0, b"def");
        write_return(phys_offset, &mut frames[2]);
        let object = Object(Arc::new(frames));
        let mapped = space.map(0x40_0000, object, 0, 3, Protection::ReadWrite, &mut || {
            memory.frames.alloc()
        });
        assert!(mapped.is_ok());
        space.activate();

        let mut buf = [0; 6];
        assert!(copy_from_user(phys_offset, 0x40_0ffd, &mut buf));
        assert_eq!(&buf, b"abcdef");
        let mut byte = [0];
        assert!(copy_from_user(phys_offset, 0x40_2000, &mut byte));
        assert_eq!(byte, [0xc3]);
        // The last two bytes are past the mapping.
        assert!(!copy_from_user(phys_offset, 0x40_2ffe, &mut [0; 4]));
        // Kernel pages aren't user pages.
        let start = kernel.kernel_space().start;
        assert!(!copy_from_user(phys_offset, start, &mut byte));
    }

    #[test]
    fn copied_tables_keep_large_pages_and_go_global() {
        let mut memory = Memory::new();
        let mut loader = PageTables {
            phys_offset: memory.phys_offset,
            level_4: 0,
        };
        loader.level_4 = loader.new_table(memory.frame());
        let level_3 = loader.new_table(memory.frame());
        let level_2 = loader.new_table(memory.frame());
        let level_1 = loader.new_table(memory.frame());
        let index = PageTableIndex::new;
        let entries = [
            (loader.level_4, 256, level_3 | PRESENT | WRITABLE),
            (level_3, 0, level_2 | PRESENT | WRITABLE),
            // 1 GiB and 2 MiB pages, and a table of 4 KiB ones.
            (level_3, 1, 0x8000_0000 | PRESENT | PS_OR_PAT | NO_EXECUTE),
            (level_2, 0, 0x4000_0000 | PRESENT | WRITABLE | PS_OR_PAT),
            (level_2, 1, level_1 | PRESENT | WRITABLE),
            (level_1, 3, 0x1234_5000 | PRESENT),
        ];
        for (table, i, value) in entries {
            // SAFETY: The tables aren't in use.
            unsafe { loader.entry(table, index(i)).store(value) };
        }

        let level_4 = loader.copy(loader.level_4, 4, UPPER_HALF..512, &mut || {
            memory.frames.alloc()
        });
        let copy = PageTables {
            phys_offset: memory.phys_offset,
            level_4,
        };
        let base = space_start(index(256)).as_u64();
        let at = |phys, writable, executable| {
            Some(Translation {
                phys,
                writable,
                executable,
                user: false,
            })
        };
        assert_eq!(
            copy.translate(base + 0x4000_0123),
            at(0x8000_0123, false, false)
        );
        assert_eq!(
            copy.translate(base + 0x1f_ffff),
            at(0x401f_ffff, true, true)
        );
        assert_eq!(
            copy.translate(base + 0x20_3456),
            at(0x1234_5456, false, true)
        );
        assert_eq!(copy.translate(base + 0x20_4000), None);
        let Some(leaf) = copy.leaf(page(base + 0x20_3000)) else {
            panic!("the 4 KiB page wasn't copied");
        };
        assert_eq!(leaf.load(), 0x1234_5000 | PRESENT | GLOBAL);
        assert_ne!(copy.frame(page(base + 0x20_3000)), loader.frame(page(base)));
        // The copy has tables of its own.
        assert_ne!(
            copy.entry(level_4, index(256)).load(),
            loader.entry(loader.level_4, index(256)).load()
        );
    }

    impl Entry<'_> {
        /// Sets `bits` if the entry is present, in one atomic step, as the
        /// CPU's page walker sets Accessed and Dirty.
        fn walk(self, bits: u64) {
            // SAFETY: As in load.
            let entry = unsafe { AtomicU64::from_ptr(self.0) };
            let _ = entry.try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                (value & PRESENT != 0).then_some(value | bits)
            });
        }
    }

    #[test]
    fn walker_bits_survive_updates_and_unmaps() {
        let mut memory = Memory::new();
        let kernel = kernel(&mut memory);
        let free = memory.frames.free_frames();
        let space = new_space(&kernel, &mut memory);
        for addr in [0x40_0000, 0x40_4000] {
            let object = memory.object(4);
            let mapped = space.map(addr, object, 0, 4, Protection::ReadWrite, &mut || {
                memory.frames.alloc()
            });
            assert!(mapped.is_ok());
        }
        space.activate();
        let tables = PageTables {
            phys_offset: space.tables.phys_offset,
            level_4: space.tables.level_4,
        };
        let walker = thread::spawn(move || {
            for addr in (0x40_0000..0x40_8000).step_by(4096) {
                if let Some(leaf) = tables.leaf(page(addr)) {
                    leaf.walk(ACCESSED | DIRTY);
                }
            }
        });
        for addr in (0x40_0000..0x40_4000).step_by(4096) {
            let Some(leaf) = space.tables.leaf(page(addr)) else {
                panic!("{addr:#x} isn't mapped");
            };
            // SAFETY: Nothing reaches the page's memory.
            unsafe { leaf.update(|value| value & !WRITABLE) };
        }
        let Some(object) = space.unmap(0x40_4000) else {
            panic!("the mapping at 0x40_4000 is gone");
        };
        assert!(walker.join().is_ok());
        memory.free_object(object);

        for addr in (0x40_0000..0x40_4000).step_by(4096) {
            let Some(leaf) = space.tables.leaf(page(addr)) else {
                panic!("{addr:#x} isn't mapped");
            };
            let value = leaf.load();
            assert_eq!(value & (WRITABLE | ACCESSED | DIRTY), ACCESSED | DIRTY);
        }
        assert_eq!(space.translate(0x40_4000), None);
        destroy(space, &mut memory, free);
    }
}
