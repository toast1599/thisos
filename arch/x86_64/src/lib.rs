//! ThisOS's x86_64 support: the implementation of `thisos-hal` for x86_64.
//! It is the only crate allowed unsafe code, and the only one that may
//! depend on the `x86_64` crate.

#![no_std]
#![deny(clippy::unwrap_used, clippy::expect_used)]

use bootloader_api::BootloaderConfig;
use bootloader_api::config::Mapping;
use thisos_hal::{BootInfo, Cpu};

#[doc(hidden)]
pub use bootloader_api as __bootloader_api;

/// The x86_64 architecture, as the kernel sees it through `thisos-hal`.
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

#[doc(hidden)]
pub const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

#[doc(hidden)]
pub fn boot_info(_boot: &'static mut bootloader_api::BootInfo) -> BootInfo {
    BootInfo {}
}

/// Defines the kernel's entry point: `entry!(kmain)` makes the boot loader
/// call `kmain(thisos_hal::BootInfo) -> !`. The kernel uses this macro so
/// that it never names the boot loader's types.
#[macro_export]
macro_rules! entry {
    ($main:path) => {
        $crate::__bootloader_api::entry_point!(__thisos_start, config = &$crate::BOOTLOADER_CONFIG);

        fn __thisos_start(boot: &'static mut $crate::__bootloader_api::BootInfo) -> ! {
            $main($crate::boot_info(boot))
        }
    };
}
