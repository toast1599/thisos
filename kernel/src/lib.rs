//! The ThisOS kernel. It is generic: it reaches the hardware only through
//! `thisos-hal`'s traits, and never names an architecture's types. The
//! kernel binary and the faulting tests in `tests/` are entry points into
//! this library.

#![no_std]
#![cfg_attr(test, no_main)]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::run)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod frames;
mod heap;
mod logger;
mod paging;
mod panic;
#[cfg(feature = "testing")]
pub mod testing;
mod trap;

use thisos_hal::{BootInfo, Cpu};

pub use trap::trap;

/// The architecture the kernel is built for.
#[cfg(target_arch = "x86_64")]
pub type Arch = thisos_arch_x86_64::X86_64;

#[cfg(target_arch = "x86_64")]
pub use thisos_arch_x86_64::entry;

#[cfg(test)]
entry!(test_kmain, trap);

/// The commit, profile and build time, which `cargo xtask build` passes in.
const BUILD: &str = match option_env!("THISOS_BUILD") {
    Some(build) => build,
    None => "unknown build",
};

/// Starts logging to the devices in `boot`, and logs the build ID first.
/// Every entry point calls this once, first.
pub fn init(boot: BootInfo<Arch>) {
    let has_framebuffer = boot.framebuffer.is_some();
    logger::init(boot.serial, boot.framebuffer);
    log::info!("ThisOS {} {BUILD}", env!("CARGO_PKG_VERSION"));
    if !has_framebuffer {
        log::warn!("no framebuffer in a format the console draws; logging to serial only");
    } else if boot.framebuffer_write_combining {
        log::info!("framebuffer is write-combining");
    } else {
        log::warn!("framebuffer isn't write-combining, so the console may be slow");
    }
    frames::init(boot.memory);
    paging::init();
    heap::init();
}

/// The kernel binary's main function.
pub fn kmain(boot: BootInfo<Arch>) -> ! {
    init(boot);
    Arch::halt()
}

#[cfg(test)]
fn test_kmain(boot: BootInfo<Arch>) -> ! {
    init(boot);
    test_main();
    Arch::halt()
}
