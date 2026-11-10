//! ThisOS's aarch64 support: the implementation of `thisos-hal` for aarch64
//! under QEMU's virt machine, booted by Limine. Together with the x86_64
//! arch crate it is the only code allowed unsafe code, and the only crate
//! that may depend on `aarch64-cpu` and `limine`.
//!
//! Not every HAL call is ported yet: those that aren't panic with
//! `<call> isn't ported to aarch64`, and the types behind them are the
//! stand-ins in `unported`, which nothing can construct.

#![no_std]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod backtrace;
mod boot;
mod catch;
mod framebuffer;
mod heap;
mod memory;
mod once;
mod paging;
mod trap;
mod unported;

use core::arch::asm;
use core::ops::Range;

use aarch64_cpu::asm::{barrier, wfi};
use aarch64_cpu::registers::{DAIF, ReadWriteable, Readable, Writeable};
use thisos_frames::Frame;
use thisos_hal::{
    BacktraceEnd, Cpu, Framebuffer, HeapStats, Platform, Probe, TimerFrequencies, TimerFrequency,
    Translation,
};

pub use framebuffer::FramebufferPixels;
pub use memory::MemoryMap;
pub use paging::{KernelPageTables, UserSpace};
pub use trap::TrapFrame;
pub use unported::NoSerial;

/// The aarch64 architecture, as the kernel sees it through `thisos-hal`.
#[derive(Debug)]
pub struct Aarch64;

impl Cpu for Aarch64 {
    fn halt() -> ! {
        DAIF.write(DAIF::D::Masked + DAIF::A::Masked + DAIF::I::Masked + DAIF::F::Masked);
        // The framebuffer is non-cacheable, so the last pixels may still
        // sit in a write buffer; this drains them to the display.
        barrier::dsb(barrier::SY);
        // Masked interrupts still end wfi, so go straight back.
        loop {
            wfi();
        }
    }

    // Not inlined, so that x29 here is this function's own frame and the
    // first return address is in the caller.
    #[inline(never)]
    fn backtrace(f: impl FnMut(u64)) -> BacktraceEnd {
        let fp: u64;
        // SAFETY: Copying x29 into a register reads no memory and changes no
        // other state.
        unsafe {
            asm!("mov {}, x29", out(reg) fp, options(nomem, nostack, preserves_flags));
        }
        backtrace::walk(fp, f)
    }

    fn breakpoint() {
        // SAFETY: brk traps, and the trap handler returns past it with
        // every register as it was.
        unsafe {
            asm!("brk #0xf000", options(nomem, nostack, preserves_flags));
        }
    }

    fn enable_interrupts() {
        DAIF.modify(DAIF::I::Unmasked);
    }

    fn without_interrupts<R>(f: impl FnOnce() -> R) -> R {
        let masked = DAIF.is_set(DAIF::I);
        DAIF.modify(DAIF::I::Masked);
        let r = f();
        if !masked {
            DAIF.modify(DAIF::I::Unmasked);
        }
        r
    }

    fn catch(probe: Probe) {
        catch::catch(probe);
    }
}

impl Platform for Aarch64 {
    type Serial = NoSerial;
    type Pixels = FramebufferPixels;
    type TrapFrame = TrapFrame;
    type MemoryMap = MemoryMap;
    type PageTables = KernelPageTables;

    fn take_page_tables(alloc: &mut dyn FnMut() -> Option<Frame>) -> Option<KernelPageTables> {
        // SAFETY: boot_info recorded where Limine mapped all physical memory
        // and the framebuffer, and linker.ld puts every function in .text.
        // Limine's TTBR0 maps nothing the kernel uses: _start left Limine's
        // stack, and everything else is in the upper half.
        unsafe {
            paging::take(
                boot::phys_offset(),
                Self::guard_page(),
                boot::text(),
                boot::framebuffer_range(),
                alloc,
            )
        }
    }

    fn move_trap_stacks(stack: &mut dyn FnMut(usize) -> u64) {
        trap::move_stack(stack);
    }

    fn trap_stacks() -> impl Iterator<Item = Range<u64>> {
        trap::stacks()
    }

    fn guard_page() -> u64 {
        backtrace::stack_bottom() - 4096
    }

    fn probe_target(probe: Probe) -> Option<u64> {
        catch::target(probe)
    }

    fn panic_serial() -> Option<NoSerial> {
        None
    }

    fn panic_framebuffer() -> Option<Framebuffer<FramebufferPixels>> {
        boot::PANIC_FRAMEBUFFER.try_lock()?.take()
    }

    fn heap_stats() -> HeapStats {
        heap::stats()
    }

    fn start_timer(_hz: u32) -> TimerFrequency {
        not_ported("Platform::start_timer")
    }

    fn timer_frequencies() -> TimerFrequencies {
        not_ported("Platform::timer_frequencies")
    }

    fn legacy_interrupts_masked() -> bool {
        not_ported("Platform::legacy_interrupts_masked")
    }

    fn translate_active(addr: u64) -> Option<Translation> {
        paging::translate_active(boot::phys_offset(), addr)
    }

    fn write_return(frame: &Frame) {
        paging::write_return(boot::phys_offset(), frame);
    }

    // Only halts until semihosting is ported, so that a test run stops on
    // its summary and xtask's timeout ends it as a failure. Panicking here
    // would recurse, since the panic path calls this.
    fn exit_qemu(_success: bool) -> ! {
        Self::halt()
    }
}

/// Panics for a HAL call that has no aarch64 implementation yet.
#[track_caller]
fn not_ported(call: &str) -> ! {
    panic!("{call} isn't ported to aarch64")
}

/// The pointer at `addr`. The one place the crate turns an integer into a
/// pointer, for an address that reaches it as an integer from a register,
/// from the loader, or from a symbol the crate's own assembly defines.
#[expect(
    clippy::disallowed_methods,
    reason = "the address comes from the hardware or the loader, never from a Rust pointer"
)]
fn ptr_at<T>(addr: u64) -> *mut T {
    let Ok(addr) = usize::try_from(addr) else {
        unreachable!("aarch64 addresses are 64 bits");
    };
    core::ptr::with_exposed_provenance_mut(addr)
}

/// `addr` as a `u64`, which on aarch64 always holds a `usize`.
fn to_u64(addr: usize) -> u64 {
    let Ok(addr) = u64::try_from(addr) else {
        unreachable!("aarch64 addresses are 64 bits");
    };
    addr
}

#[doc(hidden)]
pub use boot::boot_info;

/// Defines the kernel's entry point: `entry!(kmain, trap)` makes the arch
/// crate's `_start` call `kmain(thisos_hal::BootInfo<Aarch64>) -> !` on the
/// boot stack, with `trap`, a `thisos_hal::TrapHandler`, as the handler for
/// every trap. The kernel uses this macro so that it never names the boot
/// loader's types.
#[macro_export]
macro_rules! entry {
    ($main:path, $trap:path) => {
        // Inside an anonymous const, so the kernel can't name the entry
        // function and call it a second time.
        const _: () = {
            // _start calls it by this name.
            #[unsafe(no_mangle)]
            extern "C" fn __thisos_entry() -> ! {
                // SAFETY: _start calls this once, and nothing else can name
                // it.
                let boot = unsafe { $crate::boot_info($trap) };
                $main(boot)
            }
        };
    };
}
