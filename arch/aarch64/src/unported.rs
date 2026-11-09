//! Stand-ins for the HAL types aarch64 doesn't implement yet. Their one
//! field is private and nothing in the crate makes one, so no value of them
//! exists and their methods never run: the HAL calls that would make one
//! panic in `not_ported` first. They aren't empty enums because the
//! compiler would then flag the kernel's tests that use them as
//! unreachable. Each goes once its part is ported: the serial port, page
//! tables and address spaces, and trap frames.

use core::fmt;
use core::ops::Range;

use thisos_frames::Frame;
use thisos_hal::{BacktraceEnd, MapError, Protection, Translation};

use crate::not_ported;

/// No serial port yet: `BootInfo::serial` is always `None`.
#[derive(Debug)]
pub struct NoSerial(());

impl fmt::Write for NoSerial {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        not_ported("Serial::write_str")
    }
}

impl thisos_hal::Serial for NoSerial {}

/// No trap frames yet: nothing installs exception vectors.
#[derive(Debug)]
pub struct TrapFrame(());

impl fmt::Display for TrapFrame {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        not_ported("TrapFrame::fmt")
    }
}

impl thisos_hal::TrapFrame for TrapFrame {
    fn ip(&self) -> u64 {
        not_ported("TrapFrame::ip")
    }

    fn sp(&self) -> u64 {
        not_ported("TrapFrame::sp")
    }

    fn backtrace(&self, _: impl FnMut(u64)) -> BacktraceEnd {
        not_ported("TrapFrame::backtrace")
    }

    fn abandon(&mut self) -> bool {
        not_ported("TrapFrame::abandon")
    }
}

/// No kernel page tables yet: `take_page_tables` doesn't return.
#[derive(Debug)]
pub struct KernelPageTables(());

impl thisos_hal::PageTables for KernelPageTables {
    type AddressSpace = AddressSpace;

    fn kernel_space(&self) -> Range<u64> {
        not_ported("PageTables::kernel_space")
    }

    fn map(
        &mut self,
        _: u64,
        _: Frame,
        _: bool,
        _: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError> {
        not_ported("PageTables::map")
    }

    fn unmap(&mut self, _: u64) -> Option<Frame> {
        not_ported("PageTables::unmap")
    }

    fn grow_heap(
        &mut self,
        _: Frame,
        _: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError> {
        not_ported("PageTables::grow_heap")
    }

    fn translate(&self, _: u64) -> Option<Translation> {
        not_ported("PageTables::translate")
    }

    fn lower_half_mapped(&self) -> bool {
        not_ported("PageTables::lower_half_mapped")
    }

    fn new_space(&self, _: &mut dyn FnMut() -> Option<Frame>) -> Option<AddressSpace> {
        not_ported("PageTables::new_space")
    }

    fn activate(&self) {
        not_ported("PageTables::activate");
    }
}

/// No user address spaces yet: only `KernelPageTables` makes one.
#[derive(Debug)]
pub struct AddressSpace(());

impl thisos_hal::AddressSpace for AddressSpace {
    fn user_space(&self) -> Range<u64> {
        not_ported("AddressSpace::user_space")
    }

    fn map(
        &mut self,
        _: u64,
        _: Frame,
        _: Protection,
        _: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError> {
        not_ported("AddressSpace::map")
    }

    fn unmap(&mut self, _: u64) -> Option<Frame> {
        not_ported("AddressSpace::unmap")
    }

    fn translate(&self, _: u64) -> Option<Translation> {
        not_ported("AddressSpace::translate")
    }

    fn activate(&self) {
        not_ported("AddressSpace::activate");
    }

    fn destroy(self, _: &mut dyn FnMut(Frame), _: &mut dyn FnMut(Frame)) {
        not_ported("AddressSpace::destroy");
    }
}
