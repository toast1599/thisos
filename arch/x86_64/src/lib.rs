//! ThisOS's x86_64 support: the implementation of `thisos-hal` for x86_64.
//! It is the only crate allowed unsafe code, and the only one that may
//! depend on the `x86_64` crate.

#![no_std]
#![deny(clippy::unwrap_used, clippy::expect_used)]

use core::fmt;

use bootloader_api::BootloaderConfig;
use bootloader_api::config::Mapping;
use thisos_hal::{BootInfo, Cpu, Platform, Serial};
use uart_16550::backend::PioBackend;
use uart_16550::{BaudRate, Config, Uart16550Tty};

#[doc(hidden)]
pub use bootloader_api as __bootloader_api;

/// The x86_64 architecture, as the kernel sees it through `thisos-hal`.
#[derive(Debug)]
pub struct X86_64;

impl Cpu for X86_64 {
    fn halt() -> ! {
        x86_64::instructions::interrupts::disable();
        // An NMI or SMI can still wake the CPU from hlt, so go straight back.
        loop {
            x86_64::instructions::hlt();
        }
    }
}

impl Platform for X86_64 {
    type Serial = Com1;
}

/// COM1, the first PC serial port, at 115200 baud, 8N1, polled.
#[derive(Debug)]
pub struct Com1(Uart16550Tty<PioBackend>);

impl fmt::Write for Com1 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_str(s)
    }
}

impl Serial for Com1 {}

const COM1_PORT: u16 = 0x3F8;

#[doc(hidden)]
pub const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

/// Builds the kernel's [`BootInfo`] from the loader's, and sets up the
/// devices it holds.
///
/// # Safety
///
/// Call this once, from the entry point, before anything else uses the
/// devices: COM1's I/O ports.
#[doc(hidden)]
pub unsafe fn boot_info(_boot: &'static mut bootloader_api::BootInfo) -> BootInfo<X86_64> {
    let config = Config {
        baud_rate: BaudRate::Baud115200,
        ..Config::DEFAULT
    };
    // SAFETY: The caller runs this once, before anything else touches COM1's
    // eight ports, and the port moves into the BootInfo, so it is the only
    // user of those ports from here on.
    let tty = unsafe { Uart16550Tty::new_port(COM1_PORT, config) };
    // No device answering is normal: the laptop has no serial port.
    let serial = tty.ok().map(Com1);
    BootInfo { serial }
}

/// Defines the kernel's entry point: `entry!(kmain)` makes the boot loader
/// call `kmain(thisos_hal::BootInfo<X86_64>) -> !`. The kernel uses this
/// macro so that it never names the boot loader's types.
#[macro_export]
macro_rules! entry {
    ($main:path) => {
        // Inside an anonymous const, so the kernel can't name the entry
        // function and call it a second time.
        const _: () = {
            $crate::__bootloader_api::entry_point!(
                __thisos_start,
                config = &$crate::BOOTLOADER_CONFIG
            );

            fn __thisos_start(boot: &'static mut $crate::__bootloader_api::BootInfo) -> ! {
                // SAFETY: The loader jumps to the entry point once, and
                // nothing else can name this function.
                let boot = unsafe { $crate::boot_info(boot) };
                $main(boot)
            }
        };
    };
}
