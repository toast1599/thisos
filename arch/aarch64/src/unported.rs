//! Stand-ins for the HAL types aarch64 doesn't implement yet. Their one
//! field is private and nothing in the crate makes one, so no value of them
//! exists and their methods never run: the HAL calls that would make one
//! panic in `not_ported` first. They aren't empty enums because the
//! compiler would then flag the kernel's tests that use them as
//! unreachable. Each goes once its part is ported; only the serial port's
//! is left.

use core::fmt;

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
