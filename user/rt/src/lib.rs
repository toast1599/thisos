//! The runtime every ThisOS program links: its entry point, its syscalls,
//! [`print!`] and [`println!`], and its panic handler. A program names its
//! main function with [`entry!`].

#![no_std]

use core::arch::{asm, naked_asm};
use core::fmt::{self, Write as _};
use core::panic::PanicInfo;

use thisos_abi::{Error, MAX_WRITE, Syscall};

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

/// Logs `text` as one line of the kernel's log, without a newline at its
/// end. Fails with [`Error::TooLong`] for more than [`MAX_WRITE`] bytes.
pub fn write(text: &str) -> Result<(), Error> {
    match write_bytes(text.as_bytes()) {
        0 => Ok(()),
        code => Err(error(code)),
    }
}

/// Makes the write syscall with `bytes`, and returns its error code.
fn write_bytes(bytes: &[u8]) -> u64 {
    let error;
    // SAFETY: By ADR 0007, write reads the bytes.len() bytes at
    // bytes.as_ptr(), which bytes holds, writes no user memory, and changes
    // only RAX, RDX, RCX and R11.
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") Syscall::Write.number() => error,
            in("rdi") bytes.as_ptr(),
            in("rsi") bytes.len(),
            lateout("rdx") _,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack, readonly),
        );
    }
    error
}

/// The error with `code`, which the kernel returned.
fn error(code: u64) -> Error {
    let Some(error) = Error::from_code(code) else {
        panic!("the kernel returned unknown error code {code}");
    };
    error
}

/// Prints to the kernel's log, as `format!` formats: one line per call,
/// or more for text longer than [`MAX_WRITE`] bytes, which is split
/// between characters. A newline at the end of a line is dropped.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::print_fmt(format_args!($($arg)*))
    };
}

/// Prints as [`print!`] does, with a newline added.
#[macro_export]
macro_rules! println {
    () => {
        $crate::print_fmt(format_args!("\n"))
    };
    ($($arg:tt)*) => {
        $crate::print_fmt(format_args!("{}\n", format_args!($($arg)*)))
    };
}

/// What [`print!`] calls. Panics if a write fails.
#[doc(hidden)]
pub fn print_fmt(args: fmt::Arguments) {
    let mut print = Print(Buffer::new());
    // Print's write_str never fails, so only a Display impl can.
    assert!(print.write_fmt(args).is_ok(), "formatting failed");
    print.flush();
}

/// Up to [`MAX_WRITE`] bytes of text, cut only between characters.
struct Buffer {
    bytes: [u8; MAX_WRITE],
    len: usize,
}

impl Buffer {
    fn new() -> Self {
        Buffer {
            bytes: [0; MAX_WRITE],
            len: 0,
        }
    }

    /// Appends as many whole characters of `text` as fit, and returns the
    /// rest.
    fn push<'a>(&mut self, text: &'a str) -> &'a str {
        let mut n = (MAX_WRITE - self.len).min(text.len());
        while !text.is_char_boundary(n) {
            n -= 1;
        }
        let (head, rest) = text.split_at(n);
        self.bytes[self.len..self.len + n].copy_from_slice(head.as_bytes());
        self.len += n;
        rest
    }
}

/// Writes its buffer each time it fills, and when flushed.
struct Print(Buffer);

impl Print {
    fn flush(&mut self) {
        if self.0.len == 0 {
            return;
        }
        let code = write_bytes(&self.0.bytes[..self.0.len]);
        assert!(code == 0, "write failed: {:?}", error(code));
        self.0.len = 0;
    }
}

impl fmt::Write for Print {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let mut rest = self.0.push(text);
        while !rest.is_empty() {
            self.flush();
            rest = self.0.push(rest);
        }
        Ok(())
    }
}

/// Keeps as much as fits in its buffer, and drops the rest.
struct Truncate(Buffer);

impl fmt::Write for Truncate {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0.push(text);
        Ok(())
    }
}

/// Writes the panic's location and message, cut to [`MAX_WRITE`] bytes,
/// then ends the thread with status 101, Rust's own exit code for a panic.
/// It neither uses [`print!`] nor checks the write, so that a panic there
/// doesn't come back here.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut message = Truncate(Buffer::new());
    let _ = write!(message, "{info}");
    write_bytes(&message.0.bytes[..message.0.len]);
    exit(101)
}
