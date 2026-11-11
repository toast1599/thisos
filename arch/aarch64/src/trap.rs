//! Traps: the vector table, the stubs that save the registers in a
//! [`TrapFrame`] and call the kernel's handler, and the overflow stack they
//! move to when the trapped code's stack has no room for the frame.

use core::arch::{asm, global_asm};
use core::cell::UnsafeCell;
use core::fmt;
use core::mem::offset_of;
use core::ops::{Range, RangeInclusive};
use core::sync::atomic::{AtomicU64, Ordering};

use thisos_hal::{Access, BacktraceEnd, Cpu, Trap, TrapHandler};

use crate::boot::STACK_SIZE;
use crate::gic::{self, Acknowledged, Group};
use crate::once::Once;
use crate::{Aarch64, backtrace, catch, not_ported, timer, to_u64};

// The vector table: 16 entries of 0x80 bytes, for a synchronous exception,
// an IRQ, an FIQ and an SError, taken from EL1 on SP_EL0, from EL1 on
// SP_EL1, from EL0 in AArch64 and from EL0 in AArch32. The .org fails to
// assemble if an entry outgrows its 0x80 bytes.
//
// The CPU pushes nothing, so each entry first makes room for a TrapFrame
// and checks, as Linux does, that it fit: every stack EL1 runs on is
// STACK_SIZE bytes with its start aligned to twice that, so bit
// STACK_SHIFT of the frame's address is clear on the stack and set for
// STACK_SIZE below it. The add and sub around tbnz test that bit with no
// free register, and put SP and x0 back after it.
//
// An entry that finds the stack overflowed parks the frame's address and
// x0 in the EL0 thread registers, which the trapped code never gets back
// since an overflow never resumes, moves to the overflow stack, and adds
// OVERFLOWED to its entry number. The common path saves the rest of the
// frame, with the trapped SP that x1 brings, calls trap_entry, and
// restores everything but SP, which it gets back by dropping the frame.
global_asm!(
    ".pushsection .text.thisos_vectors, \"ax\", %progbits",
    ".balign 2048",
    ".globl thisos_vectors",
    ".hidden thisos_vectors",
    "thisos_vectors:",
    ".set .Lentry, 0",
    ".rept 16",
    "sub sp, sp, #{frame}",
    "add sp, sp, x0",
    "sub x0, sp, x0",
    "tbnz x0, #{shift}, 1f",
    "sub x0, sp, x0",
    "sub sp, sp, x0",
    "stp x0, x1, [sp]",
    "mov x0, #.Lentry",
    "add x1, sp, #{frame}",
    "b .Ltrap_common",
    "1:",
    "msr tpidr_el0, x0",
    "sub x0, sp, x0",
    "msr tpidrro_el0, x0",
    "adrp x0, {top}",
    "ldr x0, [x0, :lo12:{top}]",
    "mov sp, x0",
    "mov x0, #(.Lentry + {overflowed})",
    "b .Ltrap_overflow",
    ".org thisos_vectors + 0x80 * (.Lentry + 1)",
    ".set .Lentry, .Lentry + 1",
    ".endr",
    ".Ltrap_overflow:",
    "sub sp, sp, #{frame}",
    "str x1, [sp, #8]",
    "mrs x1, tpidrro_el0",
    "str x1, [sp]",
    "mrs x1, tpidr_el0",
    "add x1, x1, #{frame}",
    ".Ltrap_common:",
    "stp x2, x3, [sp, #0x10]",
    "stp x4, x5, [sp, #0x20]",
    "stp x6, x7, [sp, #0x30]",
    "stp x8, x9, [sp, #0x40]",
    "stp x10, x11, [sp, #0x50]",
    "stp x12, x13, [sp, #0x60]",
    "stp x14, x15, [sp, #0x70]",
    "stp x16, x17, [sp, #0x80]",
    "stp x18, x19, [sp, #0x90]",
    "stp x20, x21, [sp, #0xa0]",
    "stp x22, x23, [sp, #0xb0]",
    "stp x24, x25, [sp, #0xc0]",
    "stp x26, x27, [sp, #0xd0]",
    "stp x28, x29, [sp, #0xe0]",
    "stp x30, x1, [sp, #0xf0]",
    "mrs x2, elr_el1",
    "mrs x3, spsr_el1",
    "stp x2, x3, [sp, #0x100]",
    "mrs x2, esr_el1",
    "mrs x3, far_el1",
    "stp x2, x3, [sp, #0x110]",
    "str x0, [sp, #0x120]",
    "mov x0, sp",
    "bl {entry}",
    "ldp x2, x3, [sp, #0x100]",
    "msr elr_el1, x2",
    "msr spsr_el1, x3",
    "ldp x2, x3, [sp, #0x10]",
    "ldp x4, x5, [sp, #0x20]",
    "ldp x6, x7, [sp, #0x30]",
    "ldp x8, x9, [sp, #0x40]",
    "ldp x10, x11, [sp, #0x50]",
    "ldp x12, x13, [sp, #0x60]",
    "ldp x14, x15, [sp, #0x70]",
    "ldp x16, x17, [sp, #0x80]",
    "ldp x18, x19, [sp, #0x90]",
    "ldp x20, x21, [sp, #0xa0]",
    "ldp x22, x23, [sp, #0xb0]",
    "ldp x24, x25, [sp, #0xc0]",
    "ldp x26, x27, [sp, #0xd0]",
    "ldp x28, x29, [sp, #0xe0]",
    "ldr x30, [sp, #0xf0]",
    "ldp x0, x1, [sp]",
    "add sp, sp, #{frame}",
    "eret",
    ".popsection",
    frame = const FRAME_SIZE,
    shift = const STACK_SIZE.trailing_zeros(),
    top = sym OVERFLOW_TOP,
    overflowed = const OVERFLOWED,
    entry = sym trap_entry,
);

unsafe extern "C" {
    static thisos_vectors: u8;
}

/// The registers when the CPU trapped, as the vector table's stubs save
/// them. For a trap from EL0, `sp` is still the kernel's.
#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame {
    x: [u64; 31],
    sp: u64,
    elr: u64,
    spsr: u64,
    esr: u64,
    far: u64,
    /// The vector table entry the CPU took, from 0 to 15, plus
    /// [`OVERFLOWED`] when it found the stack overflowed.
    entry: u64,
    _align: u64,
}

const FRAME_SIZE: usize = size_of::<TrapFrame>();

// The stubs' offsets, and the 16-byte stack alignment the C ABI wants.
const _: () = assert!(
    offset_of!(TrapFrame, sp) == 0xf8
        && offset_of!(TrapFrame, elr) == 0x100
        && offset_of!(TrapFrame, esr) == 0x110
        && offset_of!(TrapFrame, entry) == 0x120
        && FRAME_SIZE % 16 == 0
);

/// Added to a frame's entry number by an entry that found the stack
/// overflowed.
const OVERFLOWED: u64 = 0x10;

/// Each entry's place in its group of four.
const SYNC: u64 = 0;
const IRQ: u64 = 1;
const FIQ: u64 = 2;

/// The entries for traps from EL1 on `SP_EL1`, the only ones the kernel
/// resumes from.
const EL1H_SYNC: u64 = 4;
const EL1H_IRQ: u64 = 5;

/// The synchronous entries for traps from EL0.
const EL0_SYNC: u64 = 8;
const EL0_AARCH32_SYNC: u64 = 12;

const ENTRY_NAMES: [&str; 16] = [
    "EL1t sync",
    "EL1t IRQ",
    "EL1t FIQ",
    "EL1t SError",
    "EL1h sync",
    "EL1h IRQ",
    "EL1h FIQ",
    "EL1h SError",
    "EL0 sync",
    "EL0 IRQ",
    "EL0 FIQ",
    "EL0 SError",
    "EL0 AArch32 sync",
    "EL0 AArch32 IRQ",
    "EL0 AArch32 FIQ",
    "EL0 AArch32 SError",
];

/// `SPSR_EL1.M`, the mode `eret` returns to.
const SPSR_MODE: u64 = 0xf;
/// EL1 on `SP_EL1`.
const SPSR_EL1H: u64 = 0b0101;

impl thisos_hal::TrapFrame for TrapFrame {
    fn ip(&self) -> u64 {
        self.elr
    }

    fn sp(&self) -> u64 {
        self.sp
    }

    // The synchronous entries from EL0 take exceptions user code's own
    // instructions raise.
    fn raised_by_user(&self) -> bool {
        matches!(self.entry(), EL0_SYNC | EL0_AARCH32_SYNC)
    }

    fn syscall(&self) -> (u64, [u64; 6]) {
        not_ported("TrapFrame::syscall")
    }

    fn set_syscall_result(&mut self, _: u64, _: u64) {
        not_ported("TrapFrame::set_syscall_result")
    }

    fn backtrace(&self, mut f: impl FnMut(u64)) -> BacktraceEnd {
        f(self.elr);
        backtrace::walk(self.x[29], f)
    }

    fn abandon(&mut self) -> bool {
        catch::abandon(self)
    }
}

impl TrapFrame {
    /// Makes `eret` continue at `pc` at EL1 on `SP_EL1`, in place of the
    /// trapped code, with its interrupt masks. `eret` leaves SP where the
    /// stub dropped the frame, so the code at `pc` sets its own.
    pub(crate) fn resume_at(&mut self, pc: u64) {
        self.elr = pc;
        self.spsr = (self.spsr & !SPSR_MODE) | SPSR_EL1H;
    }

    fn entry(&self) -> u64 {
        self.entry & !OVERFLOWED
    }

    fn overflowed(&self) -> bool {
        self.entry & OVERFLOWED != 0
    }
}

impl fmt::Display for TrapFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = usize::try_from(self.entry())
            .ok()
            .and_then(|entry| ENTRY_NAMES.get(entry))
            .unwrap_or(&"");
        write!(f, "registers at {name}")?;
        if self.overflowed() {
            write!(f, " on an overflowed stack")?;
        }
        // The CPU writes ESR and FAR only for synchronous exceptions and
        // SErrors, so for an interrupt they hold an older trap's.
        if !matches!(self.entry() % 4, IRQ | FIQ) {
            write!(f, ", ESR {:#x}, FAR {:#x}", self.esr, self.far)?;
        }
        writeln!(f, ":")?;
        let mut registers = [("", 0); 34];
        registers[..3].copy_from_slice(&[("pc", self.elr), ("sp", self.sp), ("spsr", self.spsr)]);
        for (slot, register) in registers[3..]
            .iter_mut()
            .zip(X_NAMES.into_iter().zip(self.x))
        {
            *slot = register;
        }
        for row in registers.chunks(4) {
            write!(f, " ")?;
            for (name, value) in row {
                write!(f, " {name:>4} {value:#018x}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

const X_NAMES: [&str; 31] = [
    "x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9", "x10", "x11", "x12", "x13", "x14",
    "x15", "x16", "x17", "x18", "x19", "x20", "x21", "x22", "x23", "x24", "x25", "x26", "x27",
    "x28", "x29", "x30",
];

// ESR_EL1 exception classes.
const EC_INSTRUCTION_ABORT_EL0: u8 = 0x20;
const EC_INSTRUCTION_ABORT_EL1: u8 = 0x21;
const EC_DATA_ABORT_EL0: u8 = 0x24;
const EC_DATA_ABORT_EL1: u8 = 0x25;
const EC_BRK: u8 = 0x3c;

// ESR_EL1 syndrome bits of an abort.
/// FAR doesn't hold the faulting address.
const FNV: u64 = 1 << 10;
/// A data abort was on a write.
const WNR: u64 = 1 << 6;
const FSC: u64 = 0x3f;
/// The fault status codes for translation, access flag and permission
/// faults, at every level.
const PAGE_TABLE_FAULTS: RangeInclusive<u64> = 0x04..=0x0f;

/// The exception class in `esr`.
fn class(esr: u64) -> u8 {
    // Six bits, so the low byte is all of it.
    ((esr >> 26) & 0x3f).to_le_bytes()[0]
}

/// The trap a synchronous exception is. A breakpoint moves the frame past
/// its `brk`, which the CPU leaves ELR at.
fn sync(frame: &mut TrapFrame) -> Trap {
    let page_table_fault = frame.esr & FNV == 0 && PAGE_TABLE_FAULTS.contains(&(frame.esr & FSC));
    let page_fault = |access| Trap::PageFault {
        address: frame.far,
        access,
    };
    match class(frame.esr) {
        EC_BRK => {
            frame.elr = frame.elr.wrapping_add(4);
            Trap::Breakpoint
        }
        EC_INSTRUCTION_ABORT_EL0 | EC_INSTRUCTION_ABORT_EL1 if page_table_fault => {
            page_fault(Access::Execute)
        }
        EC_DATA_ABORT_EL0 | EC_DATA_ABORT_EL1 if page_table_fault => {
            page_fault(if frame.esr & WNR != 0 {
                Access::Write
            } else {
                Access::Read
            })
        }
        vector => Trap::Other {
            vector,
            error: frame.esr,
        },
    }
}

extern "C" fn trap_entry(frame: &mut TrapFrame) {
    // init stores the handler before it sets VBAR_EL1, the only way here.
    let Some(handler) = HANDLER.get() else {
        Aarch64::halt();
    };
    // An overflow of the overflow stack: the stub put this frame over the
    // frames of the handler that overflowed it, so nothing is left to
    // report from.
    if frame.overflowed() && on_overflow_stack(frame.sp) {
        Aarch64::halt();
    }
    let mut acknowledged = None;
    let trap = if frame.overflowed() {
        Trap::DoubleFault
    } else {
        match frame.entry() % 4 {
            SYNC => sync(frame),
            kind @ (IRQ | FIQ) => {
                let group = if kind == IRQ { Group::One } else { Group::Zero };
                let (trap, interrupt) = interrupt(group);
                acknowledged = interrupt;
                trap
            }
            _ => Trap::Other {
                vector: class(frame.esr),
                error: frame.esr,
            },
        }
    };
    handler(trap, frame);
    // After the handler, as x86_64's EOI, so that the next tick can't come
    // while this one is still handled. An abandoned handler ends it too.
    if let Some(interrupt) = acknowledged {
        interrupt.end();
    }
    let resumes = if frame.overflowed() {
        // An overflow the handler abandoned resumes in the catch, from
        // registers it saved itself, not from the state the overflow left.
        catch::abandoned(frame)
    } else {
        matches!(frame.entry(), EL1H_SYNC | EL1H_IRQ)
    };
    if !resumes {
        Aarch64::halt();
    }
}

/// Acknowledges the interrupt in `group` that the CPU took, and returns
/// the trap it is, with the interrupt to end after the handler: none for a
/// special interrupt ID, such as 1023 for one that went away. The timer's
/// next tick is set up first, which lowers its interrupt.
fn interrupt(group: Group) -> (Trap, Option<Acknowledged>) {
    match gic::acknowledge(group) {
        Ok(interrupt) if matches!(group, Group::One) && timer::is_timer(interrupt.intid()) => {
            timer::rearm();
            (Trap::Timer, Some(interrupt))
        }
        Ok(interrupt) => (
            Trap::Stray {
                vector: interrupt.intid(),
            },
            Some(interrupt),
        ),
        Err(special) => (Trap::Stray { vector: special }, None),
    }
}

static HANDLER: Once<TrapHandler<Aarch64>> = Once::new();

/// The top of the stack the vector table's stubs move to when they find
/// the stack overflowed, which they read without a lock.
static OVERFLOW_TOP: AtomicU64 = AtomicU64::new(0);

/// Whether `sp` is on the overflow stack or within a stack's size below it.
fn on_overflow_stack(sp: u64) -> bool {
    let top = OVERFLOW_TOP.load(Ordering::Relaxed);
    top.wrapping_sub(sp) <= 2 * to_u64(STACK_SIZE)
}

/// The overflow stack until `move_stack` replaces it, with nothing unmapped
/// below it. Aligned to twice its size, as every stack EL1 runs on, so that
/// a trap taken on it passes the stubs' check. Rust code never reads or
/// writes its bytes; the stubs do.
#[repr(C, align(0x20000))]
struct Stack(UnsafeCell<[u8; STACK_SIZE]>);

const _: () = assert!(align_of::<Stack>() == 2 * STACK_SIZE);

// SAFETY: Rust code only takes the stack's address. The stubs use the
// bytes, and SMP is a non-goal, so one CPU runs on them at a time.
unsafe impl Sync for Stack {}

static BOOT_OVERFLOW_STACK: Stack = Stack(UnsafeCell::new([0; STACK_SIZE]));

/// Points `VBAR_EL1` at the vector table, with `handler` as the kernel's
/// trap handler.
///
/// # Safety
///
/// Call this once, before anything traps.
pub unsafe fn init(handler: TrapHandler<Aarch64>) {
    HANDLER.init(handler);
    let top = BOOT_OVERFLOW_STACK.0.get().wrapping_add(1);
    OVERFLOW_TOP.store(to_u64(top.addr()), Ordering::Relaxed);
    // SAFETY: thisos_vectors is the vector table above, 2 KiB aligned in
    // .text, which every page table the kernel runs on maps executable.
    // Its stubs save and restore every register they change, and the isb
    // makes the CPU take the next trap through it.
    unsafe {
        asm!(
            "msr vbar_el1, {}",
            "isb",
            in(reg) &raw const thisos_vectors,
            options(nostack, preserves_flags),
        );
    }
}

/// Moves the overflow stack to one `stack` maps: `stack(size)` returns the
/// address just past a stack of `size` bytes with an unmapped page below
/// it, aligned as every stack EL1 runs on.
pub fn move_stack(stack: &mut dyn FnMut(usize) -> u64) {
    let top = stack(STACK_SIZE);
    let size = to_u64(STACK_SIZE);
    assert_eq!(
        (top - size) % (2 * size),
        0,
        "overflow stack at {top:#x} isn't aligned to twice its size"
    );
    // The stubs read it on this core, which sees its own stores in order,
    // and the boot overflow stack stays valid, so a trap meanwhile takes
    // either stack.
    OVERFLOW_TOP.store(top, Ordering::Relaxed);
}

/// The overflow stack.
pub fn stacks() -> impl Iterator<Item = Range<u64>> {
    let top = OVERFLOW_TOP.load(Ordering::Relaxed);
    core::iter::once(top - to_u64(STACK_SIZE)..top)
}
