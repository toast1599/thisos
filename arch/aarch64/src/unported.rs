//! Stand-ins for the HAL types aarch64 doesn't implement yet. Their one
//! field is private and nothing in the crate makes one, so no value of them
//! exists and their methods never run: the HAL calls that would make one
//! panic in `not_ported` first. They aren't empty enums because the
//! compiler would then flag the kernel's tests that use them as
//! unreachable. Each goes once its part is ported: the serial port and
//! trap frames.

use core::fmt;

use thisos_hal::BacktraceEnd;

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
