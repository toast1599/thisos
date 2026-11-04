//! The traits the ThisOS kernel needs from an architecture. Each
//! architecture crate implements them, and the kernel uses nothing else
//! from the hardware.

#![no_std]
#![forbid(unsafe_code)]

use core::fmt;

/// What the kernel learns from the boot loader, in a form that names no
/// boot protocol. Each architecture crate builds it from its loader's own
/// structure. Fields are added as the kernel comes to need them.
#[derive(Debug)]
pub struct BootInfo<P: Platform> {
    /// The serial port, or `None` when no device answers.
    pub serial: Option<P::Serial>,
}

/// Control of the processor the kernel runs on.
pub trait Cpu {
    /// Stops this processor for good: interrupts stay off and it never
    /// runs kernel code again.
    fn halt() -> !;
}

/// The devices an architecture hands the kernel in [`BootInfo`].
pub trait Platform {
    /// The serial port the kernel logs to.
    type Serial: Serial;
}

/// A serial port the kernel writes text to. Writing `\n` starts a new line
/// on the terminal at the other end.
pub trait Serial: fmt::Write + Send {}
