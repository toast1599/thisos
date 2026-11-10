//! Running a probe that traps, and coming back from it. `catch` saves the
//! callee-saved registers on the stack, records where, and branches to the
//! probe; `abandon` points a trap frame at the code that restores them, so
//! the handler's `eret` lands back in `catch`. The probes are assembly
//! loops, so the only frame abandoned is theirs, which holds nothing.

use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicU64, Ordering};

use thisos_hal::{Cpu, Platform, Probe, TrapFrame as _};

use crate::paging::USER_SPACE;
use crate::trap::TrapFrame;
use crate::{Aarch64, to_u64};

/// The stack pointer `thisos_catch` saved the registers at while a catch
/// runs, and 0 otherwise.
static SAVED_SP: AtomicU64 = AtomicU64::new(0);

// thisos_catch takes the probe's number in x0 and its target's address in
// x1. It returns only through thisos_catch_landing, where abandon sends a
// trapped probe, and which takes SP from SAVED_SP since eret leaves it
// wherever the trap's stub dropped its frame. The overflow probe pushes
// from thisos_catch's own frame down, so the registers saved above it
// survive. The two targets are in .rodata, which the kernel's page tables
// map read-only and no-execute, and Rust code never reads them, so a write
// that the CPU lets through changes nothing Rust relies on.
global_asm!(
    ".pushsection .rodata.thisos_probe, \"a\", %progbits",
    ".balign 4",
    ".globl thisos_probe_ret",
    ".hidden thisos_probe_ret",
    "thisos_probe_ret:",
    "ret",
    ".globl thisos_probe_read_only",
    ".hidden thisos_probe_read_only",
    "thisos_probe_read_only:",
    ".byte 0",
    ".popsection",
    ".pushsection .text.thisos_catch, \"ax\", %progbits",
    ".globl thisos_catch",
    ".hidden thisos_catch",
    "thisos_catch:",
    "stp x29, x30, [sp, #-96]!",
    "stp x19, x20, [sp, #16]",
    "stp x21, x22, [sp, #32]",
    "stp x23, x24, [sp, #48]",
    "stp x25, x26, [sp, #64]",
    "stp x27, x28, [sp, #80]",
    "mov x9, sp",
    "adrp x10, {saved}",
    "str x9, [x10, :lo12:{saved}]",
    "cbz x0, 2f",
    "cmp x0, #1",
    "b.eq 3f",
    "cmp x0, #3",
    "b.eq 4f",
    "cmp x0, #4",
    "b.eq 5f",
    "1:",
    "yield",
    "b 1b",
    "2:",
    "ldrb w9, [x1]",
    "b 2b",
    "3:",
    "str x9, [sp, #-16]!",
    "b 3b",
    "4:",
    "ldrb w9, [x1]",
    "strb w9, [x1]",
    "b 4b",
    "5:",
    "blr x1",
    "b 5b",
    ".globl thisos_catch_landing",
    ".hidden thisos_catch_landing",
    "thisos_catch_landing:",
    "adrp x10, {saved}",
    "ldr x9, [x10, :lo12:{saved}]",
    "mov sp, x9",
    "str xzr, [x10, :lo12:{saved}]",
    "ldp x19, x20, [sp, #16]",
    "ldp x21, x22, [sp, #32]",
    "ldp x23, x24, [sp, #48]",
    "ldp x25, x26, [sp, #64]",
    "ldp x27, x28, [sp, #80]",
    "ldp x29, x30, [sp], #96",
    "ret",
    ".popsection",
    saved = sym SAVED_SP,
);

unsafe extern "C" {
    static thisos_catch_landing: u8;
    static thisos_probe_read_only: u8;
    static thisos_probe_ret: u8;
}

/// Runs `probe` until a trap handler abandons it.
pub fn catch(probe: Probe) {
    assert_eq!(
        SAVED_SP.load(Ordering::Relaxed),
        0,
        "a catch is running already"
    );
    let number: u64 = match probe {
        Probe::ReadGuardPage | Probe::ReadUser => 0,
        Probe::OverflowStack => 1,
        Probe::Spin => 2,
        Probe::WriteReadOnly => 3,
        Probe::ExecuteNoExecute | Probe::ExecuteUser => 4,
    };
    let run = || {
        // SAFETY: thisos_catch saves and restores every callee-saved
        // register, and returns, through thisos_catch_landing, to the
        // instruction after the call with the stack pointer it was called
        // with. The probes touch only x9, x10, x30, the stack below the
        // saved registers and their own targets: the guard page, which
        // take_page_tables leaves unmapped, so the read only traps; stack
        // that no frame above uses; the read-only byte, written back with
        // the value it holds; the ret, which only returns; and the user
        // probe page, which the read can't change, and which the call can't
        // run, since every user page is PXN. The C ABI clobbers cover x9,
        // x10, x30 and the flags.
        unsafe {
            asm!(
                "bl thisos_catch",
                in("x0") number,
                in("x1") target(probe).unwrap_or(0),
                clobber_abi("C"),
            );
        }
    };
    match probe {
        // Interrupt handlers run on the stack the probe fills, so one that
        // came near the guard page would overflow there itself, and abandon
        // would give up the handler along with the probe.
        Probe::OverflowStack => Aarch64::without_interrupts(run),
        Probe::ReadGuardPage
        | Probe::Spin
        | Probe::WriteReadOnly
        | Probe::ExecuteNoExecute
        | Probe::ReadUser
        | Probe::ExecuteUser => run(),
    }
}

/// The address whose page fault ends `probe`; see `Platform::probe_target`.
pub fn target(probe: Probe) -> Option<u64> {
    match probe {
        Probe::ReadGuardPage => Some(Aarch64::guard_page()),
        Probe::WriteReadOnly => Some(address(&raw const thisos_probe_read_only)),
        Probe::ExecuteNoExecute => Some(address(&raw const thisos_probe_ret)),
        Probe::ReadUser | Probe::ExecuteUser => Some(USER_SPACE.start),
        Probe::OverflowStack | Probe::Spin => None,
    }
}

fn address(byte: *const u8) -> u64 {
    to_u64(byte.addr())
}

/// Sends `frame` to the running catch's landing. Returns `false` when no
/// catch is running.
pub fn abandon(frame: &mut TrapFrame) -> bool {
    if SAVED_SP.load(Ordering::Relaxed) == 0 {
        return false;
    }
    frame.resume_at(landing());
    true
}

/// Whether `frame` will return to a catch's landing.
pub fn abandoned(frame: &TrapFrame) -> bool {
    frame.ip() == landing()
}

fn landing() -> u64 {
    address(&raw const thisos_catch_landing)
}
