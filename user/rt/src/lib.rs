//! The runtime every ThisOS program links: its entry point, its syscalls,
//! [`print!`] and [`println!`], and its panic handler. A program names its
//! main function with [`entry!`].

#![no_std]

use core::arch::asm;
#[cfg(not(test))]
use core::arch::naked_asm;
use core::fmt::{self, Write as _};
#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))]
use core::sync::atomic::{AtomicBool, Ordering};

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
// defining it any other way takes unsafe code, which programs forbid. Host
// tests link no program, so they leave out everything that names it, and
// the panic handler, which std has.
#[cfg(not(test))]
unsafe extern "Rust" {
    /// The function `entry!` exports.
    safe fn __thisos_main() -> u64;
}

/// Where the kernel starts the program, with a 16-byte aligned stack and
/// every register but RDI zeroed, RBP included, which ends a backtrace's
/// frame chain here. The call makes the stack aligned as the System V ABI
/// wants it at a function's entry.
#[cfg(not(test))]
#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn _start() -> ! {
    naked_asm!("call {start}", "ud2", start = sym start)
}

#[cfg(not(test))]
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

/// Logs `text` in the kernel's log, one record per line, without a newline
/// at its end. Fails with [`Error::TooLong`] for more than [`MAX_WRITE`]
/// bytes.
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

/// Prints to the kernel's log, as `format!` formats: one record per line,
/// without its newline, and one for what is left after the last newline.
/// A line longer than [`MAX_WRITE`] bytes goes out as more records, split
/// between characters.
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
    print_to(args, |record| {
        let code = write_bytes(record);
        assert!(code == 0, "write failed: {:?}", error(code));
    });
}

/// Formats `args` and hands each record [`print!`] makes to `write`.
fn print_to(args: fmt::Arguments, write: impl FnMut(&[u8])) {
    let mut print = Print {
        buffer: Buffer::new(),
        write,
    };
    // Print's write_str never fails, so only a Display impl can.
    assert!(print.write_fmt(args).is_ok(), "formatting failed");
    if print.buffer.len != 0 {
        print.flush();
    }
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

/// Hands its buffer to `write` as a record at each newline, without the
/// newline, and when it is full and more of the line arrives. A full
/// buffer waits for that, so that a line of exactly [`MAX_WRITE`] bytes
/// and its newline make one record, not that line and an empty one.
struct Print<W> {
    buffer: Buffer,
    write: W,
}

impl<W: FnMut(&[u8])> Print<W> {
    fn flush(&mut self) {
        (self.write)(&self.buffer.bytes[..self.buffer.len]);
        self.buffer.len = 0;
    }
}

impl<W: FnMut(&[u8])> fmt::Write for Print<W> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for (i, line) in text.split('\n').enumerate() {
            if i > 0 {
                self.flush();
            }
            let mut rest = self.buffer.push(line);
            while !rest.is_empty() {
                self.flush();
                rest = self.buffer.push(rest);
            }
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
/// doesn't come back here. A panic while formatting the message would only
/// panic again, so it exits at once.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    static PANICKING: AtomicBool = AtomicBool::new(false);
    if !PANICKING.swap(true, Ordering::Relaxed) {
        let mut message = Truncate(Buffer::new());
        let _ = write!(message, "{info}");
        write_bytes(&message.0.bytes[..message.0.len]);
    }
    exit(101)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;
    use std::string::String;
    use std::vec::Vec;

    use super::*;

    /// The records [`print!`] makes of `args`.
    fn records(args: fmt::Arguments) -> Vec<String> {
        let mut records = Vec::new();
        print_to(args, |record| {
            records.push(String::from_utf8(record.to_vec()).unwrap());
        });
        records
    }

    #[test]
    fn a_full_line_and_its_newline_make_one_record() {
        let line = "a".repeat(MAX_WRITE);
        // The newline comes in a write_str of its own, after the line has
        // filled the buffer.
        assert_eq!(records(format_args!("{line}\n")), [line]);
    }

    #[test]
    fn each_line_is_a_record() {
        assert_eq!(records(format_args!("\n")), [""]);
        assert_eq!(records(format_args!("")), Vec::<String>::new());
        assert_eq!(records(format_args!("one\n\ntwo")), ["one", "", "two"]);
        assert_eq!(records(format_args!("{}{}\n", "par", "ts")), ["parts"]);
    }

    #[test]
    fn a_long_line_splits_between_characters() {
        // Two-byte characters, starting one byte in, so that the buffer
        // fills one byte short of a character.
        let line = format!("a{}", "é".repeat(MAX_WRITE));
        let records = records(format_args!("{line}\n"));
        assert_eq!(records.concat(), line);
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].len(), MAX_WRITE - 1);
    }

    #[test]
    fn a_truncated_message_ends_on_a_character() {
        let mut message = Truncate(Buffer::new());
        let text = format!("a{}", "é".repeat(MAX_WRITE));
        write!(message, "{text}").unwrap();
        assert_eq!(message.0.len, MAX_WRITE - 1);
        assert_eq!(
            &message.0.bytes[..message.0.len],
            &text.as_bytes()[..MAX_WRITE - 1]
        );
    }
}
