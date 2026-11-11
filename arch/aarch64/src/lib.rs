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
mod devices;
mod framebuffer;
mod gic;
mod heap;
mod memory;
mod once;
mod paging;
mod timer;
mod trap;
mod unported;

use core::arch::asm;
use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};

use aarch64_cpu::asm::{barrier, wfi};
use aarch64_cpu::registers::{DAIF, ReadWriteable, Readable, Writeable};
use thisos_frames::Frame;
use thisos_hal::{
    BacktraceEnd, Cpu, Framebuffer, HeapStats, Platform, Probe, TimerFrequencies, TimerFrequency,
    Translation, UserProgram,
};

pub use framebuffer::FramebufferPixels;
pub use memory::MemoryMap;
pub use paging::{KernelPageTables, UserSpace};
pub use trap::TrapFrame;
pub use unported::{NoContext, NoSerial};

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

    fn wait_for_interrupt() {
        not_ported("Cpu::wait_for_interrupt")
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

    fn user_access_blocked() -> bool {
        let pan: u64;
        // SAFETY: Reading PSTATE.PAN into a register reads no memory and
        // changes no other state.
        unsafe {
            asm!(
                ".arch_extension pan",
                "mrs {}, pan",
                out(reg) pan,
                options(nomem, nostack, preserves_flags),
            );
        }
        // PSTATE.PAN reads back in bit 22.
        pan & (1 << 22) != 0
    }
    fn enter_user(_: u64, _: u64, _: u64) -> ! {
        not_ported("Cpu::enter_user")
    }
}

impl Platform for Aarch64 {
    type Serial = NoSerial;
    type Pixels = FramebufferPixels;
    type TrapFrame = TrapFrame;
    type MemoryMap = MemoryMap;
    type PageTables = KernelPageTables;
    type Context = NoContext;

    fn take_page_tables(alloc: &mut dyn FnMut() -> Option<Frame>) -> Option<KernelPageTables> {
        // SAFETY: boot_info recorded where Limine mapped all physical memory
        // and the framebuffer, and found the GIC's registers in the device
        // tree, and linker.ld puts every function in .text.
        // Limine's TTBR0 maps nothing the kernel uses: _start left Limine's
        // stack, and everything else is in the upper half.
        unsafe {
            paging::take(
                boot::phys_offset(),
                Self::guard_page(),
                boot::text(),
                boot::framebuffer_range(),
                &devices::get().registers(),
                alloc,
            )
        }
    }

    fn activate_kernel_tables() {
        not_ported("Platform::activate_kernel_tables")
    }

    fn move_trap_stacks(stack: &mut dyn FnMut(usize) -> u64) {
        trap::move_stack(stack);
    }

    fn trap_stacks() -> impl Iterator<Item = Range<u64>> {
        trap::stacks()
    }

    fn new_context(_: Range<u64>, _: fn(NoContext) -> !) -> NoContext {
        not_ported("Platform::new_context")
    }

    fn switch(_: NoContext) -> NoContext {
        not_ported("Platform::switch")
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

    fn start_timer(hz: u32) -> TimerFrequency {
        let devices = devices::get();
        gic::init(boot::phys_offset(), &devices);
        timer::start(hz, devices.timer)
    }

    // CNTFRQ_EL0 is all there is: QEMU's virt machine has no second clock
    // fine enough to check it against.
    fn timer_frequencies() -> TimerFrequencies {
        TimerFrequencies {
            reported: None,
            measured: None,
        }
    }

    // The GIC is the only interrupt controller.
    fn legacy_interrupts_masked() -> bool {
        true
    }

    fn translate_active(addr: u64) -> Option<Translation> {
        paging::translate_active(boot::phys_offset(), addr)
    }

    fn write_return(frame: &Frame) {
        paging::write_return(boot::phys_offset(), frame);
    }

    fn write_user_program(_: &Frame, _: UserProgram) {
        not_ported("Platform::write_user_program")
    }

    // Through semihosting's SYS_EXIT_EXTENDED, which QEMU turns into its
    // exit status: 33 and 35, as x86_64's debug exit device gives. Without
    // -semihosting-config the hlt is undefined, and the panic path its trap
    // takes calls this again, which then only halts.
    fn exit_qemu(success: bool) -> ! {
        static EXITING: AtomicBool = AtomicBool::new(false);
        if devices::qemu_virt() && !EXITING.swap(true, Ordering::Relaxed) {
            const SYS_EXIT_EXTENDED: u64 = 0x20;
            const APPLICATION_EXIT: u64 = 0x2_0026;
            let block = [APPLICATION_EXIT, if success { 33 } else { 35 }];
            // SAFETY: Under QEMU with semihosting, the call reads the two
            // words at x1 and ends the process; without it, the hlt traps as
            // an undefined instruction, and EXITING keeps it from running
            // again. On real hardware the root isn't QEMU's, so it never
            // runs.
            unsafe {
                asm!(
                    "hlt #0xf000",
                    inout("x0") SYS_EXIT_EXTENDED => _,
                    in("x1") block.as_ptr(),
                    options(nostack, readonly),
                );
            }
        }
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
/// from the loader, from a symbol the crate's own assembly defines, or from
/// the page tables, as the heap's base does.
#[expect(
    clippy::disallowed_methods,
    reason = "the address comes from the hardware, the loader or the page tables, never from a Rust pointer"
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
