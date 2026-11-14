//! Page tables, reached through Limine's mapping of all physical memory.
//! The kernel's tables are TTBR1's, the upper half every address space
//! shares; a user space is a level 0 table of its own in TTBR0. Nothing
//! here makes a reference to a table: each access is one atomic operation
//! on one entry, as on x86_64. The walker writes no entries today, since
//! `TCR_EL1.HA` and HD stay off, but it would if they were ever turned on.

use core::arch::asm;
use core::marker::PhantomData;
use core::ops::{Deref, Range};
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use aarch64_cpu::asm::barrier::{self, ISH, ISHST};
use aarch64_cpu::registers::{ID_AA64MMFR1_EL1, MAIR_EL1, Readable, TCR_EL1, TTBR0_EL1, TTBR1_EL1};
use spin::Mutex;
use thisos_frames::Frame;
use thisos_hal::{MapError, MapErrorKind, Protection, Translation};
use thisos_regions::Regions;

use crate::heap::{self, Heap};
use crate::ptr_at;

// Descriptor bits, for the 4 KiB granule.
const VALID: u64 = 1 << 0;
/// In a level 0 to 2 entry, a table rather than a block; in a level 3
/// entry it must be set.
const TABLE_OR_PAGE: u64 = 1 << 1;
const ATTR_INDEX: u64 = 7 << 2;
/// Attribute index 2, which Limine leaves unused and the kernel gives the
/// framebuffer.
const ATTR_NON_CACHEABLE: u64 = 2 << 2;
/// Attribute index 3, which Limine leaves unused and the kernel gives
/// device registers.
const ATTR_DEVICE: u64 = 3 << 2;
/// AP[1]: EL0 may reach the page.
const EL0: u64 = 1 << 6;
/// AP[2]: nothing may write the page.
const READ_ONLY: u64 = 1 << 7;
const INNER_SHAREABLE: u64 = 3 << 8;
/// The Access flag. Without it the first access faults, since the walker
/// doesn't set it with `TCR_EL1.HA` off.
const ACCESSED: u64 = 1 << 10;
const NOT_GLOBAL: u64 = 1 << 11;
/// EL1 may not execute the page.
const PXN: u64 = 1 << 53;
/// EL0 may not execute the page.
const UXN: u64 = 1 << 54;
const ADDRESS: u64 = 0x0000_ffff_ffff_f000;

/// The bits every leaf the kernel makes has: valid, normal write-back
/// memory (attribute index 0), inner shareable, accessed.
const LEAF: u64 = VALID | TABLE_OR_PAGE | INNER_SHAREABLE | ACCESSED;
const TABLE: u64 = VALID | TABLE_OR_PAGE;

/// The first address TTBR1 translates: T1SZ is 16, as Limine sets it.
const UPPER_HALF: u64 = 0xffff_0000_0000_0000;

/// Where user mappings may go: TTBR0's 256 TiB, above the first 64 KiB.
pub const USER_SPACE: Range<u64> = 0x1_0000..1 << 48;

/// `MAIR_EL1`'s attribute 2 as normal memory, non-cacheable inner and outer,
/// so that the CPU gathers stores to it as x86_64's write-combining does.
const MAIR_NON_CACHEABLE: u64 = 0x44;
/// `MAIR_EL1`'s attribute 3 as Device-nGnRE: no gathering or reordering,
/// with early write acknowledgement, as Linux maps device registers.
const MAIR_DEVICE: u64 = 0x04;

/// The page tables at `level_0`. Whoever makes one is the only user of the
/// tables while it lives, apart from the CPU's page walker.
#[derive(Debug)]
pub struct PageTables {
    phys_offset: u64,
    level_0: u64,
}

/// The table indices of `addr` at levels 0 to 3.
fn indices(addr: u64) -> [u64; 4] {
    [39, 30, 21, 12].map(|shift| (addr >> shift) & 511)
}

/// The bytes one entry of a level `level` table maps.
fn entry_size(level: u8) -> u64 {
    1 << (39 - 9 * u32::from(level))
}

impl PageTables {
    /// Entry `index` of the table at physical address `table`.
    fn entry(&self, table: u64, index: u64) -> Entry<'_> {
        // The table's address comes from a TTBR or a table entry, and
        // reaches memory through the physical-memory mapping.
        Entry(ptr_at(self.phys_offset + table + 8 * index), PhantomData)
    }

    /// The level 3 entry that maps `page`, when it is mapped as a 4 KiB
    /// page.
    fn leaf(&self, page: u64) -> Option<Entry<'_>> {
        let [i0, i1, i2, i3] = indices(page);
        let mut table = self.level_0;
        for index in [i0, i1, i2] {
            let value = self.entry(table, index).load();
            if value & VALID == 0 || value & TABLE_OR_PAGE == 0 {
                return None;
            }
            table = value & ADDRESS;
        }
        let leaf = self.entry(table, i3);
        (leaf.load() & VALID != 0).then_some(leaf)
    }

    /// Makes `frame` an empty table, and returns its physical address. The
    /// frame belongs to the tables from here on.
    fn new_table(&self, frame: Frame) -> u64 {
        let start = frame.into_start();
        // A physical address, reached through the physical-memory mapping.
        let table = ptr_at::<u64>(self.phys_offset + start);
        // SAFETY: The allocator handed over the frame, so nothing else
        // reads or writes it, and it isn't in any table yet, so the CPU's
        // walker doesn't either. Its 512 entries are its 4 KiB, which the
        // physical-memory mapping maps.
        unsafe { ptr::write_bytes(table, 0, 512) };
        start
    }

    /// A new empty table for the kernel's tables, from `alloc`. Panics if
    /// it has no frames left, since the kernel can't run without them.
    fn kernel_table(&self, alloc: &mut dyn FnMut() -> Option<Frame>) -> u64 {
        let Some(frame) = alloc() else {
            panic!("no frames left for the kernel's page tables");
        };
        self.new_table(frame)
    }

    /// Copies the valid entries of the level `level` table at `from`, which
    /// maps from `va`, into a new table, and returns the new table's
    /// address. Each entry becomes what `copy_entry` makes of it.
    fn copy(
        &self,
        from: u64,
        level: u8,
        va: u64,
        leaf: &dyn Fn(u64, u64, u64) -> u64,
        split: &dyn Fn(u64, u64) -> bool,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> u64 {
        let to = self.kernel_table(alloc);
        for index in 0..512 {
            let value = self.entry(from, index).load();
            if value & VALID == 0 {
                continue;
            }
            let address = va + index * entry_size(level);
            let value = self.copy_entry(value, level, address, leaf, split, alloc);
            // SAFETY: The new table isn't in use yet: no TTBR points at it.
            unsafe { self.entry(to, index).store(value) };
        }
        to
    }

    /// What the valid entry `value` of a level `level` table, which maps
    /// from `va`, becomes in a copy. A table is copied in turn. A block for
    /// which `split` gives true, given its address and size, becomes a
    /// table of smaller entries that map the same memory. Any other leaf
    /// becomes what `leaf` makes of it, given its address, size and value.
    fn copy_entry(
        &self,
        value: u64,
        level: u8,
        va: u64,
        leaf: &dyn Fn(u64, u64, u64) -> u64,
        split: &dyn Fn(u64, u64) -> bool,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> u64 {
        let size = entry_size(level);
        if !is_leaf(level, value) {
            let table = self.copy(value & ADDRESS, level + 1, va, leaf, split, alloc);
            return (value & !ADDRESS) | table;
        }
        if level == 3 || !split(va, size) {
            return leaf(va, size, value);
        }
        let to = self.kernel_table(alloc);
        let child = entry_size(level + 1);
        let page = if level + 1 == 3 { TABLE_OR_PAGE } else { 0 };
        for index in 0..512 {
            let offset = index * child;
            let value = (value & !ADDRESS & !TABLE_OR_PAGE) | ((value & ADDRESS) + offset) | page;
            let value = self.copy_entry(value, level + 1, va + offset, leaf, split, alloc);
            // SAFETY: The new table isn't in use yet: no TTBR points at it.
            unsafe { self.entry(to, index).store(value) };
        }
        to | TABLE
    }

    /// Maps `page` to `frame` with leaf bits `flags`, and links any new
    /// table on the way, from `alloc`.
    fn map_page(
        &self,
        page: u64,
        frame: Frame,
        flags: u64,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError> {
        // SAFETY: The frame is the caller's to give, and on success the
        // leaf holds it until unmap reclaims it.
        match unsafe { self.map_address(page, frame.start(), flags, alloc) } {
            Ok(()) => {
                let _ = frame.into_start();
                Ok(())
            }
            Err(kind) => Err(MapError { kind, given: frame }),
        }
    }

    /// Maps `page` to the physical address `phys` with leaf bits `flags`,
    /// and links any new table on the way, from `alloc`.
    ///
    /// # Safety
    ///
    /// `phys` must be a frame the caller hands to these tables, a frame
    /// that stays allocated while the page maps it and that the kernel
    /// holds no reference into, or device registers.
    unsafe fn map_address(
        &self,
        page: u64,
        phys: u64,
        flags: u64,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapErrorKind> {
        let [i0, i1, i2, i3] = indices(page);
        let mut table = self.level_0;
        for index in [i0, i1, i2] {
            let entry = self.entry(table, index);
            let mut value = entry.load();
            if value & VALID == 0 {
                let Some(new) = alloc() else {
                    return Err(MapErrorKind::NoFrames);
                };
                value = self.new_table(new) | TABLE;
                // SAFETY: The entry isn't valid, so nothing is reached
                // through it yet.
                unsafe { entry.store(value) };
            }
            // Only Limine's mappings have blocks, and none is in kernel
            // space, the heap's range, the lower half or device registers,
            // so this is a kernel bug.
            assert!(
                value & TABLE_OR_PAGE != 0,
                "a block on the way to {page:#x}"
            );
            table = value & ADDRESS;
        }
        let leaf = self.entry(table, i3);
        if leaf.load() & VALID != 0 {
            return Err(MapErrorKind::Mapped);
        }
        // SAFETY: The entry isn't valid, so nothing is reached through it
        // yet, and the caller vouches for phys.
        unsafe { leaf.store(phys | flags) };
        // An invalid entry is never in a TLB, so the new one needs no
        // flush, only to reach the walker before the page is used.
        barrier::dsb(ISHST);
        barrier::isb(barrier::SY);
        Ok(())
    }

    /// Unmaps `page` and returns its frame, or `None` if it isn't mapped as
    /// a 4 KiB page. Then drops it from the TLB with `flush`.
    ///
    /// # Safety
    ///
    /// The page's frame must have come from `map_page`, nothing may reach
    /// other memory through the page once it is gone, and `flush` must drop
    /// every TLB entry for the page.
    unsafe fn unmap_page(&self, page: u64, flush: impl FnOnce(u64)) -> Option<Frame> {
        // SAFETY: As for this call.
        let phys = unsafe { self.clear_leaf(page, flush) }?;
        // SAFETY: map_page stored into_start of the frame there, and the
        // entry, now clear, was its only holder.
        Some(unsafe { Frame::reclaim(phys) })
    }

    /// Unmaps `page` and returns the address it mapped, or `None` if it
    /// isn't mapped as a 4 KiB page. Then drops it from the TLB with
    /// `flush`.
    ///
    /// # Safety
    ///
    /// Nothing may reach other memory through the page once it is gone,
    /// and `flush` must drop every TLB entry for the page.
    unsafe fn clear_leaf(&self, page: u64, flush: impl FnOnce(u64)) -> Option<u64> {
        let leaf = self.leaf(page)?;
        // SAFETY: The caller guarantees nothing reaches other memory through
        // the page once it is gone.
        let value = unsafe { leaf.clear() };
        flush(page);
        Some(value & ADDRESS)
    }

    /// Where `addr` is mapped in these tables, which map the half of the
    /// address space that holds `addr`, or `None` if it isn't.
    fn translate(&self, addr: u64) -> Option<Translation> {
        let mut table = self.level_0;
        for (level, index) in (0..4).zip(indices(addr)) {
            let value = self.entry(table, index).load();
            if value & VALID == 0 {
                return None;
            }
            if is_leaf(level, value) {
                let size = entry_size(level);
                let user = value & EL0 != 0;
                return Some(Translation {
                    phys: (value & ADDRESS & !(size - 1)) | (addr & (size - 1)),
                    writable: value & READ_ONLY == 0,
                    executable: value & if user { UXN } else { PXN } == 0,
                    user,
                });
            }
            table = value & ADDRESS;
        }
        None
    }

    /// Calls `free` with each table below the level `level` table at
    /// `table`. The frames that level 3 entries map are left alone: they
    /// are a user space's, whose mappings' objects hold them.
    ///
    /// # Safety
    ///
    /// Nothing may use the tables or the pages they map any more, and every
    /// table in them must have come from `new_table`.
    unsafe fn release(&self, table: u64, level: u8, free: &mut dyn FnMut(Frame)) {
        if level == 3 {
            return;
        }
        for index in 0..512 {
            let value = self.entry(table, index).load();
            if value & VALID == 0 {
                continue;
            }
            let frame = value & ADDRESS;
            // SAFETY: As for this call.
            unsafe { self.release(frame, level + 1, free) };
            // SAFETY: new_table took the frame with into_start, and nothing
            // uses the table any more.
            free(unsafe { Frame::reclaim(frame) });
        }
    }
}

/// Whether `value`, a valid entry in a level `level` table, maps memory
/// rather than pointing at a table.
fn is_leaf(level: u8, value: u64) -> bool {
    level == 3 || (level > 0 && value & TABLE_OR_PAGE == 0)
}

/// A page-table entry. Only `PageTables::entry` makes one, from a table it
/// reached through the physical-memory mapping, so the pointer is 8-byte
/// aligned and mapped, and the CPU's page walker, which only reads entries
/// while `TCR_EL1.HA` and HD are off, is its only other user while the
/// `PageTables` lives.
#[derive(Clone, Copy)]
struct Entry<'t>(*mut u64, PhantomData<&'t PageTables>);

impl Entry<'_> {
    fn load(self) -> u64 {
        // SAFETY: As the type says: aligned, mapped, and accessed only
        // atomically. The reference lasts this one load.
        unsafe { AtomicU64::from_ptr(self.0) }.load(Ordering::Relaxed)
    }

    /// Replaces the entry with `value`. The store is Release, so that what
    /// was written before it, such as the zeroes of a table it links, can't
    /// be moved after it.
    ///
    /// # Safety
    ///
    /// The entry must not be valid, or its table must not be in use, so
    /// that nothing the kernel reaches changes underneath it.
    unsafe fn store(self, value: u64) {
        // SAFETY: As in load.
        unsafe { AtomicU64::from_ptr(self.0) }.store(value, Ordering::Release);
    }

    /// Clears the entry, and returns what it held.
    ///
    /// # Safety
    ///
    /// Nothing may reach other memory through the entry's page once it is
    /// gone.
    unsafe fn clear(self) -> u64 {
        // SAFETY: As in load.
        unsafe { AtomicU64::from_ptr(self.0) }.swap(0, Ordering::Relaxed)
    }
}

/// Runs `tlbi <op>` on every CPU once the table stores before it are done,
/// and waits until the entries are gone. With a page address, the
/// instruction gets the page's number.
macro_rules! tlbi {
    ($op:literal) => {
        // SAFETY: TLB maintenance only makes the CPUs walk the tables
        // again; it reads and writes no memory the kernel uses.
        unsafe {
            asm!(
                "dsb ishst",
                concat!("tlbi ", $op),
                "dsb ish",
                "isb",
                options(nostack, preserves_flags),
            )
        }
    };
    ($op:literal, $page:expr) => {
        // SAFETY: As above.
        unsafe {
            asm!(
                "dsb ishst",
                concat!("tlbi ", $op, ", {}"),
                "dsb ish",
                "isb",
                in(reg) ($page >> 12) & ((1 << 44) - 1),
                options(nostack, preserves_flags),
            )
        }
    };
}

/// Drops the global entry for kernel page `page` from every TLB.
fn flush_kernel_page(page: u64) {
    tlbi!("vaae1is", page);
}

/// Drops every TLB entry of ASID 0, which every user space uses, and
/// keeps the kernel's global ones. The leading isb puts the caller's TTBR0
/// and TCR writes in effect first, so that no walk after the TLBI uses the
/// old values.
fn flush_user() {
    barrier::isb(barrier::SY);
    tlbi!("aside1is, xzr");
}

/// Drops the entry for user page `page`, in ASID 0, from every TLB.
fn flush_user_page(page: u64) {
    tlbi!("vae1is", page);
}

/// Turns TTBR0's walks off, so that the lower half is empty, and drops
/// what the TLB held of it.
fn empty_lower_half() {
    // SAFETY: With TTBR0's walks off every lower-half access faults, which
    // reaches no memory.
    unsafe { set_ttbr0_walks(false) };
    flush_user();
}

/// Whether TTBR0's walks are on, so that the lower half is mapped through
/// TTBR0's table.
fn ttbr0_walks() -> bool {
    TCR_EL1.matches_all(TCR_EL1::EPD0::EnableTTBR0Walks)
}

/// The page tables the kernel built and runs on. Only `take` makes one,
/// once.
#[derive(Debug)]
pub struct KernelPageTables {
    tables: PageTables,
    /// The level 0 entry the heap and `map` map into.
    space: u64,
    heap: Heap,
}

/// Whether `range` and the `size` bytes from `va` share an address.
fn overlaps(range: &Range<u64>, va: u64, size: u64) -> bool {
    !range.is_empty() && va < range.end && range.start.saturating_sub(va) < size
}

/// Whether the `size` bytes from `va` all lie in `range`.
fn inside(range: &Range<u64>, va: u64, size: u64) -> bool {
    range.contains(&va) && size <= range.end - va
}

/// The 4 KiB pages that hold `range`.
fn pages(range: &Range<u64>) -> Range<u64> {
    range.start & !0xfff..range.end.next_multiple_of(4096)
}

/// What the kernel's copy of Limine's leaf `value`, which maps `size`
/// bytes from `va`, is: global, out of EL0's reach, executable by EL1 only
/// if it is read-only and inside the kernel's `text_pages`, and normal
/// non-cacheable memory if it holds any of the framebuffer's `fb_pages`.
fn kernel_leaf(
    va: u64,
    size: u64,
    value: u64,
    text_pages: &Range<u64>,
    fb_pages: &Range<u64>,
) -> u64 {
    let code = value & READ_ONLY != 0 && inside(text_pages, va, size);
    let pxn = if code { 0 } else { PXN };
    let attr = if overlaps(fb_pages, va, size) {
        ATTR_NON_CACHEABLE
    } else {
        value & ATTR_INDEX
    };
    (value & !NOT_GLOBAL & !PXN & !ATTR_INDEX & !EL0) | UXN | pxn | attr
}

/// Copies Limine's upper half into new tables made from `alloc`'s frames,
/// letting EL1 execute only the kernel's `text` and making the framebuffer
/// at `fb` normal non-cacheable memory, and maps the physical `devices` as
/// device memory in the physical-memory mapping. Then switches to them,
/// empties the lower half and turns on PAN. Returns `None` after the first
/// call. Panics if the CPU lacks PAN, if Limine's `TCR_EL1` isn't what this
/// module assumes, if Limine mapped the boot stack's guard page at `guard`,
/// or if it mapped any of `devices`.
///
/// # Safety
///
/// `phys_offset` must be where Limine mapped all physical memory, `text`
/// must hold all of the kernel's code, `fb` must be the framebuffer or
/// empty, `devices` must be device registers and no memory, and nothing
/// may use the lower half.
pub unsafe fn take(
    phys_offset: u64,
    guard: u64,
    text: Range<u64>,
    fb: Range<u64>,
    devices: &[Range<u64>],
    alloc: &mut dyn FnMut() -> Option<Frame>,
) -> Option<KernelPageTables> {
    static TAKEN: AtomicBool = AtomicBool::new(false);
    if TAKEN.swap(true, Ordering::Relaxed) {
        return None;
    }
    assert!(
        ID_AA64MMFR1_EL1.read(ID_AA64MMFR1_EL1::PAN) != 0,
        "the CPU lacks PAN"
    );
    let tcr = TCR_EL1.extract();
    let t1sz = tcr.read(TCR_EL1::T1SZ);
    assert!(t1sz == 16, "Limine set TCR_EL1.T1SZ to {t1sz}, not 16");
    let tg1 = tcr.read(TCR_EL1::TG1);
    assert!(
        tcr.matches_all(TCR_EL1::TG1::KiB_4),
        "Limine set TCR_EL1.TG1 to {tg1:#b}, not 4 KiB"
    );
    // USER_SPACE and every user space's walk assume the same for TTBR0.
    let t0sz = tcr.read(TCR_EL1::T0SZ);
    assert!(t0sz == 16, "Limine set TCR_EL1.T0SZ to {t0sz}, not 16");
    let tg0 = tcr.read(TCR_EL1::TG0);
    assert!(
        tcr.matches_all(TCR_EL1::TG0::KiB_4),
        "Limine set TCR_EL1.TG0 to {tg0:#b}, not 4 KiB"
    );
    // Either would let the walker write entries, which the software Access
    // flag and Entry, whose walker only reads, assume it never does.
    assert!(
        tcr.read(TCR_EL1::HA) == 0 && tcr.read(TCR_EL1::HD) == 0,
        "Limine turned on TCR_EL1.HA or HD"
    );
    let loader = PageTables {
        phys_offset,
        level_0: TTBR1_EL1.get_baddr(),
    };
    assert!(
        loader.translate(guard).is_none(),
        "Limine mapped the boot stack's guard page at {guard:#x}"
    );
    // linker.ld starts the next segment on a new page, so the text's last
    // page holds nothing else.
    let text_pages = pages(&text);
    let fb_pages = pages(&fb);
    let level_0 = loader.copy(
        loader.level_0,
        0,
        UPPER_HALF,
        &|va, size, value| kernel_leaf(va, size, value, &text_pages, &fb_pages),
        // A block only partly framebuffer would make its other memory
        // non-cacheable, which is too slow for RAM.
        &|va, size| overlaps(&fb_pages, va, size) && !inside(&fb_pages, va, size),
        alloc,
    );
    let Some(space) = (0..512).find(|&i| loader.entry(loader.level_0, i).load() & VALID == 0)
    else {
        panic!("Limine left no level 0 entry free in the upper half");
    };
    let tables = PageTables {
        phys_offset,
        level_0,
    };
    let level_1 = tables.kernel_table(alloc);
    // SAFETY: The new tables aren't in use yet.
    unsafe { tables.entry(level_0, space).store(level_1 | TABLE) };
    for page in devices.iter().flat_map(|range| pages(range).step_by(4096)) {
        let flags = VALID | TABLE_OR_PAGE | ACCESSED | ATTR_DEVICE | PXN | UXN;
        // SAFETY: The caller guarantees the page holds device registers.
        if let Err(kind) = unsafe { tables.map_address(phys_offset + page, page, flags, alloc) } {
            panic!("mapping the device page {page:#x} failed: {kind:?}");
        }
    }
    let mair = MAIR_EL1.get() & !(0xffff << 16);
    // SAFETY: Limine's tables use attributes 0 and 1 alone, so changing 2
    // and 3 changes no live mapping.
    unsafe { write_mair(mair | (MAIR_NON_CACHEABLE << 16) | (MAIR_DEVICE << 24)) };
    // SAFETY: The new tables map every upper-half address to the same frame
    // as Limine's, with the same attributes apart from the global bit,
    // EL0 access, execute permissions and the framebuffer's cacheability,
    // and the kernel's code, stack, statics, Limine's answers, the
    // framebuffer and the physical-memory mapping are all in the upper
    // half. The kernel executes only its code, which the caller guarantees
    // is in text and stays read-only and executable. load_kernel's isb puts
    // the MAIR write in effect too.
    unsafe { load_kernel(level_0) };
    if !fb_pages.is_empty() {
        // SAFETY: fb_pages is the framebuffer, mapped in the new tables.
        unsafe { clean(fb_pages) };
    }
    // The caller guarantees nothing uses the lower half.
    empty_lower_half();
    // SAFETY: PAN only stops EL1 reaching EL0 pages, of which there are
    // none yet; the kernel never reaches one but through user copies, which
    // will clear PSTATE.PAN for it.
    unsafe { enable_pan() };
    Some(KernelPageTables {
        tables,
        space,
        // SAFETY: Limine mapped nothing in the space entry, TAKEN makes
        // this the only Heap, and only grow_heap maps in its range, since
        // map stays inside kernel_space, which starts above it.
        heap: unsafe { Heap::new(space_start(space)) },
    })
}

/// Makes TTBR1 point at the level 0 table at `level_0`, and drops every
/// TLB entry Limine's tables left.
///
/// # Safety
///
/// The table must map everything the kernel reaches as the tables TTBR1
/// points at now do, executable where the kernel executes.
unsafe fn load_kernel(level_0: u64) {
    // The Release stores that built the tables order them, but the walker
    // is only certain to see them after a dsb.
    barrier::dsb(ISHST);
    // SAFETY: The caller guarantees the table maps what the kernel reaches.
    unsafe { write_ttbr1(level_0) };
    barrier::isb(barrier::SY);
    // QEMU models no TLB conflicts; hardware that does could raise one
    // between the switch and this flush for an address whose old and new
    // entries differ in the global bit, which needs the switch made from
    // tables that map the switching code alone.
    tlbi!("vmalle1is");
}

/// Points `TTBR0_EL1` at the level 0 table at `table`, with ASID 0.
///
/// # Safety
///
/// While TTBR0's walks are on, the table must map nothing but frames it
/// holds, so that the lower half reaches no memory the kernel uses
/// otherwise.
unsafe fn write_ttbr0(table: u64) {
    // SAFETY: The caller guarantees what the table maps.
    unsafe { asm!("msr ttbr0_el1, {}", in(reg) table, options(nostack, preserves_flags)) };
}

/// Points `TTBR1_EL1` at the level 0 table at `table`.
///
/// # Safety
///
/// The table must map everything the kernel reaches, as the kernel expects
/// to reach it, executable where the kernel executes.
unsafe fn write_ttbr1(table: u64) {
    // SAFETY: The caller guarantees what the table maps.
    unsafe { asm!("msr ttbr1_el1, {}", in(reg) table, options(nostack, preserves_flags)) };
}

/// Turns TTBR0's walks on or off with `TCR_EL1.EPD0` (bit 7).
///
/// # Safety
///
/// Turning them on needs TTBR0 to hold a table that `write_ttbr0`'s
/// contract covers.
unsafe fn set_ttbr0_walks(on: bool) {
    let tcr = TCR_EL1.get();
    let tcr = if on { tcr & !(1 << 7) } else { tcr | (1 << 7) };
    // SAFETY: Only EPD0 changes, as the caller allows.
    unsafe { asm!("msr tcr_el1, {}", in(reg) tcr, options(nostack, preserves_flags)) };
}

/// Writes `value` to `MAIR_EL1`.
///
/// # Safety
///
/// Every attribute index a live mapping uses must keep its memory type, or
/// its TLB entries must be dropped before anything reaches it again.
unsafe fn write_mair(value: u64) {
    // SAFETY: The caller guarantees no live mapping changes type.
    unsafe { asm!("msr mair_el1, {}", in(reg) value, options(nostack, preserves_flags)) };
}

/// Sets PSTATE.PAN, and makes every exception taken to EL1 set it too.
///
/// # Safety
///
/// Nothing at EL1 may be using EL0 pages without clearing PAN itself.
unsafe fn enable_pan() {
    // SAFETY: Clearing SCTLR_EL1.SPAN (bit 23) only changes what an
    // exception does to PSTATE.PAN, and setting PAN only stops EL1 reaching
    // EL0 pages, which the caller guarantees nothing does.
    unsafe {
        asm!(
            "mrs {tmp}, sctlr_el1",
            "bic {tmp}, {tmp}, #(1 << 23)",
            "msr sctlr_el1, {tmp}",
            ".arch_extension pan",
            "msr pan, #1",
            "isb",
            tmp = out(reg) _,
            options(nostack, preserves_flags),
        );
    }
}

/// The first address that level 0 entry `index` of the upper half maps.
fn space_start(index: u64) -> u64 {
    UPPER_HALF + index * entry_size(0)
}

impl thisos_hal::PageTables for KernelPageTables {
    type AddressSpace<O: Deref<Target = [Frame]> + Send + Sync> = UserSpace<O>;

    fn kernel_space(&self) -> Range<u64> {
        let start = space_start(self.space);
        start + heap::SIZE..start + entry_size(0)
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
        let write = if writable { 0 } else { READ_ONLY };
        self.tables
            .map_page(page, frame, LEAF | PXN | UXN | write, alloc)
    }

    fn unmap(&mut self, page: u64) -> Option<Frame> {
        if page % 4096 != 0 || !self.kernel_space().contains(&page) {
            return None;
        }
        // SAFETY: Only map maps in kernel space, through map_page, and
        // kernel space's addresses are handed out once, so code still using
        // the page faults. The page is global, which flush_kernel_page
        // drops whatever the ASID.
        unsafe { self.tables.unmap_page(page, flush_kernel_page) }
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
        self.tables.map_page(next, frame, LEAF | PXN | UXN, alloc)?;
        // SAFETY: map_page mapped the page writable to the frame, and
        // nothing unmaps in the heap's range, maps there or reaches the
        // frame otherwise.
        unsafe { self.heap.grow() };
        Ok(())
    }

    fn translate(&self, addr: u64) -> Option<Translation> {
        if addr < UPPER_HALF {
            return None;
        }
        self.tables.translate(addr)
    }

    // These tables are TTBR1's and can't map the lower half, so this says
    // whether the CPU maps any of it now: whether TTBR0's walks are on with
    // a table that maps anything. The kernel's activate turns them off.
    fn lower_half_mapped(&self) -> bool {
        if !ttbr0_walks() {
            return false;
        }
        let ttbr0 = PageTables {
            phys_offset: self.tables.phys_offset,
            level_0: TTBR0_EL1.get_baddr(),
        };
        (0..512).any(|i| ttbr0.entry(ttbr0.level_0, i).load() & VALID != 0)
    }

    fn new_space<O: Deref<Target = [Frame]> + Send + Sync>(
        &self,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Option<UserSpace<O>> {
        let level_0 = self.tables.new_table(alloc()?);
        Some(UserSpace {
            tables: PageTables {
                phys_offset: self.tables.phys_offset,
                level_0,
            },
            kernel: self.tables.level_0,
            mappings: Mutex::new(Regions::new()),
        })
    }

    fn activate(&self) {
        empty_lower_half();
    }
}

/// A user address space: a level 0 table for TTBR0. Only
/// `KernelPageTables::new_space` makes one.
pub struct UserSpace<O> {
    tables: PageTables,
    /// The kernel's level 0 table, which maps the upper half for every
    /// space.
    kernel: u64,
    /// Every page mapped in the lower half is in one of these, and maps a
    /// frame of its object. Map and unmap change the tables only while
    /// they hold this lock.
    mappings: Mutex<Regions<O>>,
}

impl<O> UserSpace<O> {
    /// Whether the CPU translates the lower half through this space.
    fn active(&self) -> bool {
        ttbr0_walks() && TTBR0_EL1.get_baddr() == self.tables.level_0
    }
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
            Protection::ReadOnly => READ_ONLY | UXN,
            Protection::ReadWrite => UXN,
            Protection::ReadExecute => READ_ONLY,
        };
        let active = self.active();
        let flush = |page| {
            if active {
                flush_user_page(page);
            }
        };
        let mut mappings = self.mappings.lock();
        let object = match mappings.insert(addr..end, object) {
            Ok(object) => object,
            Err(object) => return fail(MapErrorKind::Mapped, object),
        };
        let mut done = addr;
        let mapped = object[first..first + pages].iter().try_for_each(|frame| {
            // SAFETY: The mapping now holds the object, which holds the
            // frame, until unmap or destroy clears the page. The kernel
            // never reaches user pages but through user copies.
            unsafe {
                self.tables.map_address(
                    done,
                    frame.start(),
                    LEAF | EL0 | NOT_GLOBAL | PXN | flags,
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
            unsafe { self.tables.clear_leaf(page, flush) };
        }
        let Some((_, object)) = mappings.remove(addr) else {
            panic!("the mapping at {addr:#x} went missing");
        };
        fail(kind, object)
    }

    fn unmap(&self, addr: u64) -> Option<O> {
        let mut mappings = self.mappings.lock();
        let (range, object) = mappings.remove(addr)?;
        let active = self.active();
        for page in range.step_by(4096) {
            // SAFETY: Only map maps in the lower half, every page of the
            // mapping, and the frames stay the object's, so nothing but
            // user code reaches memory through them, and the object goes
            // back only after the pages are gone. Every space uses ASID 0,
            // and switching spaces drops ASID 0's entries, so the TLB holds
            // the page only while this space is active.
            let cleared = unsafe {
                self.tables.clear_leaf(page, |page| {
                    if active {
                        flush_user_page(page);
                    }
                })
            };
            assert!(
                cleared.is_some(),
                "mapped page {page:#x} wasn't in the tables"
            );
        }
        Some(object)
    }

    fn translate(&self, addr: u64) -> Option<Translation> {
        let level_0 = if addr >= UPPER_HALF {
            self.kernel
        } else if addr < 1 << 48 {
            self.tables.level_0
        } else {
            return None;
        };
        PageTables {
            phys_offset: self.tables.phys_offset,
            level_0,
        }
        .translate(addr)
    }

    fn activate(&self) {
        // SAFETY: Only map maps in the space, with frames its mappings'
        // objects hold until the pages are gone.
        unsafe { write_ttbr0(self.tables.level_0) };
        // Without it the TCR write could take effect first, and walks
        // would start from the TTBR0 a destroyed space left.
        barrier::isb(barrier::SY);
        // SAFETY: TTBR0 holds this space's table, as above.
        unsafe { set_ttbr0_walks(true) };
        flush_user();
    }

    fn destroy(self, unmapped: &mut dyn FnMut(O), free: &mut dyn FnMut(Frame)) {
        if self.active() {
            empty_lower_half();
        }
        // SAFETY: self is gone after this, TTBR0's walks don't use its
        // tables, and every table in them came from new_table.
        unsafe { self.tables.release(self.tables.level_0, 0, free) };
        // SAFETY: new_space took the level 0 frame with into_start, and
        // nothing uses the table any more.
        free(unsafe { Frame::reclaim(self.tables.level_0) });
        // No table maps the objects' frames any more.
        for (_, object) in self.mappings.into_inner().take_all() {
            unmapped(object);
        }
    }
}

/// Where `addr` is mapped in the tables the CPU uses now.
pub fn translate_active(phys_offset: u64, addr: u64) -> Option<Translation> {
    // Only reads entries, one atomic load each, so it can share the tables
    // with their owner.
    let level_0 = if addr >= UPPER_HALF {
        TTBR1_EL1.get_baddr()
    } else if addr < 1 << 48 && ttbr0_walks() {
        TTBR0_EL1.get_baddr()
    } else {
        return None;
    };
    PageTables {
        phys_offset,
        level_0,
    }
    .translate(addr)
}

/// Writes `ret` at the start of `frame`, and makes instruction fetches see
/// it.
pub fn write_return(phys_offset: u64, frame: &mut Frame) {
    // A physical address, reached through the physical-memory mapping.
    let word = ptr_at::<u32>(phys_offset + frame.start());
    // SAFETY: The caller holds the frame mutably. Kernel page tables hold
    // the frames they map, and a user space holds an object that shares
    // the frames it maps, so neither maps this one: nothing maps it but the
    // physical-memory mapping, and no Rust reference to its memory exists.
    unsafe { word.write_volatile(0xd65f_03c0) };
    // SAFETY: Cleaning the data cache line to the point of unification and
    // invalidating the instruction cache change no memory, only which copy
    // of it an instruction fetch sees.
    unsafe {
        asm!(
            "dc cvau, {}",
            "dsb ish",
            "ic ialluis",
            "dsb ish",
            "isb",
            in(reg) word,
            options(nostack, preserves_flags),
        );
    }
}

/// Writes back and drops every data cache line that holds part of the
/// virtual addresses `range`. Lines the console's stores left while Limine
/// mapped the framebuffer write-back go out to memory this way; no new
/// ones come once it is non-cacheable.
///
/// # Safety
///
/// `range` must be mapped.
unsafe fn clean(range: Range<u64>) {
    let ctr: u64;
    // SAFETY: Reading CTR_EL0 into a register changes nothing.
    unsafe { asm!("mrs {}, ctr_el0", out(reg) ctr, options(nomem, nostack, preserves_flags)) };
    // DminLine, bits 19:16, is the smallest data cache line in words.
    let line = 4 << ((ctr >> 16) & 0xf);
    let mut addr = range.start & !(line - 1);
    while addr < range.end {
        // SAFETY: Cleaning and invalidating a line writes back what the
        // cache holds of a mapped address, and changes no memory's value.
        unsafe { asm!("dc civac, {}", in(reg) addr, options(nostack, preserves_flags)) };
        addr += line;
    }
    barrier::dsb(ISH);
}
