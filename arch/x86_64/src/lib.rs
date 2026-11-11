//! ThisOS's x86_64 support: the implementation of `thisos-hal` for x86_64.
//! It is the only crate allowed unsafe code, and the only one that may
//! depend on the `x86_64` crate.

#![no_std]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod apic;
mod backtrace;
mod catch;
mod context;
mod framebuffer;
mod heap;
mod memory;
mod once;
mod paging;
mod pat;
mod pic;
mod pit;
mod trap;
mod tsc;
mod user;

use core::arch::asm;
use core::fmt;
use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};

use bootloader_api::BootloaderConfig;
use bootloader_api::config::Mapping;
use bootloader_api::info::{FrameBuffer, PixelFormat};
use once::Once;
use thisos_frames::Frame;
use thisos_hal::{
    BacktraceEnd, BootInfo, Cpu, Framebuffer, HeapStats, Platform, Probe, Serial, TimerFrequencies,
    TimerFrequency, Translation, TrapHandler, UserProgram,
};
use uart_16550::backend::PioBackend;
use uart_16550::{BaudRate, Config, Uart16550Tty};
use x86_64::VirtAddr;
use x86_64::instructions::port::Port;
use x86_64::registers::control::{Cr4, Cr4Flags};
use x86_64::registers::rflags::{self, RFlags};

#[doc(hidden)]
pub use bootloader_api as __bootloader_api;
pub use context::Context;
pub use framebuffer::FramebufferPixels;
pub use memory::MemoryMap;
pub use paging::KernelPageTables;
pub use trap::TrapFrame;

/// The x86_64 architecture, as the kernel sees it through `thisos-hal`.
#[derive(Debug)]
pub struct X86_64;

impl Cpu for X86_64 {
    fn halt() -> ! {
        x86_64::instructions::interrupts::disable();
        // Drains the write-combining buffers, which hlt doesn't, so that the
        // last text written to the framebuffer reaches the screen.
        // SAFETY: sfence, which every x86_64 CPU has, only orders stores;
        // it reads no registers and changes no state the kernel relies on.
        // The target builds without SSE, which is why the intrinsic can't
        // be called safely.
        unsafe {
            asm!("sfence", options(nostack, preserves_flags));
        }
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

    fn breakpoint() {
        x86_64::instructions::interrupts::int3();
    }

    fn enable_interrupts() {
        x86_64::instructions::interrupts::enable();
    }

    fn wait_for_interrupt() {
        x86_64::instructions::interrupts::enable_and_hlt();
    }

    fn without_interrupts<R>(f: impl FnOnce() -> R) -> R {
        x86_64::instructions::interrupts::without_interrupts(f)
    }

    fn catch(probe: Probe) {
        catch::catch(probe);
    }

    fn user_access_blocked() -> bool {
        Cr4::read().contains(Cr4Flags::SUPERVISOR_MODE_ACCESS_PREVENTION)
            && !rflags::read().contains(RFlags::ALIGNMENT_CHECK)
    }

    fn enter_user(ip: u64, sp: u64, arg: u64) -> ! {
        user::enter_user(ip, sp, arg)
    }
}

impl Platform for X86_64 {
    type Serial = Com1;
    type Pixels = FramebufferPixels;
    type TrapFrame = TrapFrame;
    type MemoryMap = MemoryMap;
    type PageTables = KernelPageTables;
    type Context = Context;

    fn take_page_tables(alloc: &mut dyn FnMut() -> Option<Frame>) -> Option<KernelPageTables> {
        // SAFETY: boot_info recorded where the loader mapped all physical
        // memory, and loaded the kernel's own descriptor tables before the
        // kernel could call this.
        unsafe { paging::take(phys_offset(), alloc) }
    }

    fn activate_kernel_tables() {
        paging::activate_kernel();
    }

    fn move_trap_stacks(stack: &mut dyn FnMut(usize) -> u64) {
        trap::move_stacks(stack);
    }

    fn trap_stacks() -> impl Iterator<Item = Range<u64>> {
        trap::stacks()
    }

    fn new_context(stack: Range<u64>, entry: fn(Context) -> !) -> Context {
        context::new(stack, entry)
    }

    fn switch(to: Context) -> Context {
        context::switch(to)
    }

    fn heap_stats() -> HeapStats {
        heap::stats()
    }

    fn start_timer(hz: u32) -> TimerFrequency {
        apic::start(hz)
    }

    fn timer_frequencies() -> TimerFrequencies {
        apic::frequencies()
    }

    fn legacy_interrupts_masked() -> bool {
        pic::masked()
    }

    fn translate_active(addr: u64) -> Option<Translation> {
        paging::translate_active(phys_offset(), addr)
    }

    fn write_return(frame: &Frame) {
        paging::write_return(phys_offset(), frame);
    }

    fn write_user_program(frame: &Frame, program: UserProgram) {
        user::write_program(phys_offset(), frame, program);
    }

    fn guard_page() -> u64 {
        backtrace::stack_bottom() - 4096
    }

    fn probe_target(probe: Probe) -> Option<u64> {
        catch::target(probe)
    }

    fn panic_framebuffer() -> Option<Framebuffer<FramebufferPixels>> {
        PANIC_FRAMEBUFFER.try_lock()?.take()
    }

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

/// Where the loader mapped all physical memory, which `boot_info` records.
static PHYS_OFFSET: Once<VirtAddr> = Once::new();

fn phys_offset() -> VirtAddr {
    let Some(&offset) = PHYS_OFFSET.get() else {
        panic!("the loader didn't map physical memory");
    };
    offset
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
    // Everything the loader maps for the kernel goes in the upper half,
    // which leaves the lower half to user space. Only the loader's own
    // identity mappings are below it.
    config.mappings.dynamic_range_start = Some(0xffff_8000_0000_0000);
    config
};

/// Loads the descriptor tables with `trap` as the trap handler, builds the
/// kernel's [`BootInfo`] from the loader's, and sets up the devices it
/// holds. Panics if the CPU lacks SMEP or SMAP.
///
/// # Safety
///
/// Call this once, from the entry point, before anything else uses the
/// descriptor tables or the devices: COM1's and the legacy PICs' I/O ports.
///
/// `boot` is the one `&'static mut` that CONTRIBUTING allows, because the
/// loader's entry signature fixes it. It goes no further than this function.
#[doc(hidden)]
pub unsafe fn boot_info(
    boot: &'static mut bootloader_api::BootInfo,
    trap: TrapHandler<X86_64>,
) -> BootInfo<X86_64> {
    backtrace::init(
        boot.kernel_stack_bottom..boot.kernel_stack_bottom + boot.kernel_stack_len,
        boot.kernel_image_offset,
    );
    // Before the IDT, so that a panic from the first trap on finds the
    // panic path's handle to the screen.
    let framebuffer = boot.framebuffer.take().and_then(framebuffer);
    // SAFETY: The caller runs this once, first, so nothing has used the
    // segment registers or descriptor tables yet, and nothing has trapped.
    unsafe {
        trap::init(trap);
    }
    // SAFETY: The caller runs this once, before anything else, so interrupts
    // are still off and nothing has used the PICs' ports.
    unsafe {
        pic::disable();
    }
    // SAFETY: The caller runs this once, before anything else touches COM1's
    // eight ports, and the port moves into the BootInfo, so it is the only
    // user of those ports from here on.
    let tty = unsafe { Uart16550Tty::new_port(COM1_PORT, COM1_CONFIG) };
    // No device answering is normal: the laptop has no serial port.
    let serial = tty.ok().map(Com1);
    let phys_offset = boot.physical_memory_offset.into_option().map(VirtAddr::new);
    if let Some(offset) = phys_offset {
        PHYS_OFFSET.init(offset);
    }
    let framebuffer_write_combining = match (&framebuffer, phys_offset) {
        (Some(fb), Some(offset)) => {
            let range = fb.pixels.range();
            // SAFETY: BOOTLOADER_CONFIG has the loader map all physical
            // memory, at offset. The caller runs this once, before
            // anything else, so nothing else uses the page tables, and
            // the loader sets PWT on none of its pages, so none selects
            // PAT entry 1.
            unsafe { pat::write_combine(range, offset) }
        }
        _ => false,
    };
    // Nothing writes the loader's structure after this, so the rest is
    // read through a shared reference.
    let boot: &'static bootloader_api::BootInfo = boot;
    BootInfo {
        serial,
        framebuffer,
        framebuffer_write_combining,
        memory: MemoryMap::new(&boot.memory_regions),
        ramdisk: boot
            .ramdisk_addr
            .into_option()
            .map(|start| ramdisk(start, boot.ramdisk_len)),
    }
}

/// The ramdisk the loader mapped at `start`, `len` bytes long.
fn ramdisk(start: u64, len: u64) -> &'static [u8] {
    let Ok(len) = usize::try_from(len) else {
        panic!("the ramdisk's length {len:#x} doesn't fit a usize");
    };
    // The address is a BootInfo field, one of the integers CONTRIBUTING
    // lets the arch crate turn into a pointer.
    let start = VirtAddr::new(start).as_ptr::<u8>();
    // SAFETY: The loader maps the ramdisk's len bytes at start, in the upper
    // half that the kernel's page tables copy, so it stays mapped for the
    // kernel's life. Its frames are in a Loader region, which the frame
    // allocator never hands out, and nothing writes them, through this
    // mapping or the physical-memory one.
    unsafe { core::slice::from_raw_parts(start, len) }
}

fn framebuffer(mut fb: FrameBuffer) -> Option<Framebuffer<FramebufferPixels>> {
    let info = fb.info();
    let format = match (info.pixel_format, info.bytes_per_pixel) {
        (PixelFormat::Rgb, 4) => thisos_hal::PixelFormat::Rgb,
        (PixelFormat::Bgr, 4) => thisos_hal::PixelFormat::Bgr,
        _ => return None,
    };
    // buffer_mut's slice is the loader's only way to give the framebuffer's
    // address, and the one reference to it CONTRIBUTING allows: nothing
    // reads or writes through it, only its pointer range is kept.
    // SAFETY: The loader maps the framebuffer for the kernel's life.
    // boot_info, which runs once, takes this FrameBuffer from the loader's
    // BootInfo and hands it to nothing else, and no kernel code reads or
    // writes the framebuffer's frames through the physical-memory mapping.
    let [pixels, panic_pixels] =
        unsafe { FramebufferPixels::new(fb.buffer_mut().as_mut_ptr_range())? };
    let framebuffer = |pixels| Framebuffer {
        pixels,
        width: info.width,
        height: info.height,
        stride: info.stride,
        format,
    };
    *PANIC_FRAMEBUFFER.lock() = Some(framebuffer(panic_pixels));
    Some(framebuffer(pixels))
}

/// The second framebuffer handle, until a panic takes it.
static PANIC_FRAMEBUFFER: spin::Mutex<Option<Framebuffer<FramebufferPixels>>> =
    spin::Mutex::new(None);

/// Defines the kernel's entry point: `entry!(kmain, trap)` makes the boot
/// loader call `kmain(thisos_hal::BootInfo<X86_64>) -> !`, after making
/// `trap` the [`TrapHandler`]. The kernel uses this macro so that it never
/// names the boot loader's types. The function it generates takes the
/// loader's `&'static mut` and hands it straight to `boot_info`.
#[macro_export]
macro_rules! entry {
    ($main:path, $trap:path) => {
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
                let boot = unsafe { $crate::boot_info(boot, $trap) };
                $main(boot)
            }
        };
    };
}
