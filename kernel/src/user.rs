//! User code: loading init from the ramdisk, and the traps user code
//! raises, its syscalls and the faults that end it.

use core::{fmt, str};

use spin::Mutex;
use thisos_abi::{Error, MAX_WRITE, Syscall};
use thisos_elf::Elf;
use thisos_hal::{
    AddressSpace as _, Cpu, MapErrorKind, Platform, Protection, Trap, TrapFrame as _,
};

use crate::{Arch, frames, paging, thread};

/// The size of a program's stack, which ends where user space does.
const STACK_SIZE: u64 = 64 << 10;

/// How a user thread ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    /// Through the exit syscall, with this status.
    Exited(u64),
    /// By a trap its code raised at `ip`.
    Killed { trap: Trap, ip: u64 },
}

/// How the last user thread to end ended, for tests to wait on.
#[cfg(test)]
static ENDED: Mutex<Option<End>> = Mutex::new(None);

/// The text of the last write syscall, and its length, for tests. The
/// syscall takes the lock with interrupts off, so tests must too.
#[cfg(test)]
static WRITTEN: Mutex<([u8; MAX_WRITE], usize)> = Mutex::new(([0; MAX_WRITE], 0));

/// The loader's ramdisk, which holds init's ELF file.
static RAMDISK: Mutex<Option<&'static [u8]>> = Mutex::new(None);

/// Logs the size of the loader's ramdisk, and keeps it for [`start_init`].
pub fn init(ramdisk: Option<&'static [u8]>) {
    match ramdisk {
        Some(ramdisk) => log::info!("ramdisk is {} bytes", ramdisk.len()),
        None => log::warn!("no ramdisk, so no init to run"),
    }
    *RAMDISK.lock() = ramdisk;
}

/// Starts init from the ramdisk, if there is one. Panics if it doesn't
/// load.
pub fn start_init() {
    let Some(ramdisk) = *RAMDISK.lock() else {
        return;
    };
    match load(ramdisk) {
        Ok(program) => program.start(),
        Err(e) => panic!("loading init failed: {e}"),
    }
}

/// Why [`load`] failed.
enum LoadError {
    /// The file isn't an executable the kernel loads.
    Elf(thisos_elf::Error),
    /// The page at `page` couldn't be mapped.
    Map { page: u64, kind: MapErrorKind },
    /// No frame was free.
    NoFrames,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Elf(e) => write!(f, "not an executable the kernel loads: {e:?}"),
            LoadError::Map { page, kind } => write!(f, "mapping {page:#x} failed: {kind:?}"),
            LoadError::NoFrames => write!(f, "no free frame"),
        }
    }
}

/// A program loaded into an address space of its own, with a stack.
struct Program {
    space: paging::AddressSpace,
    entry: u64,
    stack_top: u64,
}

impl Program {
    /// Starts the program in a thread of its own, with 0 as its argument.
    /// Panics if kernel space or the frames run out.
    fn start(self) {
        thread::spawn_user(self.space, self.entry, self.stack_top, 0);
    }
}

/// Loads the executable in `file` into a new address space, with a zeroed
/// stack at the end of user space.
fn load(file: &[u8]) -> Result<Program, LoadError> {
    let elf = Elf::new(file).map_err(LoadError::Elf)?;
    let mut space = paging::new_space().ok_or(LoadError::NoFrames)?;
    match map_program(&elf, &mut space) {
        Ok(stack_top) => Ok(Program {
            space,
            entry: elf.entry(),
            stack_top,
        }),
        Err(e) => {
            space.destroy(&mut frames::free, &mut frames::free);
            Err(e)
        }
    }
}

/// Maps `elf`'s segments and a stack into `space`, and returns the stack's
/// top.
fn map_program(elf: &Elf, space: &mut paging::AddressSpace) -> Result<u64, LoadError> {
    for segment in elf.segments() {
        // Elf::new rejects segments both writable and executable.
        let protection = if segment.executable {
            Protection::ReadExecute
        } else if segment.writable {
            Protection::ReadWrite
        } else {
            Protection::ReadOnly
        };
        for page in segment.pages() {
            map_page(space, page.address, page.offset, page.data, protection)?;
        }
    }
    let top = space.user_space().end;
    for page in (top - STACK_SIZE..top).step_by(4096) {
        map_page(space, page, 0, &[], Protection::ReadWrite)?;
    }
    Ok(top)
}

/// Maps a new frame at `page` in `space`, holding `data` from `offset`
/// and zeroes everywhere else.
fn map_page(
    space: &mut paging::AddressSpace,
    page: u64,
    offset: usize,
    data: &[u8],
    protection: Protection,
) -> Result<(), LoadError> {
    let mut frame = frames::alloc().ok_or(LoadError::NoFrames)?;
    Arch::fill_frame(&mut frame, offset, data);
    space
        .map(page, frame, protection, &mut frames::alloc)
        .map_err(|e| {
            frames::free(e.frame);
            LoadError::Map { page, kind: e.kind }
        })
}

/// Handles a trap that [`TrapFrame::raised_by_user`](thisos_hal::TrapFrame::raised_by_user)
/// says user code raised: runs a syscall, and ends the thread on any other
/// trap, or on the exit syscall.
pub fn trap(trap: Trap, frame: &mut <Arch as Platform>::TrapFrame) {
    let end = if trap == Trap::Syscall {
        let (number, args) = frame.syscall();
        match Syscall::try_from(number) {
            Ok(Syscall::Exit) => End::Exited(args[0]),
            Ok(Syscall::Write) => {
                return match write(args[0], args[1]) {
                    Ok(len) => frame.set_syscall_result(0, len),
                    Err(e) => frame.set_syscall_result(e.code(), 0),
                };
            }
            Err(e) => return frame.set_syscall_result(e.code(), 0),
        }
    } else {
        End::Killed {
            trap,
            ip: frame.ip(),
        }
    };
    match end {
        End::Exited(status) => log::info!("user thread exited with status {status:#x}"),
        End::Killed { trap, ip } => log::warn!("user thread killed by {trap} at {ip:#x}"),
    }
    // User code holds no kernel lock, so nothing this handler interrupted
    // does either, and exit frees threads with interrupts on.
    Arch::enable_interrupts();
    #[cfg(test)]
    {
        *ENDED.lock() = Some(end);
    }
    thread::exit()
}

/// Logs the `len` bytes of text at `buf` in user space as one record, as
/// the write syscall does, and returns `len`. Runs with interrupts off, as
/// syscalls do, and copies into a buffer on the stack, since trap handlers
/// never allocate.
fn write(buf: u64, len: u64) -> Result<u64, Error> {
    let mut bytes = [0; MAX_WRITE];
    let bytes = usize::try_from(len)
        .ok()
        .and_then(|len| bytes.get_mut(..len))
        .ok_or(Error::TooLong)?;
    if !Arch::copy_from_user(buf, bytes) {
        return Err(Error::BadAddress);
    }
    let text = str::from_utf8(bytes).map_err(|_| Error::NotUtf8)?;
    let text = text.strip_suffix('\n').unwrap_or(text);
    log::info!(target: "user", "{text}");
    #[cfg(test)]
    {
        let mut written = WRITTEN.lock();
        written.0[..text.len()].copy_from_slice(text.as_bytes());
        written.1 = text.len();
    }
    Ok(len)
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::ptr;
    use core::sync::atomic::AtomicU64;

    use thisos_hal::{Access, UserProgram};

    use super::*;
    use crate::testing::spin_until;

    /// The text of the last write syscall, which it then forgets.
    fn take_written() -> String {
        Arch::without_interrupts(|| {
            let mut written = WRITTEN.lock();
            let (bytes, len) = &mut *written;
            let text = String::from_utf8_lossy(&bytes[..*len]).into_owned();
            *len = 0;
            text
        })
    }

    /// Runs `program` in a new space, with its code in the first user page
    /// and a stack page after it, and `arg` as its argument. Returns how it
    /// ended and where its code was; see [`run_in`].
    fn run(program: UserProgram, arg: impl FnOnce(u64) -> u64) -> (End, u64) {
        run_with_stack(program, |stack| (Vec::new(), arg(stack)))
    }

    /// Runs `program` as [`run`] does, with the bytes `setup` returns at
    /// the start of its stack page, and the argument it returns. `setup`
    /// gets the stack page's address.
    fn run_with_stack(
        program: UserProgram,
        setup: impl FnOnce(u64) -> (Vec<u8>, u64),
    ) -> (End, u64) {
        let before = frames::free_frames();
        let Some(mut space) = paging::new_space() else {
            panic!("no free frame for a space");
        };
        let code = space.user_space().start;
        let stack = code + 4096;
        let (bytes, arg) = setup(stack);
        let code_frame = page_frame();
        Arch::write_user_program(&code_frame, program);
        let mut stack_frame = page_frame();
        Arch::fill_frame(&mut stack_frame, 0, &bytes);
        for (page, frame, protection) in [
            (code, code_frame, Protection::ReadExecute),
            (stack, stack_frame, Protection::ReadWrite),
        ] {
            if let Err(e) = space.map(page, frame, protection, &mut frames::alloc) {
                panic!("mapping {page:#x} failed: {:?}", e.kind);
            }
        }
        let program = Program {
            space,
            entry: code,
            stack_top: stack + 4096,
        };
        (run_in(program, before, arg), code)
    }

    /// Starts `program` with `arg` as its argument, and returns how it
    /// ended once its thread and space are freed. Checks that every frame
    /// taken since there were `before` free came back: those of the space
    /// and its pages, and those of the thread's stack. Kernel-space tables
    /// the stack needed stay, so those are left out. Meanwhile it checks
    /// that this thread, a kernel thread, runs on the kernel's tables every
    /// time ticks switch back to it.
    fn run_in(program: Program, before: usize, arg: u64) -> End {
        let space_frames = before - frames::free_frames();
        let entry = program.entry;
        *ENDED.lock() = None;
        thread::spawn_user(program.space, entry, program.stack_top, arg);
        // User code allocates nothing, and nothing frees the thread until
        // wait_for_exits, so this counts its stack whenever it runs.
        let spawned = frames::free_frames();
        spin_until("the user thread to end", || {
            assert_eq!(Arch::translate_active(entry), None, "user code's page");
            ENDED.lock().is_some()
        });
        thread::wait_for_exits();
        assert_eq!(
            frames::free_frames(),
            spawned + space_frames + thread::STACK_SIZE / 4096,
            "frames leaked"
        );
        let Some(end) = *ENDED.lock() else {
            unreachable!("ENDED was set");
        };
        end
    }

    fn page_frame() -> thisos_frames::Frame {
        let Some(frame) = frames::alloc() else {
            panic!("no free frame");
        };
        frame
    }

    fn static_address(target: &AtomicU64) -> u64 {
        let Ok(address) = u64::try_from(ptr::from_ref(target).addr()) else {
            panic!("a static's address doesn't fit a u64");
        };
        address
    }

    #[test_case]
    fn init_loads_writes_and_exits() {
        let Some(ramdisk) = *RAMDISK.lock() else {
            panic!("no ramdisk");
        };
        take_written();
        let before = frames::free_frames();
        let program = match load(ramdisk) {
            Ok(program) => program,
            Err(e) => panic!("loading init failed: {e}"),
        };
        assert_eq!(run_in(program, before, 0), End::Exited(0));
        assert_eq!(take_written(), "hello from init");
    }

    #[test_case]
    fn a_syscall_keeps_registers_and_an_unknown_one_fails() {
        let (end, _) = run(UserProgram::Syscalls, |_| 0);
        assert_eq!(end, End::Exited(Error::UnknownSyscall.code()));
    }

    /// Runs [`UserProgram::Write`] with `len` and the buffer `buf` returns,
    /// given where `text` is in the stack page, and returns its exit
    /// status and what it wrote.
    fn run_write(buf: impl FnOnce(u64) -> u64, len: usize, text: &[u8]) -> (End, String) {
        take_written();
        let (end, _) = run_with_stack(UserProgram::Write, |stack| {
            let at = stack + 16;
            let mut bytes = Vec::new();
            bytes.extend(buf(at).to_le_bytes());
            bytes.extend(u64::try_from(len).unwrap_or(u64::MAX).to_le_bytes());
            bytes.extend(text);
            (bytes, stack)
        });
        (end, take_written())
    }

    #[test_case]
    fn write_logs_text_without_its_newline() {
        let text = b"written from user code\n";
        let (end, written) = run_write(|at| at, text.len(), text);
        assert_eq!(end, End::Exited(0));
        assert_eq!(written, "written from user code");
        let long = [b'a'; MAX_WRITE];
        let (end, written) = run_write(|at| at, long.len(), &long);
        assert_eq!(end, End::Exited(0));
        assert_eq!(written.len(), MAX_WRITE);
    }

    #[test_case]
    fn write_rejects_bad_buffers_and_writes_nothing() {
        static TARGET: AtomicU64 = AtomicU64::new(0);
        let failed = |error: Error| (End::Exited(error.code()), String::new());
        let kernel = static_address(&TARGET);
        assert_eq!(run_write(|_| kernel, 1, b""), failed(Error::BadAddress));
        // The last 4 bytes of the stack page, and 4 of the unmapped page
        // after it.
        let across = |at: u64| at - 16 + 4096 - 4;
        assert_eq!(run_write(across, 8, b""), failed(Error::BadAddress));
        assert_eq!(run_write(|at| at, 2, b"\xff\xfe"), failed(Error::NotUtf8));
        assert_eq!(
            run_write(|at| at, MAX_WRITE + 1, b""),
            failed(Error::TooLong)
        );
    }

    #[test_case]
    fn user_code_reads_its_own_stack() {
        let (end, _) = run(UserProgram::Read, |stack| stack);
        assert_eq!(end, End::Exited(0));
    }

    #[test_case]
    fn reading_kernel_memory_kills_the_thread() {
        static TARGET: AtomicU64 = AtomicU64::new(0);
        let address = static_address(&TARGET);
        let (end, code) = run(UserProgram::Read, |_| address);
        let trap = Trap::PageFault {
            address,
            access: Access::Read,
        };
        assert_eq!(end, End::Killed { trap, ip: code });
    }

    #[test_case]
    fn a_privileged_instruction_kills_the_thread() {
        let (end, code) = run(UserProgram::Privileged, |_| 0);
        assert!(
            matches!(end, End::Killed { trap: Trap::Other { .. }, ip } if ip == code),
            "{end:?}"
        );
    }

    // The kernel saves no SIMD state, so user code may not use it.
    #[test_case]
    fn simd_kills_the_thread() {
        let (end, code) = run(UserProgram::Simd, |_| 0);
        assert!(
            matches!(end, End::Killed { trap: Trap::Other { .. }, ip } if ip == code),
            "{end:?}"
        );
    }

    // The user thread spins through many ticks, each of which preempts it
    // in ring 3 and switches to this thread.
    #[test_case]
    fn ticks_preempt_user_code() {
        let (end, _) = run(UserProgram::Spin, |_| 0);
        assert_eq!(end, End::Exited(0));
    }
}
