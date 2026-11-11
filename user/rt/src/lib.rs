//! The runtime every ThisOS program links: its entry point, its syscalls
//! and its panic handler. A program names its main function with
//! [`entry!`].

#![no_std]

use core::arch::{asm, naked_asm};
use core::panic::PanicInfo;

use thisos_abi::Syscall;

/// Makes `main`, a `fn() -> u64`, the program's main function: the thread
/// ends through [`exit`] with the status it returns.
#[macro_export]
macro_rules! entry {
    ($main:path) => {
        // Inside an anonymous const, so the program can't name the exported
        // function and call it a second time.
        const _: () = {
            #[unsafe(export_name = "__thisos_main")]
            fn __thisos_main() -> u64 {
                let main: fn() -> u64 = $main;
                main()
            }
        };
    };
}

// Safe to call: only entry! defines the symbol, with this signature, since
// defining it any other way takes unsafe code, which programs forbid.
unsafe extern "Rust" {
    /// The function `entry!` exports.
    safe fn __thisos_main() -> u64;
}

/// Where the kernel starts the program, with a 16-byte aligned stack and
/// every register but RDI zeroed, RBP included, which ends a backtrace's
/// frame chain here. The call makes the stack aligned as the System V ABI
/// wants it at a function's entry.
#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn _start() -> ! {
    naked_asm!("call {start}", "ud2", start = sym start)
}

extern "C" fn start() -> ! {
    exit(__thisos_main())
}

/// Ends the calling thread with `status`.
pub fn exit(status: u64) -> ! {
    // SAFETY: By ADR 0007, exit reads only its number and argument
    // registers, touches no user memory and never returns.
    unsafe {
        asm!(
            "syscall",
            in("rax") const { Syscall::Exit.number() },
            in("rdi") status,
            options(noreturn, nostack),
        );
    }
}

/// Ends the thread with status 101, Rust's own exit code for a panic. The
/// message goes nowhere yet: there is no syscall to write it.
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    exit(101)
}
