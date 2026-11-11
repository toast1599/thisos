//! What ThisOS's kernel and user code agree on: syscall numbers and
//! errors. How they pass through the registers is ADR 0007's.

#![no_std]
#![forbid(unsafe_code)]

/// The most bytes one [`Syscall::Write`] takes.
pub const MAX_WRITE: usize = 1024;

/// A syscall, by its number in RAX.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Syscall {
    /// Ends the calling thread, with its first argument as the thread's
    /// exit status. It never returns.
    Exit,
    /// Logs the UTF-8 text at its first argument, as long as its second,
    /// at most [`MAX_WRITE`] bytes, as one line of the kernel's log, without
    /// a newline at its end. Its value is the length. It fails, writing
    /// nothing, with [`Error::TooLong`], [`Error::BadAddress`] or
    /// [`Error::NotUtf8`].
    Write,
}

impl Syscall {
    /// The syscall's number.
    pub const fn number(self) -> u64 {
        match self {
            Syscall::Exit => 0,
            Syscall::Write => 1,
        }
    }
}

impl TryFrom<u64> for Syscall {
    type Error = Error;

    fn try_from(number: u64) -> Result<Syscall, Error> {
        match number {
            0 => Ok(Syscall::Exit),
            1 => Ok(Syscall::Write),
            _ => Err(Error::UnknownSyscall),
        }
    }
}

/// Why a syscall failed, by its code in RAX. A syscall that succeeds
/// returns 0 there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// No syscall has the number in RAX.
    UnknownSyscall,
    /// A buffer isn't all in pages mapped for user code.
    BadAddress,
    /// Text isn't UTF-8.
    NotUtf8,
    /// A buffer is longer than the syscall takes.
    TooLong,
}

impl Error {
    /// The error's code.
    pub const fn code(self) -> u64 {
        match self {
            Error::UnknownSyscall => 1,
            Error::BadAddress => 2,
            Error::NotUtf8 => 3,
            Error::TooLong => 4,
        }
    }

    /// The error with `code`, or `None` if no error has it.
    pub const fn from_code(code: u64) -> Option<Error> {
        match code {
            1 => Some(Error::UnknownSyscall),
            2 => Some(Error::BadAddress),
            3 => Some(Error::NotUtf8),
            4 => Some(Error::TooLong),
            _ => None,
        }
    }
}
