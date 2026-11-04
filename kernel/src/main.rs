//! The ThisOS kernel. It is generic: it reaches the hardware only through
//! `thisos-hal`'s traits, and never names an architecture's types.

#![no_std]
#![no_main]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod logger;
mod panic;

use thisos_hal::{BootInfo, Cpu};

#[cfg(target_arch = "x86_64")]
type Arch = thisos_arch_x86_64::X86_64;

#[cfg(target_arch = "x86_64")]
thisos_arch_x86_64::entry!(kmain);

/// The commit, profile and build time, which `cargo xtask build` passes in.
const BUILD: &str = match option_env!("THISOS_BUILD") {
    Some(build) => build,
    None => "unknown build",
};

fn kmain(boot: BootInfo<Arch>) -> ! {
    let has_framebuffer = boot.framebuffer.is_some();
    logger::init(boot.serial, boot.framebuffer);
    log::info!("ThisOS {} {BUILD}", env!("CARGO_PKG_VERSION"));
    if !has_framebuffer {
        log::warn!("no framebuffer in a format the console draws; logging to serial only");
    }
    Arch::halt()
}
