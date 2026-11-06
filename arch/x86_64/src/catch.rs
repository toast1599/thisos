//! Running a probe that traps, and coming back from it. `catch` saves the
//! callee-saved registers on the stack, records where, and jumps to the
//! probe; `abandon` points a trap frame at the code that restores them, so
//! the handler's `iretq` lands back in `catch`. The probes are assembly
//! loops, so the only frame abandoned is theirs, which holds nothing.

use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicU64, Ordering};

use thisos_hal::{Platform, Probe, TrapFrame as _};
use x86_64::VirtAddr;

use crate::X86_64;
use crate::trap::TrapFrame;

/// The stack pointer `thisos_catch` saved the registers at while a catch
/// runs, and 0 otherwise.
static SAVED_RSP: AtomicU64 = AtomicU64::new(0);

// thisos_catch takes the probe's number in rdi and the guard page's address
// in rsi. It returns only through thisos_catch_landing, where abandon sends
// a trapped probe. The overflow probe pushes from thisos_catch's own frame
// down, so the registers saved above it survive.
global_asm!(
    ".pushsection .text.thisos_catch, \"ax\", @progbits",
    ".globl thisos_catch",
    ".hidden thisos_catch",
    "thisos_catch:",
    "push rbp",
    "push rbx",
    "push r12",
    "push r13",
    "push r14",
    "push r15",
    "mov qword ptr [rip + {saved}], rsp",
    "cmp rdi, 0",
    "je 2f",
    "cmp rdi, 1",
    "je 3f",
    "1:",
    "pause",
    "jmp 1b",
    "2:",
    "mov al, byte ptr [rsi]",
    "jmp 2b",
    "3:",
    "push rax",
    "jmp 3b",
    ".globl thisos_catch_landing",
    ".hidden thisos_catch_landing",
    "thisos_catch_landing:",
    "mov qword ptr [rip + {saved}], 0",
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop rbx",
    "pop rbp",
    "ret",
    ".popsection",
    saved = sym SAVED_RSP,
);

unsafe extern "C" {
    static thisos_catch_landing: u8;
}

/// Runs `probe` until a trap handler abandons it.
pub fn catch(probe: Probe) {
    assert_eq!(
        SAVED_RSP.load(Ordering::Relaxed),
        0,
        "a catch is running already"
    );
    let number: u64 = match probe {
        Probe::ReadGuardPage => 0,
        Probe::OverflowStack => 1,
        Probe::Spin => 2,
    };
    // SAFETY: thisos_catch saves and restores every callee-saved register,
    // and returns, through thisos_catch_landing, to the instruction after
    // the call with the stack pointer it was called with. The probes touch
    // only rax and the stack below the saved registers: the guard page,
    // which the loader leaves unmapped, so the read only traps, and stack
    // that no frame above uses. The C ABI clobbers cover rax and the flags.
    unsafe {
        asm!(
            "call thisos_catch",
            in("rdi") number,
            in("rsi") X86_64::guard_page(),
            clobber_abi("C"),
        );
    }
}

/// Sends `frame` to the running catch's landing. Returns `false` when no
/// catch is running.
pub fn abandon(frame: &mut TrapFrame) -> bool {
    let rsp = SAVED_RSP.load(Ordering::Relaxed);
    if rsp == 0 {
        return false;
    }
    frame.resume_at(landing(), rsp);
    true
}

/// Whether `frame` will return to a catch's landing.
pub fn abandoned(frame: &TrapFrame) -> bool {
    frame.ip() == landing()
}

fn landing() -> u64 {
    VirtAddr::from_ptr(&raw const thisos_catch_landing).as_u64()
}
