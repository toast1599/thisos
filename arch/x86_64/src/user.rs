//! Entering user code, and the programs tests run there.

use core::arch::{asm, global_asm};
use core::ptr;

use thisos_abi::Syscall;
use thisos_frames::Frame;
use thisos_hal::UserProgram;
use x86_64::VirtAddr;

use crate::paging::USER_SPACE;
use crate::trap::{USER_CODE, USER_DATA};

/// Leaves for user code; see `Cpu::enter_user`.
pub fn enter_user(ip: u64, sp: u64, arg: u64) -> ! {
    assert!(
        USER_SPACE.contains(&ip) && (USER_SPACE.start..=USER_SPACE.end).contains(&sp),
        "entering user code at {ip:#x} with stack pointer {sp:#x}, outside user space {USER_SPACE:#x?}"
    );
    // SAFETY: iretq takes the frame pushed here to ring 3 at ip, on stack
    // sp, both in user space, which the assert checks, with RFLAGS holding
    // only IF and the always-set bit 1, so AC and TF are clear. The frame
    // is on the calling thread's own stack, whose top TSS.RSP0 holds, and
    // this never returns, so nothing on that stack is used again: the next
    // trap from ring 3 starts at its top. Every general register but rdi,
    // which holds arg, is cleared, so no kernel value reaches user code.
    unsafe {
        asm!(
            "push {ss}",
            "push rsi",
            "push {rflags}",
            "push {cs}",
            "push rdx",
            "xor eax, eax",
            "xor ebx, ebx",
            "xor ecx, ecx",
            "xor edx, edx",
            "xor esi, esi",
            "xor ebp, ebp",
            "xor r8d, r8d",
            "xor r9d, r9d",
            "xor r10d, r10d",
            "xor r11d, r11d",
            "xor r12d, r12d",
            "xor r13d, r13d",
            "xor r14d, r14d",
            "xor r15d, r15d",
            "iretq",
            ss = const USER_DATA,
            rflags = const 0x202,
            cs = const USER_CODE,
            in("rdi") arg,
            in("rsi") sp,
            in("rdx") ip,
            options(noreturn),
        );
    }
}

/// The bytes each program has, from the start of its slot.
const SLOT: usize = 256;

// One program per slot, in UserProgram's order, as write_program numbers
// them. The .org fails to assemble if a program outgrows its slot, and
// pads it with int3. Every jump is relative, so a program runs wherever it
// is mapped. The programs are in .rodata, which the loader maps no-execute,
// so the kernel never runs them; it only copies them.
global_asm!(
    ".pushsection .rodata.thisos_user_programs, \"a\", @progbits",
    ".p2align 4",
    ".globl thisos_user_programs",
    ".hidden thisos_user_programs",
    "thisos_user_programs:",
    // Syscalls. Every value is negative, so its upper half is all ones,
    // and a register zeroed or truncated on the way shows.
    "mov rbx, -0x01010101",
    "mov rbp, -0x02020202",
    "mov rsi, -0x03030303",
    "mov rdi, -0x04040404",
    "mov r8, -0x05050505",
    "mov r9, -0x06060606",
    "mov r10, -0x07070707",
    "mov r12, -0x08080808",
    "mov r13, -0x09090909",
    "mov r14, -0x0a0a0a0a",
    "mov r15, -0x0b0b0b0b",
    "mov rax, -1",
    "syscall",
    "test rdx, rdx",
    "jnz 2f",
    "cmp rbx, -0x01010101",
    "jne 2f",
    "cmp rbp, -0x02020202",
    "jne 2f",
    "cmp rsi, -0x03030303",
    "jne 2f",
    "cmp rdi, -0x04040404",
    "jne 2f",
    "cmp r8, -0x05050505",
    "jne 2f",
    "cmp r9, -0x06060606",
    "jne 2f",
    "cmp r10, -0x07070707",
    "jne 2f",
    "cmp r12, -0x08080808",
    "jne 2f",
    "cmp r13, -0x09090909",
    "jne 2f",
    "cmp r14, -0x0a0a0a0a",
    "jne 2f",
    "cmp r15, -0x0b0b0b0b",
    "jne 2f",
    "mov rdi, rax",
    "jmp 3f",
    "2:",
    "mov rdi, -1",
    "3:",
    "mov eax, {exit}",
    "syscall",
    "ud2",
    ".org thisos_user_programs + 1 * {slot}, 0xcc",
    // Read.
    "mov al, byte ptr [rdi]",
    "xor edi, edi",
    "mov eax, {exit}",
    "syscall",
    "ud2",
    ".org thisos_user_programs + 2 * {slot}, 0xcc",
    // Privileged.
    "hlt",
    ".org thisos_user_programs + 3 * {slot}, 0xcc",
    // Simd.
    "xorps xmm0, xmm0",
    ".org thisos_user_programs + 4 * {slot}, 0xcc",
    // Spin.
    "rdtsc",
    "shl rdx, 32",
    "or rax, rdx",
    "mov rbx, rax",
    "2:",
    "pause",
    "rdtsc",
    "shl rdx, 32",
    "or rax, rdx",
    "sub rax, rbx",
    "cmp rax, {cycles}",
    "jb 2b",
    "xor edi, edi",
    "mov eax, {exit}",
    "syscall",
    "ud2",
    ".org thisos_user_programs + 5 * {slot}, 0xcc",
    // Write. The syscall keeps RSI, the length.
    "mov rsi, qword ptr [rdi + 8]",
    "mov rdi, qword ptr [rdi]",
    "mov eax, {write}",
    "syscall",
    "test rax, rax",
    "jnz 2f",
    "cmp rdx, rsi",
    "je 2f",
    "mov rax, -1",
    "2:",
    "mov rdi, rax",
    "mov eax, {exit}",
    "syscall",
    "ud2",
    ".org thisos_user_programs + 6 * {slot}, 0xcc",
    // Debug. ICEBP, as a byte since assemblers spell it differently.
    ".byte 0xf1",
    ".org thisos_user_programs + 7 * {slot}, 0xcc",
    ".popsection",
    exit = const Syscall::Exit.number(),
    write = const Syscall::Write.number(),
    slot = const SLOT,
    cycles = const 1 << 28,
);

unsafe extern "C" {
    static thisos_user_programs: [u8; 7 * SLOT];
}

/// Writes `program` at the start of `frame`, through the physical-memory
/// mapping at `phys_offset`.
pub fn write_program(phys_offset: VirtAddr, frame: &Frame, program: UserProgram) {
    let slot = match program {
        UserProgram::Syscalls => 0,
        UserProgram::Read => 1,
        UserProgram::Privileged => 2,
        UserProgram::Simd => 3,
        UserProgram::Spin => 4,
        UserProgram::Write => 5,
        UserProgram::Debug => 6,
    };
    let source = (&raw const thisos_user_programs)
        .cast::<u8>()
        .wrapping_add(slot * SLOT);
    // A physical address, reached through the physical-memory mapping.
    let target = (phys_offset + frame.start()).as_mut_ptr::<u8>();
    // SAFETY: source is one slot of thisos_user_programs, which nothing
    // writes. The caller holds the frame, and page tables hold every frame
    // they map, so nothing maps it but the physical-memory mapping, and no
    // Rust reference to its memory exists. A slot is smaller than a frame.
    unsafe { ptr::copy_nonoverlapping(source, target, SLOT) };
}
