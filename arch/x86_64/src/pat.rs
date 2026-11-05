//! Mapping the framebuffer write-combining. Firmware leaves it uncached
//! (UC) in the MTRRs, where every store is its own bus transaction; with
//! write-combining (WC) the CPU gathers stores into 64-byte bursts. A page
//! gets WC by selecting a PAT entry that holds WC: entry 1, as Linux uses,
//! picked by PWT alone.

use core::ops::Range;

use x86_64::VirtAddr;
use x86_64::registers::control::Cr3;
use x86_64::registers::model_specific::Msr;
use x86_64::structures::paging::mapper::{MappedFrame, MapperFlush, TranslateResult};
use x86_64::structures::paging::{
    Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, Size4KiB, Translate,
};

const IA32_MTRRCAP: u32 = 0xfe;
const IA32_PAT: u32 = 0x277;
const IA32_MTRR_DEF_TYPE: u32 = 0x2ff;
const IA32_MTRR_PHYSBASE0: u32 = 0x200;

const UC: u64 = 0x00;
const WC: u64 = 0x01;

/// The PAT and MTRR MSRs. Only `new` makes one, after CPUID reports both,
/// so every MSR its methods read or write exists.
struct Msrs {
    variable_ranges: u32,
}

impl Msrs {
    fn new() -> Option<Self> {
        let edx = core::arch::x86_64::__cpuid(1).edx;
        let (mtrr, pat) = (1 << 12, 1 << 16);
        if edx & mtrr == 0 || edx & pat == 0 {
            return None;
        }
        let mut msrs = Self { variable_ranges: 0 };
        msrs.variable_ranges = u32::from(msrs.read(IA32_MTRRCAP).to_le_bytes()[0]);
        Some(msrs)
    }

    /// Reads `msr`, which is `IA32_MTRRCAP`, `IA32_PAT`,
    /// `IA32_MTRR_DEF_TYPE` or a variable-range pair below `IA32_MTRRCAP`'s
    /// count: the methods below pass nothing else.
    #[expect(
        clippy::unused_self,
        reason = "a Msrs is the proof that CPUID reported these MSRs"
    )]
    fn read(&self, msr: u32) -> u64 {
        // SAFETY: CPUID reported MTRRs and the PAT, so the MSRs the methods
        // here pass exist, and reading them changes no state.
        unsafe { Msr::new(msr).read() }
    }

    /// Whether the MTRRs make the 4 KiB frame at `addr` UC: MTRRs off, or
    /// a variable range covering it is UC, or none covers it and the
    /// default type is UC. Frames below 1 MiB, where the fixed-range MTRRs
    /// apply, count as not UC, since nothing here reads those.
    fn uncached(&self, addr: u64) -> bool {
        let def = self.read(IA32_MTRR_DEF_TYPE);
        let (enabled, fixed_enabled) = (1 << 11, 1 << 10);
        if def & enabled == 0 {
            return true;
        }
        if addr < 0x10_0000 && def & fixed_enabled != 0 {
            return false;
        }
        let mut covered = false;
        for i in 0..self.variable_ranges {
            let base = self.read(IA32_MTRR_PHYSBASE0 + 2 * i);
            let mask = self.read(IA32_MTRR_PHYSBASE0 + 2 * i + 1);
            let valid = 1 << 11;
            let bits = mask & !0xfff;
            if mask & valid == 0 || addr & bits != base & bits {
                continue;
            }
            // UC wins over every other type a second range gives.
            if base & 0xff == UC {
                return true;
            }
            covered = true;
        }
        !covered && def & 0xff == UC
    }

    /// Changes PAT entry 1 from its power-on WT to WC.
    ///
    /// # Safety
    ///
    /// No page may select PAT entry 1: that is, none may have PWT set
    /// without PCD and the PAT bit.
    unsafe fn set_entry_1_wc(&mut self) {
        let pat = self.read(IA32_PAT);
        let pat = (pat & !0xff00) | (WC << 8);
        // SAFETY: `IA32_PAT` exists, as new checked. The caller guarantees no
        // page selects entry 1, so no memory changes type under a mapping
        // and nothing cached needs flushing first.
        unsafe { Msr::new(IA32_PAT).write(pat) };
    }
}

/// The kernel's page tables, reached through the loader's mapping of all
/// physical memory. Only `new` makes one.
struct PageTables(OffsetPageTable<'static>);

impl PageTables {
    /// # Safety
    ///
    /// `phys_offset` must be where the loader mapped all physical memory,
    /// and nothing else may use the page tables while this exists.
    unsafe fn new(phys_offset: VirtAddr) -> Self {
        let (frame, _) = Cr3::read();
        let table = (phys_offset + frame.start_address().as_u64()).as_mut_ptr::<PageTable>();
        // SAFETY: The caller guarantees that phys_offset maps all physical
        // memory, so this is the level 4 table CR3 names, and that nothing
        // else uses the tables, so this is the only reference to it. The CPU's
        // page walker still sets Accessed and Dirty bits in it while the
        // reference lives, which the aliasing rules don't allow. That is
        // accepted here because the reference lasts one call, on one core,
        // and nothing reads those bits; see the open question on page tables
        // in docs/design.md.
        let table = unsafe { &mut *table };
        // SAFETY: As above: the offset maps every frame the tables use, and
        // this is the only user of the level 4 table.
        Self(unsafe { OffsetPageTable::new(table, phys_offset) })
    }

    /// The frame and flags of `page`, when it is mapped as a 4 KiB page.
    fn entry(&self, page: Page) -> Option<(u64, PageTableFlags)> {
        match self.0.translate(page.start_address()) {
            TranslateResult::Mapped {
                frame: MappedFrame::Size4KiB(frame),
                flags,
                ..
            } => Some((frame.start_address().as_u64(), flags)),
            _ => None,
        }
    }

    /// Sets PWT on `page`, which then selects PAT entry 1, and flushes it
    /// from the TLB. Returns whether the page was mapped to update.
    ///
    /// # Safety
    ///
    /// PAT entry 1 must hold a memory type `page` may have: for WC, every
    /// other mapping of its frame must be UC.
    unsafe fn select_entry_1(&mut self, page: Page, flags: PageTableFlags) -> bool {
        let flags = (flags - PageTableFlags::NO_CACHE - PageTableFlags::HUGE_PAGE)
            | PageTableFlags::WRITE_THROUGH;
        // SAFETY: The page stays mapped to the same frame with the same
        // access rights, so every reference into it stays valid; only its
        // memory type changes, which the caller guarantees is allowed.
        let result = unsafe { self.0.update_flags(page, flags) };
        result.map(MapperFlush::flush).is_ok()
    }
}

/// Makes the mapping at `range` write-combining, and returns whether it
/// did. It does nothing unless every page of the range is mapped 4 KiB and
/// the MTRRs make every frame UC.
///
/// # Safety
///
/// `phys_offset` must be where the loader mapped all physical memory, and
/// nothing else may use the page tables until this returns. No page may
/// select PAT entry 1 yet.
pub unsafe fn write_combine(range: Range<VirtAddr>, phys_offset: VirtAddr) -> bool {
    let Some(mut msrs) = Msrs::new() else {
        return false;
    };
    if range.is_empty() {
        return false;
    }
    // SAFETY: The caller guarantees both conditions.
    let mut tables = unsafe { PageTables::new(phys_offset) };
    let pages = Page::<Size4KiB>::range_inclusive(
        Page::containing_address(range.start),
        Page::containing_address(range.end - 1u64),
    );
    // Checked in full first, so that a failure changes nothing.
    for page in pages {
        match tables.entry(page) {
            Some((frame, _)) if msrs.uncached(frame) => {}
            _ => return false,
        }
    }
    // SAFETY: The caller guarantees no page selects entry 1 yet.
    unsafe { msrs.set_entry_1_wc() };
    let mut all = true;
    for page in pages {
        let Some((_, flags)) = tables.entry(page) else {
            all = false;
            continue;
        };
        // SAFETY: Entry 1 now holds WC. The MTRRs make the page's frame UC,
        // as checked above, so the frame's other mapping, in the loader's
        // mapping of all physical memory, is UC too, and the WC page has no
        // cacheable alias.
        all &= unsafe { tables.select_entry_1(page, flags) };
    }
    all
}
