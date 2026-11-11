//! Kernel threads' contexts, and switching between them. A suspended
//! thread's callee-saved registers sit on its own stack, under the address
//! its switch returns to, and its context holds the stack pointer to them.

use core::arch::{asm, global_asm};
use core::mem::offset_of;
use core::ops::Range;

use x86_64::VirtAddr;
use x86_64::instructions::interrupts;

use crate::{backtrace, paging, trap};

/// A suspended thread: the stack pointer its registers were pushed at,
/// and the stack it runs on.
#[derive(Debug)]
pub struct Context {
    rsp: u64,
    stack: Range<u64>,
}

// thisos_switch takes the stack pointer to resume in rdi, and returns the
// one it left in rax, in the thread it resumes. It leaves rsi and rdx
// alone, so the thread that resumes finds in them what the one that left
// put there: its stack. A new thread's stack holds a Start, so the switch
// pops entry into r12 and zero into rbp, the end of the frame-pointer
// chain, and returns into thisos_thread_start, which calls start with the
// stack 16-aligned.
global_asm!(
    ".pushsection .text.thisos_switch, \"ax\", @progbits",
    ".globl thisos_switch",
    ".hidden thisos_switch",
    "thisos_switch:",
    "push rbp",
    "push rbx",
    "push r12",
    "push r13",
    "push r14",
    "push r15",
    "mov rax, rsp",
    "mov rsp, rdi",
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop rbx",
    "pop rbp",
    "ret",
    ".globl thisos_thread_start",
    ".hidden thisos_thread_start",
    "thisos_thread_start:",
    "mov rdi, rax",
    "mov rcx, r12",
    "call {start}",
    "ud2",
    ".popsection",
    start = sym start,
);

unsafe extern "C" {
    fn thisos_thread_start();
}

/// What `new` writes at the top of a new thread's stack: the registers
/// `thisos_switch` pops, in the order it pops them, its return address,
/// and two words that make the stack 16-aligned at `call start`.
#[repr(C)]
struct Start {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: fn(Context) -> !,
    rbx: u64,
    rbp: u64,
    ret: unsafe extern "C" fn(),
    pad: [u64; 2],
}

// After the switch's ret, rsp is the stack's page-aligned end less this.
const _: () = assert!((size_of::<Start>() - offset_of!(Start, pad)) % 16 == 0);

/// A thread that runs `entry` on `stack` once switched to; see
/// `Platform::new_context`.
pub fn new(stack: Range<u64>, entry: fn(Context) -> !) -> Context {
    let Some(space) = paging::KERNEL_SPACE.get() else {
        panic!("a thread's stack before the kernel's page tables");
    };
    assert!(
        stack.start % 4096 == 0
            && stack.end % 4096 == 0
            && stack.start < stack.end
            && space.start <= stack.start
            && stack.end <= space.end,
        "{stack:#x?} isn't whole pages inside kernel space {space:#x?}"
    );
    // The address comes from a page the arch crate mapped in the reserved
    // level 4 entry, for the caller.
    let top = VirtAddr::new(stack.end)
        .as_mut_ptr::<Start>()
        .wrapping_sub(1);
    let start = Start {
        r15: 0,
        r14: 0,
        r13: 0,
        r12: entry,
        rbx: 0,
        rbp: 0,
        ret: thisos_thread_start,
        pad: [0; 2],
    };
    // SAFETY: The caller hands over a stack in kernel space, which the
    // assert checks, that it mapped writable for this thread alone, as
    // Platform::new_context asks, so the Start at its top is mapped, and
    // nothing else reads or writes it. The stack's end is page-aligned, so
    // top is aligned for a Start.
    unsafe {
        top.write(start);
    }
    Context {
        rsp: VirtAddr::from_ptr(top).as_u64(),
        stack,
    }
}

/// Suspends the running thread and resumes `to`; see `Platform::switch`.
pub fn switch(to: Context) -> Context {
    assert!(
        !interrupts::are_enabled(),
        "switching threads with interrupts on"
    );
    let from = backtrace::stack();
    trap::set_kernel_stack(to.stack.end);
    backtrace::set_stack(to.stack);
    let (rsp, start, end): (u64, u64, u64);
    // SAFETY: to came from new or from a switch, which consumed it, so
    // to.rsp points at a Start or at the registers a switch pushed, on a
    // stack nothing has run on since, and this is the only resumption.
    // thisos_switch pushes the callee-saved registers on this thread's
    // stack, and returns here only when a later switch resumes it, with
    // those registers and rsp as they were; the C ABI clobbers cover the
    // rest. Interrupts are off, so no handler runs on either stack between
    // the two halves of the switch.
    unsafe {
        asm!(
            "call thisos_switch",
            in("rdi") to.rsp,
            inout("rsi") from.start => start,
            inout("rdx") from.end => end,
            lateout("rax") rsp,
            clobber_abi("C"),
        );
    }
    Context {
        rsp,
        stack: start..end,
    }
}

/// A new thread's first Rust frame, which `thisos_thread_start` calls with
/// what the switch that resumed it left: the context of the thread that
/// switched to it, and the `entry` that `new` put in r12.
#[expect(
    improper_ctypes_definitions,
    reason = "only thisos_thread_start calls it, passing on the Rust fn pointer new stored"
)]
extern "C" fn start(rsp: u64, stack_start: u64, stack_end: u64, entry: fn(Context) -> !) -> ! {
    entry(Context {
        rsp,
        stack: stack_start..stack_end,
    })
}
