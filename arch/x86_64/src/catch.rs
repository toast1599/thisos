//! Running a probe that traps, and coming back from it. `catch` saves the
//! callee-saved registers on the stack, records where, and jumps to the
//! probe; `abandon` points a trap frame at the code that restores them, so
//! the handler's `iretq` lands back in `catch`. The probes are assembly
//! loops, so the only frame abandoned is theirs, which holds nothing.

use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicU64, Ordering};

use thisos_hal::{Platform, Probe, TrapFrame as _};
use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::registers::control::{Cr4, Cr4Flags};

use crate::X86_64;
use crate::paging::USER_SPACE;
use crate::trap::TrapFrame;

/// The stack pointer `thisos_catch` saved the registers at while a catch
/// runs, and 0 otherwise.
static SAVED_RSP: AtomicU64 = AtomicU64::new(0);

// thisos_catch takes the probe's number in rdi and its target's address in
// rsi. It returns only through thisos_catch_landing, where abandon sends a
// trapped probe. The overflow probe pushes from thisos_catch's own frame
// down, so the registers saved above it survive. The two targets are in
// .rodata, which the loader maps read-only and no-execute, and Rust code
// never reads them, so a write that the CPU lets through changes nothing
// Rust relies on.
global_asm!(
    ".pushsection .rodata.thisos_probe, \"a\", @progbits",
    ".globl thisos_probe_read_only",
    ".hidden thisos_probe_read_only",
    "thisos_probe_read_only:",
    ".byte 0",
    ".globl thisos_probe_ret",
    ".hidden thisos_probe_ret",
    "thisos_probe_ret:",
    "ret",
    ".popsection",
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
    "cmp rdi, 3",
    "je 4f",
    "cmp rdi, 4",
    "je 5f",
    "cmp rdi, 5",
    "je 6f",
    "1:",
    "pause",
    "jmp 1b",
    "2:",
    "mov al, byte ptr [rsi]",
    "jmp 2b",
    "3:",
    "push rax",
    "jmp 3b",
    "4:",
    "mov al, byte ptr [rsi]",
    "mov byte ptr [rsi], al",
    "jmp 4b",
    "5:",
    "call rsi",
    "jmp 5b",
    "6:",
    "stac",
    "jmp 1b",
    ".globl thisos_catch_landing",
    ".hidden thisos_catch_landing",
    "thisos_catch_landing:",
    // iretq brought back the AC the probe ran with.
    "clac",
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
    static thisos_probe_read_only: u8;
    static thisos_probe_ret: u8;
}

/// Runs `probe` until a trap handler abandons it.
pub fn catch(probe: Probe) {
    assert_eq!(
        SAVED_RSP.load(Ordering::Relaxed),
        0,
        "a catch is running already"
    );
    let number: u64 = match probe {
        Probe::ReadGuardPage | Probe::ReadUser => 0,
        Probe::OverflowStack => 1,
        Probe::Spin => 2,
        Probe::WriteReadOnly => 3,
        Probe::ExecuteNoExecute | Probe::ExecuteUser => 4,
        Probe::SpinUserAccessOpen => 5,
    };
    if probe == Probe::ExecuteUser {
        assert!(
            Cr4::read().contains(Cr4Flags::SUPERVISOR_MODE_EXECUTION_PROTECTION),
            "ExecuteUser before SMEP is on"
        );
    }
    let run = || {
        // SAFETY: thisos_catch saves and restores every callee-saved
        // register, and returns, through thisos_catch_landing, to the
        // instruction after the call with the stack pointer it was called
        // with. The probes touch only rax, the stack below the saved
        // registers and their own targets: the guard page, which the loader
        // leaves unmapped, so the read only traps; stack that no frame above
        // uses; the read-only byte, written back with the value it holds;
        // the ret, which only returns; and the user probe page, which the
        // read can't change, and which the call can't run, since SMEP is on.
        // The spin with AC set reaches no memory, and the landing clears AC
        // again; trap::init checked that the CPU has stac and clac. The C
        // ABI clobbers cover rax and the flags.
        unsafe {
            asm!(
                "call thisos_catch",
                in("rdi") number,
                in("rsi") target(probe).unwrap_or(0),
                clobber_abi("C"),
            );
        }
    };
    match probe {
        // Interrupt handlers run on the stack the probe fills, so one that
        // came near the guard page would overflow there itself, and abandon
        // would give up the handler along with the probe.
        Probe::OverflowStack => interrupts::without_interrupts(run),
        Probe::ReadGuardPage
        | Probe::Spin
        | Probe::SpinUserAccessOpen
        | Probe::WriteReadOnly
        | Probe::ExecuteNoExecute
        | Probe::ReadUser
        | Probe::ExecuteUser => run(),
    }
}

/// The address whose page fault ends `probe`; see `Platform::probe_target`.
pub fn target(probe: Probe) -> Option<u64> {
    match probe {
        Probe::ReadGuardPage => Some(X86_64::guard_page()),
        Probe::WriteReadOnly => Some(address(&raw const thisos_probe_read_only)),
        Probe::ExecuteNoExecute => Some(address(&raw const thisos_probe_ret)),
        Probe::ReadUser | Probe::ExecuteUser => Some(USER_SPACE.start),
        Probe::OverflowStack | Probe::Spin | Probe::SpinUserAccessOpen => None,
    }
}

fn address(byte: *const u8) -> u64 {
    VirtAddr::from_ptr(byte).as_u64()
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
    address(&raw const thisos_catch_landing)
}
