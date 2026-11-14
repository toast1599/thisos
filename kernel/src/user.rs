//! User code: loading init from the ramdisk, and the traps user code
//! raises, its syscalls and the faults that end it.

use alloc::sync::Arc;
use core::{fmt, str};

use spin::Mutex;
use thisos_abi::{Error, MAX_WRITE, Rights, Syscall};
use thisos_elf::Elf;
use thisos_hal::{
    AddressSpace as _, Cpu, MapErrorKind, Platform, Protection, Trap, TrapFrame as _,
};

use crate::memory::{Mapping, MemoryObject};
use crate::process::{End, Object, Process};
use crate::{Arch, frames, paging, thread};

/// The size of a program's stack, which ends where user space does.
const STACK_SIZE: u64 = 64 << 10;

/// The rights of init's handle to itself.
const INIT_RIGHTS: Rights = Rights::READ
    .union(Rights::WRITE)
    .union(Rights::DUPLICATE)
    .union(Rights::TRANSFER);

/// How the last user process to end ended, for tests to wait on. The trap
/// handler takes the lock with interrupts off, so everything else must
/// too.
#[cfg(test)]
static ENDED: Mutex<Option<End>> = Mutex::new(None);

/// The records the last write syscall logged, joined by newlines, for
/// tests. Taken with interrupts off.
#[cfg(test)]
static WRITTEN: Mutex<Written> = Mutex::new(Written {
    bytes: [0; WRITTEN_SIZE],
    len: 0,
});

/// Escapes make a record up to six bytes per byte of text, as `\u{1b}`.
#[cfg(test)]
const WRITTEN_SIZE: usize = 6 * MAX_WRITE;

#[cfg(test)]
struct Written {
    bytes: [u8; WRITTEN_SIZE],
    len: usize,
}

#[cfg(test)]
impl fmt::Write for Written {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        self.bytes
            .get_mut(self.len..end)
            .ok_or(fmt::Error)?
            .copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

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
        Ok(program) => {
            let (process, handle) = new_init();
            program.start(process, u64::from(handle));
        }
        Err(e) => panic!("loading init failed: {e}"),
    }
}

/// A process for init, holding a handle to itself with [`INIT_RIGHTS`],
/// and that handle.
fn new_init() -> (Arc<Process>, u32) {
    let process = Process::new();
    let Ok(handle) = process.insert_self(INIT_RIGHTS) else {
        panic!("a new handle table is full");
    };
    (process, handle)
}

/// Why [`load`] failed.
enum LoadError {
    /// The file isn't an executable the kernel loads.
    Elf(thisos_elf::Error),
    /// The pages at `addr` couldn't be mapped.
    Map { addr: u64, kind: MapErrorKind },
    /// No memory was free.
    NoMemory,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Elf(e) => write!(f, "not an executable the kernel loads: {e:?}"),
            LoadError::Map { addr, kind } => write!(f, "mapping {addr:#x} failed: {kind:?}"),
            LoadError::NoMemory => write!(f, "no free memory"),
        }
    }
}

/// A program loaded into an address space of its own, with a stack.
struct Program {
    space: Arc<paging::AddressSpace>,
    entry: u64,
    stack_top: u64,
}

impl Program {
    /// Starts the program in a thread of `process`, with `arg` as its
    /// argument. Panics if kernel space or the frames run out.
    fn start(self, process: Arc<Process>, arg: u64) {
        thread::spawn_user(process, self.space, self.entry, self.stack_top, arg);
    }
}

/// Loads the executable in `file` into a new address space, with a zeroed
/// stack at the end of user space. Each segment and the stack are a memory
/// object of their own.
fn load(file: &[u8]) -> Result<Program, LoadError> {
    let elf = Elf::new(file).map_err(LoadError::Elf)?;
    let space = paging::new_space().ok_or(LoadError::NoMemory)?;
    match map_program(&elf, &space) {
        Ok(stack_top) => Ok(Program {
            space: Arc::new(space),
            entry: elf.entry(),
            stack_top,
        }),
        Err(e) => {
            space.destroy(&mut drop, &mut frames::free);
            Err(e)
        }
    }
}

/// Maps `elf`'s segments and a stack into `space`, and returns the stack's
/// top.
fn map_program(elf: &Elf, space: &paging::AddressSpace) -> Result<u64, LoadError> {
    for segment in elf.segments() {
        // Elf::new rejects segments both writable and executable.
        let protection = if segment.executable {
            Protection::ReadExecute
        } else if segment.writable {
            Protection::ReadWrite
        } else {
            Protection::ReadOnly
        };
        let range = segment.page_range();
        let mut object = new_object(range.end - range.start)?;
        for (frame, page) in object.frames_mut().iter_mut().zip(segment.pages()) {
            Arch::fill_frame(frame, page.offset, page.data);
        }
        map_object(space, range.start, object, protection)?;
    }
    let top = space.user_space().end;
    let stack = new_object(STACK_SIZE)?;
    map_object(space, top - STACK_SIZE, stack, Protection::ReadWrite)?;
    Ok(top)
}

/// A memory object of `size` bytes, a multiple of 4096.
fn new_object(size: u64) -> Result<MemoryObject, LoadError> {
    let pages = usize::try_from(size / 4096).map_err(|_| LoadError::NoMemory)?;
    MemoryObject::new(pages).map_err(|_| LoadError::NoMemory)
}

/// Maps all of `object`, which nothing else holds, at `addr` in `space`.
fn map_object(
    space: &paging::AddressSpace,
    addr: u64,
    object: MemoryObject,
    protection: Protection,
) -> Result<(), LoadError> {
    let pages = object.pages();
    let Ok(mapping) = Mapping::new(Arc::new(object), protection) else {
        unreachable!("a new object has no mappings to conflict with");
    };
    space
        .map(addr, mapping, 0, pages, protection, &mut frames::alloc)
        .map_err(|e| LoadError::Map { addr, kind: e.kind })
}

/// Handles a trap that [`TrapFrame::raised_by_user`](thisos_hal::TrapFrame::raised_by_user)
/// says user code raised: runs a syscall, and ends the process on any other
/// trap, or on the exit syscall.
pub fn trap(trap: Trap, frame: &mut <Arch as Platform>::TrapFrame) {
    if trap != Trap::Syscall {
        end(End::Killed {
            trap,
            ip: frame.ip(),
        });
        // A #DB comes in on an IST stack, which the next one starts at the
        // top of again, so interrupts stay off until the thread is gone
        // from it.
        thread::exit_from_trap();
    }
    // Syscalls run with interrupts on, so that they can allocate and free,
    // and a tick may preempt them. The SYSCALL stub turns interrupts off
    // again before it returns to user code.
    Arch::enable_interrupts();
    let (number, args) = frame.syscall();
    match syscall(number, args) {
        Ok(value) => frame.set_syscall_result(0, value),
        Err(e) => frame.set_syscall_result(e.code(), 0),
    }
}

/// Runs the syscall `number` with `args`, and returns its value.
fn syscall(number: u64, args: [u64; 6]) -> Result<u64, Error> {
    match Syscall::try_from(number)? {
        Syscall::Exit => {
            end(End::Exited(args[0]));
            thread::exit()
        }
        Syscall::Write => write(args[0], args[1]),
        Syscall::Close => process().close(handle(args[0])?).map(|()| 0),
        Syscall::Duplicate => process()
            .duplicate(handle(args[0])?, rights(args[1])?)
            .map(u64::from),
        Syscall::Restrict => process()
            .restrict(handle(args[0])?, rights(args[1])?)
            .map(|()| 0),
        Syscall::MemoryCreate => memory_create(args[0]),
        Syscall::Map => map(args[0], args[1], args[2], args[3], args[4]),
        Syscall::Unmap => space().unmap(args[0]).map(|_| 0).ok_or(Error::BadAddress),
    }
}

/// Logs how the running thread's process ended, and records it there.
fn end(end: End) {
    match end {
        End::Exited(status) => log::info!("user process exited with status {status:#x}"),
        End::Killed { trap, ip } => log::warn!("user process killed by {trap} at {ip:#x}"),
    }
    process().set_end(end);
    #[cfg(test)]
    Arch::without_interrupts(|| *ENDED.lock() = Some(end));
}

/// The running thread's process.
fn process() -> Arc<Process> {
    let Some(process) = thread::process() else {
        panic!("a kernel thread trapped as user code");
    };
    process
}

/// The running thread's address space.
fn space() -> Arc<paging::AddressSpace> {
    let Some(space) = thread::space() else {
        panic!("a kernel thread trapped as user code");
    };
    space
}

fn handle(arg: u64) -> Result<u32, Error> {
    u32::try_from(arg).map_err(|_| Error::BadHandle)
}

fn rights(arg: u64) -> Result<Rights, Error> {
    u32::try_from(arg)
        .ok()
        .and_then(Rights::from_bits)
        .ok_or(Error::BadRights)
}

/// The pages in `len` bytes, which must be a nonzero multiple of 4096.
fn pages(len: u64) -> Result<usize, Error> {
    if len == 0 || len % 4096 != 0 {
        return Err(Error::OutOfRange);
    }
    usize::try_from(len / 4096).map_err(|_| Error::OutOfRange)
}

/// Makes a memory object of `size` bytes, as the memory-create syscall
/// does, and returns a handle to it.
fn memory_create(size: u64) -> Result<u64, Error> {
    let object = MemoryObject::new(pages(size)?)?;
    process()
        .insert(Object::Memory(Arc::new(object)), Rights::ALL)
        .map(u64::from)
}

/// Maps `len` bytes of the memory object `handle` names from `offset` at
/// `addr`, with the rights in `rights`, as the map syscall does.
fn map(handle_arg: u64, offset: u64, len: u64, addr: u64, rights_arg: u64) -> Result<u64, Error> {
    let (object, held) = process().get(handle(handle_arg)?)?;
    let Object::Memory(object) = object else {
        return Err(Error::WrongType);
    };
    let wanted = rights(rights_arg)?;
    let protection = if wanted == Rights::READ {
        Protection::ReadOnly
    } else if wanted == Rights::READ | Rights::WRITE {
        Protection::ReadWrite
    } else if wanted == Rights::READ | Rights::EXECUTE {
        Protection::ReadExecute
    } else {
        return Err(Error::BadRights);
    };
    if !held.contains(wanted) {
        return Err(Error::AccessDenied);
    }
    if offset % 4096 != 0 {
        return Err(Error::BadAddress);
    }
    let pages = pages(len)?;
    let first = usize::try_from(offset / 4096).map_err(|_| Error::OutOfRange)?;
    if first
        .checked_add(pages)
        .is_none_or(|end| end > object.pages())
    {
        return Err(Error::OutOfRange);
    }
    let mapping = Mapping::new(object, protection)?;
    space()
        .map(addr, mapping, first, pages, protection, &mut frames::alloc)
        .map_err(|e| match e.kind {
            MapErrorKind::Mapped => Error::AddressInUse,
            MapErrorKind::NoFrames => Error::NoMemory,
            MapErrorKind::Unaligned
            | MapErrorKind::OutsideUserSpace
            | MapErrorKind::OutsideKernelSpace
            | MapErrorKind::OutOfRange => Error::BadAddress,
        })?;
    Ok(0)
}

/// Logs the `len` bytes of text at `buf` in user space as the write
/// syscall does, one record per line, and returns `len`. Copies the text
/// into a buffer on the stack first.
fn write(buf: u64, len: u64) -> Result<u64, Error> {
    let mut bytes = [0; MAX_WRITE];
    let bytes = usize::try_from(len)
        .ok()
        .and_then(|len| bytes.get_mut(..len))
        .ok_or(Error::TooLong)?;
    if !copy_from_user(buf, bytes) {
        return Err(Error::BadAddress);
    }
    let text = str::from_utf8(bytes).map_err(|_| Error::NotUtf8)?;
    let text = text.strip_suffix('\n').unwrap_or(text);
    for line in text.split('\n') {
        log::info!(target: "user", "{}", Escaped(line));
    }
    #[cfg(test)]
    Arch::without_interrupts(|| {
        use fmt::Write as _;
        let mut written = WRITTEN.lock();
        written.len = 0;
        for (i, line) in text.split('\n').enumerate() {
            let newline = if i == 0 { "" } else { "\n" };
            assert!(
                write!(written, "{newline}{}", Escaped(line)).is_ok(),
                "WRITTEN is full"
            );
        }
    });
    Ok(len)
}

/// Copies the bytes at `addr` in user space into `bytes`, as
/// [`Platform::copy_from_user`] does, one page at a time with interrupts
/// off, so that nothing unmaps a page while it is read.
fn copy_from_user(addr: u64, bytes: &mut [u8]) -> bool {
    let mut addr = addr;
    let mut rest = bytes;
    while !rest.is_empty() {
        let in_page = 4096 - addr % 4096;
        let len = rest
            .len()
            .min(usize::try_from(in_page).unwrap_or(usize::MAX));
        let (chunk, after) = rest.split_at_mut(len);
        if !Arch::without_interrupts(|| Arch::copy_from_user(addr, chunk)) {
            return false;
        }
        // The page was user space, so the next one's address fits.
        addr += in_page;
        rest = after;
    }
    true
}

/// A line of user text with its control characters shown as escapes, such
/// as `\u{1b}`, so that it can't move the terminal's cursor.
struct Escaped<'a>(&'a str);

impl fmt::Display for Escaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.chars() {
            if c.is_control() {
                write!(f, "{}", c.escape_default())?;
            } else {
                fmt::Write::write_char(f, c)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::ptr;
    use core::sync::atomic::AtomicU64;

    use thisos_hal::{Access, Cpu, UserProgram};

    use super::*;
    use crate::testing::spin_until;

    /// The records of the last write syscall, joined by newlines, which it
    /// then forgets.
    fn take_written() -> String {
        Arch::without_interrupts(|| {
            let mut written = WRITTEN.lock();
            let text = String::from_utf8_lossy(&written.bytes[..written.len]).into_owned();
            written.len = 0;
            text
        })
    }

    /// Runs `program` in a new process and space, with its code in the
    /// first user page and a stack page after it, and the argument `arg`
    /// returns given the stack page's address and the process. Returns how
    /// it ended and where its code was; see [`run_in`].
    fn run(program: UserProgram, arg: impl FnOnce(u64, &Arc<Process>) -> u64) -> (End, u64) {
        run_with_stack(program, |stack, process| (Vec::new(), arg(stack, process)))
    }

    /// Runs `program` as [`run`] does, with the bytes `setup` returns at
    /// the start of its stack page, and the argument it returns.
    fn run_with_stack(
        program: UserProgram,
        setup: impl FnOnce(u64, &Arc<Process>) -> (Vec<u8>, u64),
    ) -> (End, u64) {
        let before = frames::free_frames();
        let Some(space) = paging::new_space() else {
            panic!("no free frame for a space");
        };
        let code = space.user_space().start;
        let stack = code + 4096;
        let process = Process::new();
        let (bytes, arg) = setup(stack, &process);
        let mut code_object = page_object();
        Arch::write_user_program(&mut code_object.frames_mut()[0], program);
        let mut stack_object = page_object();
        Arch::fill_frame(&mut stack_object.frames_mut()[0], 0, &bytes);
        for (page, object, protection) in [
            (code, code_object, Protection::ReadExecute),
            (stack, stack_object, Protection::ReadWrite),
        ] {
            if let Err(e) = map_object(&space, page, object, protection) {
                panic!("{e}");
            }
        }
        let program = Program {
            space: Arc::new(space),
            entry: code,
            stack_top: stack + 4096,
        };
        (run_in(program, before, &process, arg), code)
    }

    /// Starts `program` in `process` with `arg` as its argument, and
    /// returns how it ended once its thread and space are freed. Checks
    /// that the process recorded that end and closed its handles, and that
    /// every frame taken since there were `before` free came back: those of
    /// the space and its pages, and those of the thread's stack.
    /// Kernel-space tables the stack needed stay, so those are left out.
    /// Meanwhile it checks that this thread, a kernel thread, runs on the
    /// kernel's tables every time ticks switch back to it.
    fn run_in(program: Program, before: usize, process: &Arc<Process>, arg: u64) -> End {
        let space_frames = before - frames::free_frames();
        let entry = program.entry;
        Arch::without_interrupts(|| *ENDED.lock() = None);
        program.start(Arc::clone(process), arg);
        // User code allocates nothing, and nothing frees the thread until
        // wait_for_exits, so this counts its stack whenever it runs.
        let spawned = frames::free_frames();
        spin_until("the user thread to end", || {
            assert_eq!(Arch::translate_active(entry), None, "user code's page");
            Arch::without_interrupts(|| ENDED.lock().is_some())
        });
        thread::wait_for_exits();
        assert_eq!(
            frames::free_frames(),
            spawned + space_frames + thread::STACK_SIZE / 4096,
            "frames leaked"
        );
        let Some(end) = Arch::without_interrupts(|| *ENDED.lock()) else {
            unreachable!("ENDED was set");
        };
        assert_eq!(process.end(), Some(end));
        // Handles to itself are closed, and its thread is gone.
        assert_eq!(Arc::strong_count(process), 1);
        end
    }

    fn page_object() -> MemoryObject {
        let Ok(object) = MemoryObject::new(1) else {
            panic!("no memory for a page");
        };
        object
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
        let (process, handle) = new_init();
        assert_eq!(
            run_in(program, before, &process, u64::from(handle)),
            End::Exited(0)
        );
        assert_eq!(take_written(), "hello from init, holding handle 0x10000");
    }

    #[test_case]
    fn handle_syscalls_check_handles_and_rights() {
        let (end, _) = run(UserProgram::Handles, |_, process| {
            let Ok(handle) = process.insert_self(INIT_RIGHTS) else {
                panic!("a new handle table is full");
            };
            u64::from(handle)
        });
        assert_eq!(end, End::Exited(0));
    }

    #[test_case]
    fn memory_syscalls_create_map_share_and_unmap() {
        let (end, _) = run(UserProgram::Memory, |_, process| {
            let Ok(handle) = process.insert_self(INIT_RIGHTS) else {
                panic!("a new handle table is full");
            };
            u64::from(handle)
        });
        assert_eq!(end, End::Exited(0));
    }

    #[test_case]
    fn a_syscall_keeps_registers_and_an_unknown_one_fails() {
        let (end, _) = run(UserProgram::Syscalls, |_, _| 0);
        assert_eq!(end, End::Exited(Error::UnknownSyscall.code()));
    }

    /// Runs [`UserProgram::Write`] with `len` and the buffer `buf` returns,
    /// given where `text` is in the stack page, and returns its exit
    /// status and what it wrote.
    fn run_write(buf: impl FnOnce(u64) -> u64, len: usize, text: &[u8]) -> (End, String) {
        take_written();
        let (end, _) = run_with_stack(UserProgram::Write, |stack, _| {
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
    fn write_logs_a_record_per_line_with_control_characters_escaped() {
        let text = b"one\n\ntwo\r\x1b[2J\t\n";
        let (end, written) = run_write(|at| at, text.len(), text);
        assert_eq!(end, End::Exited(0));
        assert_eq!(written, "one\n\ntwo\\r\\u{1b}[2J\\t");
        // A zero-length write reads nothing, so even a null pointer logs
        // its empty line.
        assert_eq!(run_write(|_| 0, 0, b""), (End::Exited(0), String::new()));
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
        let (end, _) = run(UserProgram::Read, |stack, _| stack);
        assert_eq!(end, End::Exited(0));
    }

    #[test_case]
    fn reading_kernel_memory_kills_the_thread() {
        static TARGET: AtomicU64 = AtomicU64::new(0);
        let address = static_address(&TARGET);
        let (end, code) = run(UserProgram::Read, |_, _| address);
        let trap = Trap::PageFault {
            address,
            access: Access::Read,
        };
        assert_eq!(end, End::Killed { trap, ip: code });
    }

    #[test_case]
    fn a_privileged_instruction_kills_the_thread() {
        let (end, code) = run(UserProgram::Privileged, |_, _| 0);
        assert!(
            matches!(end, End::Killed { trap: Trap::Other { .. }, ip } if ip == code),
            "{end:?}"
        );
    }

    // The exception arrives on the stack every user thread's debug
    // exceptions share. ICEBP raises it as a trap, after the instruction,
    // so the CPU reports the byte after it.
    #[test_case]
    fn a_debug_exception_kills_the_thread() {
        let (end, code) = run(UserProgram::Debug, |_, _| 0);
        let trap = Trap::Other {
            vector: 1,
            error: 0,
        };
        assert_eq!(end, End::Killed { trap, ip: code + 1 });
    }

    // The kernel saves no SIMD state, so user code may not use it.
    #[test_case]
    fn simd_kills_the_thread() {
        let (end, code) = run(UserProgram::Simd, |_, _| 0);
        assert!(
            matches!(end, End::Killed { trap: Trap::Other { .. }, ip } if ip == code),
            "{end:?}"
        );
    }

    // The user thread spins through many ticks, each of which preempts it
    // in ring 3 and switches to this thread.
    #[test_case]
    fn ticks_preempt_user_code() {
        let (end, _) = run(UserProgram::Spin, |_, _| 0);
        assert_eq!(end, End::Exited(0));
    }
}
