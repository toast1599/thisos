//! ThisOS's x86_64 support: the implementation of `thisos-hal` for x86_64.
//! It is the only crate allowed unsafe code, and the only one that may
//! depend on the `x86_64` crate.

#![no_std]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod backtrace;

use core::arch::asm;
use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};

use bootloader_api::BootloaderConfig;
use bootloader_api::config::Mapping;
use bootloader_api::info::{FrameBuffer, PixelFormat};
use thisos_hal::{BacktraceEnd, BootInfo, Cpu, Framebuffer, Platform, Serial};
use uart_16550::backend::PioBackend;
use uart_16550::{BaudRate, Config, Uart16550Tty};
use x86_64::instructions::port::Port;

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

    // Not inlined, so that rbp here is this function's own frame and the
    // first return address is in the caller.
    #[inline(never)]
    fn backtrace(f: impl FnMut(u64)) -> BacktraceEnd {
        let rbp: u64;
        // SAFETY: Copying rbp into a register reads no memory and changes no
        // other state.
        unsafe {
            asm!("mov {}, rbp", out(reg) rbp, options(nomem, nostack, preserves_flags));
        }
        backtrace::walk(rbp, f)
    }
}

impl Platform for X86_64 {
    type Serial = Com1;

    fn panic_serial() -> Option<Com1> {
        static TAKEN: AtomicBool = AtomicBool::new(false);
        if TAKEN.swap(true, Ordering::Relaxed) {
            return None;
        }
        // SAFETY: COM1's eight ports are the standard PC serial port's, so
        // they are valid for as long as the machine runs. The Com1 from
        // boot_info may still exist, which makes two users of the ports.
        // new_port re-initializes the UART and runs its loopback test, so
        // bytes the first user left in the transmit FIFO can be lost. Port
        // I/O to the UART touches no memory, so nothing else is affected.
        // TAKEN keeps it to two.
        let tty = unsafe { Uart16550Tty::new_port(COM1_PORT, COM1_CONFIG) };
        tty.ok().map(Com1)
    }

    fn exit_qemu(success: bool) -> ! {
        // What 0xf4 decodes to on the laptop's chipset is unknown, so real
        // hardware never sees the write.
        if under_hypervisor() {
            // QEMU exits with status (value << 1) | 1: 33 and 35. Success
            // isn't 0 because -no-reboot turns a triple fault into status 0.
            let value: u32 = if success { 0x10 } else { 0x11 };
            let mut port = Port::new(DEBUG_EXIT_PORT);
            // SAFETY: Under QEMU, 0xf4 is the isa-debug-exit device xtask
            // adds for test runs, and the write ends the process. Port I/O
            // touches no memory, so under a hypervisor without the device
            // nothing the kernel relies on changes; the CPU halts below.
            unsafe {
                port.write(value);
            }
        }
        Self::halt()
    }
}

/// Whether CPUID leaf 1 sets ECX bit 31, which real CPUs leave clear and
/// hypervisors set.
fn under_hypervisor() -> bool {
    core::arch::x86_64::__cpuid(1).ecx & (1 << 31) != 0
}

/// The I/O port xtask gives QEMU's isa-debug-exit device.
const DEBUG_EXIT_PORT: u16 = 0xf4;

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

const COM1_CONFIG: Config = Config {
    baud_rate: BaudRate::Baud115200,
    ..Config::DEFAULT
};

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
pub unsafe fn boot_info(boot: &'static mut bootloader_api::BootInfo) -> BootInfo<X86_64> {
    backtrace::init(
        boot.kernel_stack_bottom..boot.kernel_stack_bottom + boot.kernel_stack_len,
        boot.kernel_image_offset,
    );
    // SAFETY: The caller runs this once, before anything else touches COM1's
    // eight ports, and the port moves into the BootInfo, so it is the only
    // user of those ports from here on.
    let tty = unsafe { Uart16550Tty::new_port(COM1_PORT, COM1_CONFIG) };
    // No device answering is normal: the laptop has no serial port.
    let serial = tty.ok().map(Com1);
    let framebuffer = boot.framebuffer.take().and_then(framebuffer);
    BootInfo {
        serial,
        framebuffer,
    }
}

fn framebuffer(fb: FrameBuffer) -> Option<Framebuffer<'static>> {
    let info = fb.info();
    let format = match (info.pixel_format, info.bytes_per_pixel) {
        (PixelFormat::Rgb, 4) => thisos_hal::PixelFormat::Rgb,
        (PixelFormat::Bgr, 4) => thisos_hal::PixelFormat::Bgr,
        _ => return None,
    };
    Some(Framebuffer {
        buffer: fb.into_buffer(),
        width: info.width,
        height: info.height,
        stride: info.stride,
        format,
    })
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
