//! The loader's memory map, in the HAL's terms.

use core::{fmt, slice};

use limine::memmap::{
    Entry, MEMMAP_BOOTLOADER_RECLAIMABLE, MEMMAP_EXECUTABLE_AND_MODULES, MEMMAP_USABLE,
};
use thisos_hal::{MemoryKind, MemoryRegion};

/// The entries of Limine's memory map. Limine keeps them in bootloader
/// reclaimable memory, a Loader region, so whatever reclaims Loader regions
/// has to be done with the map first.
#[derive(Clone)]
pub struct MemoryMap(slice::Iter<'static, &'static Entry>);

// Limine's entries aren't Debug.
impl fmt::Debug for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryMap")
            .field("remaining", &self.0.len())
            .finish()
    }
}

impl MemoryMap {
    pub(crate) fn new(entries: &'static [&'static Entry]) -> Self {
        Self(entries.iter())
    }
}

impl Iterator for MemoryMap {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<MemoryRegion> {
        let entry = self.0.next()?;
        let kind = match entry.type_ {
            MEMMAP_USABLE => MemoryKind::Usable,
            MEMMAP_BOOTLOADER_RECLAIMABLE | MEMMAP_EXECUTABLE_AND_MODULES => MemoryKind::Loader,
            // Limine's own type numbers, which are small; one that isn't
            // still isn't free RAM.
            ty => MemoryKind::Firmware(u32::try_from(ty).unwrap_or(u32::MAX)),
        };
        Some(MemoryRegion {
            start: entry.base,
            // An entry past the top ends at it, so that its frames below
            // still count.
            end: entry.base.saturating_add(entry.length),
            kind,
        })
    }
}
