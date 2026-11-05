//! The loader's memory map, in the HAL's terms.

use core::slice;

use bootloader_api::info::{self, MemoryRegionKind};
use thisos_hal::{MemoryKind, MemoryRegion};

/// The regions of the loader's memory map. The loader keeps them in a
/// Loader region, so whatever reclaims Loader regions has to be done with
/// the map first.
#[derive(Clone, Debug)]
pub struct MemoryMap(slice::Iter<'static, info::MemoryRegion>);

impl MemoryMap {
    pub(crate) fn new(regions: &'static [info::MemoryRegion]) -> Self {
        Self(regions.iter())
    }
}

impl Iterator for MemoryMap {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<MemoryRegion> {
        let region = self.0.next()?;
        let kind = match region.kind {
            MemoryRegionKind::Usable => MemoryKind::Usable,
            MemoryRegionKind::Bootloader => MemoryKind::Loader,
            // The loader boots through UEFI only, so a BIOS type never
            // comes up; it is a firmware type all the same.
            MemoryRegionKind::UnknownUefi(ty) | MemoryRegionKind::UnknownBios(ty) => {
                MemoryKind::Firmware(ty)
            }
            // MemoryRegionKind is non_exhaustive, and a kind added later
            // isn't free RAM unless this match says so.
            _ => MemoryKind::Firmware(u32::MAX),
        };
        Some(MemoryRegion {
            start: region.start,
            end: region.end,
            kind,
        })
    }
}
