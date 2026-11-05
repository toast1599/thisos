//! Traps: the GDT and the TSS, which give the CPU stacks to take some traps
//! on, the IDT, and the stubs that save the registers in a [`TrapFrame`]
//! and call the kernel's handler.

use core::arch::global_asm;
use core::cell::UnsafeCell;
use core::fmt;

use thisos_hal::{Access, BacktraceEnd, Cpu, Trap, TrapHandler};
use x86_64::VirtAddr;
use x86_64::instructions::segmentation::{CS, SS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::registers::control::Cr2;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::idt::{Entry, EntryOptions, InterruptDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;

use crate::once::Once;
use crate::{X86_64, backtrace};

// Each stub sits 16 bytes after the one before it, so that the IDT can find
// stub n at thisos_trap_stubs + 16n. The .org fails to assemble if a stub
// outgrows its 16 bytes. A stub pushes a zero in place of the error code
// for the vectors whose error code the CPU doesn't push, so that every
// trap leaves the same frame, then pushes the vector. The common path
// pushes the general registers to complete the TrapFrame, calls trap_entry
// with it, and undoes all of that. The CPU aligns the stack to 16 bytes
// before it pushes its five words, so after the stub's two and the fifteen
// registers it is aligned again for the call, as the C ABI wants.
global_asm!(
    ".pushsection .text.thisos_trap, \"ax\", @progbits",
    ".p2align 4",
    ".globl thisos_trap_stubs",
    ".hidden thisos_trap_stubs",
    "thisos_trap_stubs:",
    ".irp v, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31",
    ".if !(\\v == 8 || (\\v >= 10 && \\v <= 14) || \\v == 17 || \\v == 21 || \\v == 29 || \\v == 30)",
    "push 0",
    ".endif",
    "push \\v",
    "jmp .Ltrap_common",
    ".org thisos_trap_stubs + 16 * (\\v + 1), 0xcc",
    ".endr",
    ".Ltrap_common:",
    "push rax",
    "push rbx",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push rbp",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "push r12",
    "push r13",
    "push r14",
    "push r15",
    "mov rdi, rsp",
    // The C ABI wants the direction flag clear, and a trap can come while
    // it is set.
    "cld",
    "call {entry}",
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rbp",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rbx",
    "pop rax",
    "add rsp, 16",
    "iretq",
    ".popsection",
    entry = sym trap_entry,
);

unsafe extern "C" {
    static thisos_trap_stubs: [u8; 32 * 16];
}

/// The registers when the CPU trapped, as the stubs and the CPU push them.
#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    vector: u64,
    error: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

impl thisos_hal::TrapFrame for TrapFrame {
    fn ip(&self) -> u64 {
        self.rip
    }

    fn sp(&self) -> u64 {
        self.rsp
    }

    fn backtrace(&self, mut f: impl FnMut(u64)) -> BacktraceEnd {
        f(backtrace::link_address(self.rip));
        backtrace::walk(self.rbp, f)
    }
}

/// The exceptions' mnemonics, by vector, with "" for the reserved ones.
const MNEMONICS: [&str; 32] = [
    "#DE", "#DB", "NMI", "#BP", "#OF", "#BR", "#UD", "#NM", "#DF", "", "#TS", "#NP", "#SS", "#GP",
    "#PF", "", "#MF", "#AC", "#MC", "#XM", "#VE", "#CP", "", "", "", "", "", "", "#HV", "#VC",
    "#SX", "",
];

impl fmt::Display for TrapFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mnemonic = usize::try_from(self.vector)
            .ok()
            .and_then(|vector| MNEMONICS.get(vector))
            .unwrap_or(&"");
        writeln!(
            f,
            "registers at vector {} {mnemonic}, error code {:#x}:",
            self.vector, self.error
        )?;
        let registers = [
            ("rip", self.rip),
            ("rsp", self.rsp),
            ("rflags", self.rflags),
            ("cs", self.cs),
            ("ss", self.ss),
            ("rax", self.rax),
            ("rbx", self.rbx),
            ("rcx", self.rcx),
            ("rdx", self.rdx),
            ("rsi", self.rsi),
            ("rdi", self.rdi),
            ("rbp", self.rbp),
            ("r8", self.r8),
            ("r9", self.r9),
            ("r10", self.r10),
            ("r11", self.r11),
            ("r12", self.r12),
            ("r13", self.r13),
            ("r14", self.r14),
            ("r15", self.r15),
        ];
        for row in registers.chunks(4) {
            write!(f, " ")?;
            for (name, value) in row {
                write!(f, " {name:>6} {value:#018x}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// The vectors that can't resume after their handler returns.
const DOUBLE_FAULT: u64 = 8;
const MACHINE_CHECK: u64 = 18;

extern "C" fn trap_entry(frame: &mut TrapFrame) {
    let trap = match frame.vector {
        3 => Trap::Breakpoint,
        DOUBLE_FAULT => Trap::DoubleFault,
        14 => {
            let (write, fetch) = (1 << 1, 1 << 4);
            let access = if frame.error & fetch != 0 {
                Access::Execute
            } else if frame.error & write != 0 {
                Access::Write
            } else {
                Access::Read
            };
            Trap::PageFault {
                address: Cr2::read_raw(),
                access,
            }
        }
        vector => Trap::Other {
            // The stubs push only vectors below 32.
            vector: vector.to_le_bytes()[0],
            error: frame.error,
        },
    };
    // init stores the handler before it loads the IDT, the only way here.
    let Some(handler) = HANDLER.get() else {
        X86_64::halt();
    };
    handler(trap, frame);
    if frame.vector == DOUBLE_FAULT || frame.vector == MACHINE_CHECK {
        X86_64::halt();
    }
}

static HANDLER: Once<TrapHandler<X86_64>> = Once::new();

/// The IST slots, counted from 0 as the x86_64 crate does, each the index
/// of its stack in `IST_STACKS`.
const DOUBLE_FAULT_IST: u16 = 0;
const NMI_IST: u16 = 1;
const MACHINE_CHECK_IST: u16 = 2;

/// A stack the CPU switches to for one vector. Rust code never reads or
/// writes its bytes; the CPU does, through the TSS.
#[repr(align(16))]
struct Stack(UnsafeCell<[u8; 20 * 1024]>);

// SAFETY: Rust code only takes the stack's address, which is the same on
// every thread. The CPU uses the bytes for the one vector whose IST slot
// holds them, and SMP is a non-goal, so two traps never share one.
unsafe impl Sync for Stack {}

impl Stack {
    /// The address just past the stack, where a stack that grows down
    /// starts.
    fn top(&self) -> VirtAddr {
        VirtAddr::from_ptr(self.0.get().wrapping_add(1))
    }
}

static IST_STACKS: [Stack; 3] = [const { Stack(UnsafeCell::new([0; 20 * 1024])) }; 3];

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<GlobalDescriptorTable> = Once::new();
static IDT: Once<InterruptDescriptorTable> = Once::new();

/// Loads the GDT, the TSS and the IDT, with `handler` as the kernel's trap
/// handler.
///
/// # Safety
///
/// Call this once, before anything else uses the segment registers or the
/// descriptor tables, and before anything traps.
pub unsafe fn init(handler: TrapHandler<X86_64>) {
    HANDLER.init(handler);

    let mut tss = TaskStateSegment::new();
    for slot in [DOUBLE_FAULT_IST, NMI_IST, MACHINE_CHECK_IST] {
        let slot = usize::from(slot);
        tss.interrupt_stack_table[slot] = IST_STACKS[slot].top();
    }
    // The CPU never writes a 64-bit TSS.
    let tss = TSS.init(tss);

    let mut gdt = GlobalDescriptorTable::new();
    let code = gdt.append(Descriptor::kernel_code_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(tss));
    // The CPU sets the TSS descriptor's busy bit when load_tss runs, but the
    // x86_64 crate keeps each entry in an atomic, so that write doesn't
    // break the shared reference's rules.
    let gdt = GDT.init(gdt);
    gdt.load();
    // SAFETY: code selects the 64-bit kernel code segment of the GDT just
    // loaded, which is what the kernel already runs as, only in the
    // loader's GDT.
    unsafe {
        CS::set_reg(code);
    }
    // SAFETY: Ring 0 may run with a null SS in 64-bit mode. The loader's
    // selector would index this GDT, where it may name the TSS, and iretq
    // reloads SS from the trap frame.
    unsafe {
        SS::set_reg(SegmentSelector::NULL);
    }
    // SAFETY: tss_selector selects the TSS descriptor of the GDT just
    // loaded, for a TSS that lives for the rest of the kernel's life.
    unsafe {
        load_tss(tss_selector);
    }

    let mut idt = InterruptDescriptorTable::new();
    install(&mut idt.divide_error, 0);
    install(&mut idt.debug, 1);
    install_on(&mut idt.non_maskable_interrupt, 2, NMI_IST);
    install(&mut idt.breakpoint, 3);
    install(&mut idt.overflow, 4);
    install(&mut idt.bound_range_exceeded, 5);
    install(&mut idt.invalid_opcode, 6);
    install(&mut idt.device_not_available, 7);
    install_on(&mut idt.double_fault, 8, DOUBLE_FAULT_IST);
    install(&mut idt.invalid_tss, 10);
    install(&mut idt.segment_not_present, 11);
    install(&mut idt.stack_segment_fault, 12);
    install(&mut idt.general_protection_fault, 13);
    install(&mut idt.page_fault, 14);
    install(&mut idt.x87_floating_point, 16);
    install(&mut idt.alignment_check, 17);
    install_on(&mut idt.machine_check, 18, MACHINE_CHECK_IST);
    install(&mut idt.simd_floating_point, 19);
    install(&mut idt.virtualization, 20);
    install(&mut idt.cp_protection_exception, 21);
    install(&mut idt.hv_injection_exception, 28);
    install(&mut idt.vmm_communication_exception, 29);
    install(&mut idt.security_exception, 30);
    IDT.init(idt).load();
}

/// Points `entry`, the IDT entry for `vector`, at that vector's stub.
fn install<F>(entry: &mut Entry<F>, vector: u8) -> &mut EntryOptions {
    let stub = VirtAddr::from_ptr(&raw const thisos_trap_stubs) + 16 * u64::from(vector);
    // SAFETY: stub is the stub for vector, which saves and restores every
    // register it changes and returns with iretq after popping the error
    // code the CPU pushes for vector, or the zero it pushed in place of
    // one. The caller passes the vector its entry is for.
    unsafe { entry.set_handler_addr(stub) }
}

/// As `install`, with the CPU switching to stack `ist` before the stub runs.
fn install_on<F>(entry: &mut Entry<F>, vector: u8, ist: u16) {
    let options = install(entry, vector);
    // SAFETY: ist is one of the slots init fills with a stack of its own in
    // IST_STACKS, and init passes each to one vector only, so no two traps
    // share one.
    unsafe {
        options.set_stack_index(ist);
    }
}
