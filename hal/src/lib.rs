//! The traits the ThisOS kernel needs from an architecture. Each
//! architecture crate implements them, and the kernel uses nothing else
//! from the hardware.

#![no_std]
#![forbid(unsafe_code)]

use core::fmt;
use core::ops::Range;

use thisos_frames::Frame;

/// What the kernel learns from the boot loader, in a form that names no
/// boot protocol. Each architecture crate builds it from its loader's own
/// structure. Fields are added as the kernel comes to need them.
#[derive(Debug)]
pub struct BootInfo<P: Platform> {
    /// The serial port, or `None` when no device answers.
    pub serial: Option<P::Serial>,
    /// The screen, or `None` when the loader set up none in a format the
    /// kernel draws.
    pub framebuffer: Option<Framebuffer<P::Pixels>>,
    /// Whether the architecture made stores to the framebuffer
    /// write-combining. Without it each store is its own bus transaction
    /// where the firmware maps the framebuffer uncached.
    pub framebuffer_write_combining: bool,
    /// Physical memory, as the loader left it.
    pub memory: P::MemoryMap,
    /// The file the loader loaded as its ramdisk, mapped in the upper half
    /// in memory the loader's map marks Loader, or `None` without one.
    pub ramdisk: Option<&'static [u8]>,
}

/// A range of physical memory in the loader's memory map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryRegion {
    /// The physical address of the first byte.
    pub start: u64,
    /// The physical address after the last byte.
    pub end: u64,
    /// Who uses it.
    pub kind: MemoryKind,
}

/// Who uses a [`MemoryRegion`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryKind {
    /// Free RAM the kernel may use.
    Usable,
    /// RAM the loader left in use: the kernel image, its stack, the page
    /// tables and the boot information, the memory map included.
    Loader,
    /// Memory the firmware reports, by the firmware's own type number, such
    /// as UEFI runtime services, ACPI tables or MMIO.
    Firmware(u32),
}

/// A linear framebuffer of 4-byte pixels, as the firmware set it up. Rows
/// are `stride` pixels apart, and `pixels` holds at least `height` of them.
#[derive(Debug)]
pub struct Framebuffer<M> {
    /// The pixels, row after row.
    pub pixels: M,
    /// Visible pixels per row.
    pub width: usize,
    /// Rows.
    pub height: usize,
    /// Pixels from the start of one row to the start of the next.
    pub stride: usize,
    /// The order of the color bytes in each pixel.
    pub format: PixelFormat,
}

/// The memory of a [`Framebuffer`], written one pixel or two at a time
/// and never read: firmware often maps it uncached, where every access is a
/// bus transaction. A pixel is 4 bytes, stored as a little-endian word, and
/// an index counts pixels from the start of the first row.
#[expect(
    clippy::len_without_is_empty,
    reason = "the console only needs the length to clear the screen"
)]
pub trait Pixels: Send {
    /// The number of pixels.
    fn len(&self) -> usize;

    /// Stores `pixel` at `index`. Panics if `index` is out of range.
    fn store(&mut self, index: usize, pixel: u32);

    /// Stores the low half of `pixels` at `index` and the high half after
    /// it, in one 8-byte store. Panics unless `index` is even and both
    /// pixels are in range.
    fn store_pair(&mut self, index: usize, pixels: u64);
}

/// The byte order of a framebuffer pixel. The fourth byte is unused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// Red, green, blue.
    Rgb,
    /// Blue, green, red.
    Bgr,
}

/// Control of the processor the kernel runs on.
pub trait Cpu {
    /// Stops this processor for good: interrupts stay off and it never
    /// runs kernel code again.
    fn halt() -> !;

    /// Walks the frame pointers on the kernel stack and calls `f` with the
    /// address of each call on it, innermost first, starting in the caller.
    /// Addresses are link addresses, inside the call instruction: what
    /// `addr2line` expects for the kernel ELF. Stops after [`MAX_FRAMES`].
    fn backtrace(f: impl FnMut(u64)) -> BacktraceEnd;

    /// Traps with [`Trap::Breakpoint`], and returns once the trap handler
    /// does.
    fn breakpoint();

    /// Lets interrupts reach the trap handler. They are off until this
    /// runs.
    fn enable_interrupts();

    /// Turns interrupts on and waits for one, then returns once its
    /// handler has, with interrupts on.
    fn wait_for_interrupt();

    /// Runs `f` with interrupts off, then turns them back on if they were
    /// on before.
    fn without_interrupts<R>(f: impl FnOnce() -> R) -> R;

    /// Runs `probe`, which never returns by itself, and returns once a trap
    /// handler calls [`TrapFrame::abandon`]. Panics if a catch is running
    /// already. For tests.
    fn catch(probe: Probe);

    /// Whether the kernel is stopped from reading, writing and running user
    /// pages right now, as it is everywhere but in user copies. For tests.
    fn user_access_blocked() -> bool;

    /// Leaves the kernel for user code at `ip` in the address space the
    /// CPU uses, with stack pointer `sp`, `arg` as its first argument,
    /// every other general register zeroed, and interrupts on. The calling
    /// thread comes back to the kernel only through traps, which start at
    /// the top of its stack, so whatever it had there is gone. Panics
    /// unless `ip` is in [`AddressSpace::user_space`] and `sp` is in it or
    /// at its end.
    fn enter_user(ip: u64, sp: u64, arg: u64) -> !;
}

/// Code the architecture runs under [`Cpu::catch`] to make a trap. Each is
/// the architecture's own, written so that abandoning it skips nothing
/// with a destructor, which abandoning a Rust closure can't promise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Probe {
    /// Reads the first byte of [`Platform::guard_page`] again and again,
    /// which traps with [`Trap::PageFault`].
    ReadGuardPage,
    /// Pushes onto the kernel stack until it runs into the guard page, where
    /// the page fault's handler has no stack to run on, so it traps with
    /// [`Trap::DoubleFault`]. It runs with interrupts off, so that no
    /// interrupt handler runs on the stack it fills and overflows there
    /// itself.
    OverflowStack,
    /// Spins forever with interrupts as they were.
    Spin,
    /// Lets the kernel reach user pages, as a user copy does, then spins
    /// forever. Trap handlers must still find [`Cpu::user_access_blocked`],
    /// and the catch blocks user access again when it returns.
    SpinUserAccessOpen,
    /// Reads a read-only byte of the architecture's own and writes it back,
    /// again and again, which traps with [`Trap::PageFault`] on write at
    /// [`Platform::probe_target`] when the CPU enforces read-only pages.
    WriteReadOnly,
    /// Calls a `ret` of the architecture's own in a no-execute page, again
    /// and again, which traps with [`Trap::PageFault`] on execute at
    /// [`Platform::probe_target`] when the CPU enforces no-execute pages.
    ExecuteNoExecute,
    /// Reads a fixed user address, [`Platform::probe_target`], again and
    /// again, which traps with [`Trap::PageFault`] on read there: because
    /// nothing is mapped there, or because the kernel may not read user
    /// pages.
    ReadUser,
    /// Calls the same user address as [`Probe::ReadUser`] again and again,
    /// which traps with [`Trap::PageFault`] on execute there: because
    /// nothing is mapped there, or because the kernel may not run user
    /// code.
    ExecuteUser,
}

/// User code the architecture writes with [`Platform::write_user_program`],
/// for tests that need code in user mode before anything loads programs.
/// Each fits in its first page and runs from anywhere. The architecture
/// writes each as it writes a [`Probe`], so kernel code can't make user
/// code of its own choosing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserProgram {
    /// Puts a value of its own in every general register a syscall keeps,
    /// makes a syscall with the number `u64::MAX`, which no syscall has,
    /// then checks those registers. It exits with the error code the
    /// syscall returned if they all kept their values and the syscall's
    /// value is 0, and with `u64::MAX` otherwise.
    Syscalls,
    /// Reads the byte at its argument, then exits with status 0.
    Read,
    /// Runs an instruction only the kernel may run.
    Privileged,
    /// Runs an SSE instruction, or the architecture's equivalent.
    Simd,
    /// Spins for 2^28 cycles of a counter user code can read, through many
    /// timer ticks, then exits with status 0.
    Spin,
}

/// The most frames [`Cpu::backtrace`] reports.
pub const MAX_FRAMES: usize = 32;

/// Why [`Cpu::backtrace`] stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BacktraceEnd {
    /// It reached the outermost frame.
    Done,
    /// It reported [`MAX_FRAMES`] frames.
    TooDeep,
    /// The next frame pointer is outside the kernel stack.
    OutsideStack(u64),
    /// The next frame pointer isn't 8-byte aligned.
    Misaligned(u64),
    /// The next frame pointer doesn't lie above the current one.
    NotIncreasing(u64),
}

/// The devices an architecture hands the kernel in [`BootInfo`].
pub trait Platform {
    /// The serial port the kernel logs to.
    type Serial: Serial;

    /// The memory of the framebuffer in [`BootInfo`].
    type Pixels: Pixels;

    /// The registers the CPU had when it trapped.
    type TrapFrame: TrapFrame;

    /// The memory map in [`BootInfo`]. A clone taken before iterating
    /// reads the whole map again.
    type MemoryMap: Iterator<Item = MemoryRegion> + Clone + Send;

    /// The kernel's own page tables.
    type PageTables: PageTables;

    /// A suspended kernel thread: where it stopped and the stack it runs
    /// on. Resuming it consumes it, so no thread resumes twice, and
    /// dropping it leaves the thread suspended for good.
    type Context: Send;

    /// Builds page tables of the kernel's own, with frames from `alloc`,
    /// that map everything the loader mapped for the kernel at the same
    /// addresses, and nothing else but the registers of devices the
    /// architecture drives itself, and switches to them. Returns `None` if
    /// they were taken already. Panics if `alloc` runs out of frames.
    fn take_page_tables(alloc: &mut dyn FnMut() -> Option<Frame>) -> Option<Self::PageTables>;

    /// Makes the CPU use the kernel's own tables, as
    /// [`PageTables::activate`] does, without the [`PageTables`] value: the
    /// scheduler switches tables in the timer's handler, which can't take
    /// the lock the kernel keeps them behind. Does nothing if the CPU uses
    /// them already. Panics before [`Platform::take_page_tables`].
    fn activate_kernel_tables();

    /// Moves the stacks the CPU switches to for some traps, which start out
    /// with nothing unmapped below them, onto stacks `stack` maps. For each
    /// it calls `stack(size)`, which returns the address just past a stack
    /// of `size` bytes with an unmapped page below it, and its start
    /// aligned to the power of two at or above twice its size. Call it
    /// once, after [`Platform::take_page_tables`].
    fn move_trap_stacks(stack: &mut dyn FnMut(usize) -> u64);

    /// The stacks the CPU switches to for some traps. For tests.
    fn trap_stacks() -> impl Iterator<Item = Range<u64>>;

    /// A kernel thread that, once [`Platform::switch`] resumes it, calls
    /// `entry` on `stack` with interrupts off, passing it the context of
    /// the thread that switched to it. `stack` is a stack of
    /// [`PageTables::kernel_space`] that nothing else uses, with an
    /// unmapped page below it, such as `move_trap_stacks`'s callback
    /// returns; the architecture writes the thread's first frame at its
    /// top. Panics unless `stack` is whole pages inside kernel space.
    fn new_context(stack: Range<u64>, entry: fn(Self::Context) -> !) -> Self::Context;

    /// Suspends the calling thread and resumes `to`. Returns once another
    /// thread switches to the caller's context, with that thread's own
    /// context, which it left to do so. Panics if interrupts are on.
    fn switch(to: Self::Context) -> Self::Context;

    /// The address of the unmapped page right below the running thread's
    /// stack, which a stack overflow runs into.
    fn guard_page() -> u64;

    /// The address whose page fault ends `probe`: the guard page for
    /// [`Probe::ReadGuardPage`], and the probe's own byte for
    /// [`Probe::WriteReadOnly`] and [`Probe::ExecuteNoExecute`], and a page
    /// in [`AddressSpace::user_space`] for [`Probe::ReadUser`] and
    /// [`Probe::ExecuteUser`]. `None` for a probe that ends some other way.
    /// For tests.
    fn probe_target(probe: Probe) -> Option<u64>;

    /// A second handle to the serial port, for a panic that finds the first
    /// one locked. It returns `Some` at most once, and `None` when no device
    /// answers. Anything else using the port at the same time interleaves
    /// its bytes with this one's.
    fn panic_serial() -> Option<Self::Serial>;

    /// A second handle to the framebuffer in [`BootInfo`], for a panic that
    /// can't use the first: the first is locked, or not handed to the
    /// logger yet. It returns `Some` at most once, and `None` when there is
    /// no framebuffer. Anything still drawing through the first mixes its
    /// pixels with this one's.
    fn panic_framebuffer() -> Option<Framebuffer<Self::Pixels>>;

    /// The kernel heap's counters. It takes the heap's lock, so trap
    /// handlers never call it.
    fn heap_stats() -> HeapStats;

    /// Starts the timer, which traps with [`Trap::Timer`] `hz` times a
    /// second once interrupts are on, and returns how fast it counts and
    /// how that was found. Call it once. Panics if the CPU lacks the timer
    /// or nothing gives its frequency.
    fn start_timer(hz: u32) -> TimerFrequency;

    /// The timer's frequency from each source the architecture has: the
    /// one the CPU reports, and one measured again against another clock,
    /// with the timer stopped meanwhile. For tests.
    fn timer_frequencies() -> TimerFrequencies;

    /// Whether every line of the legacy interrupt controller is masked, or
    /// `true` where there is none. For tests.
    fn legacy_interrupts_masked() -> bool;

    /// Where `addr` is mapped in the page tables the CPU uses now, or
    /// `None` if it isn't. For tests.
    fn translate_active(addr: u64) -> Option<Translation>;

    /// Writes code that only returns at the start of `frame`, for
    /// [`Probe::ExecuteUser`] to find if the CPU ever let it run. For
    /// tests.
    fn write_return(frame: &Frame);

    /// Writes `program` at the start of `frame`. For tests.
    fn write_user_program(frame: &Frame, program: UserProgram);

    /// Ends a QEMU test run through QEMU's debug exit device, reporting
    /// whether the tests passed. It touches the device only when it runs
    /// under a hypervisor; on real hardware it only halts.
    fn exit_qemu(success: bool) -> !;
}

/// The kernel's own page tables: the upper half every address space
/// shares, with nothing mapped in the lower half. Pages are 4 KiB.
pub trait PageTables: Send {
    /// A user address space.
    type AddressSpace: AddressSpace;

    /// The virtual addresses [`PageTables::map`] maps into. The loader
    /// mapped nothing there.
    fn kernel_space(&self) -> Range<u64>;

    /// Maps the page at `page` to `frame`, no-execute, and writable if
    /// `writable`. New tables on the way come from `alloc`. The tables hold
    /// the frame until [`PageTables::unmap`].
    fn map(
        &mut self,
        page: u64,
        frame: Frame,
        writable: bool,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError>;

    /// Unmaps the page at `page` in [`PageTables::kernel_space`], and
    /// returns the frame it mapped, or `None` if nothing was mapped there.
    /// Kernel-space addresses are never mapped again after this, so code
    /// that still uses the page faults and reaches no other memory.
    fn unmap(&mut self, page: u64) -> Option<Frame>;

    /// Maps `frame` at the page after the heap's last one, writable and
    /// no-execute, and adds the page to the heap the global allocator
    /// allocates from. The heap's pages are outside
    /// [`PageTables::kernel_space`], and stay mapped for good. New tables
    /// on the way come from `alloc`. Panics if the heap's range is full.
    fn grow_heap(
        &mut self,
        frame: Frame,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError>;

    /// Where `addr` is mapped, or `None` if it isn't.
    fn translate(&self, addr: u64) -> Option<Translation>;

    /// Whether anything is mapped in the lower half. For tests.
    fn lower_half_mapped(&self) -> bool;

    /// A new address space with nothing in its lower half, sharing this
    /// upper half, with its first table from `alloc`. `None` if `alloc` has
    /// no frame.
    fn new_space(&self, alloc: &mut dyn FnMut() -> Option<Frame>) -> Option<Self::AddressSpace>;

    /// Makes the CPU use these tables, with an empty lower half, in place
    /// of whichever address space it used.
    fn activate(&self);
}

/// A user address space: a lower half of its own, and the upper half of
/// the kernel's [`PageTables`]. Dropping one without
/// [`AddressSpace::destroy`] leaks its frames and its tables, as dropping a
/// [`Frame`] does.
pub trait AddressSpace: Send {
    /// The virtual addresses [`AddressSpace::map`] maps into: the lower
    /// half, apart from the first pages, which stay unmapped so that a
    /// null pointer plus a small offset always faults, and on x86_64 the
    /// last page, which SYSRET couldn't return past.
    fn user_space(&self) -> Range<u64>;

    /// Maps the page at `page` to `frame` for user code, with `protection`.
    /// New tables on the way come from `alloc`. The space holds the frame
    /// until [`AddressSpace::unmap`] or [`AddressSpace::destroy`].
    fn map(
        &mut self,
        page: u64,
        frame: Frame,
        protection: Protection,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError>;

    /// Unmaps the page at `page`, and returns the frame it mapped, or
    /// `None` if nothing was mapped there. Tables it leaves empty stay
    /// until [`AddressSpace::destroy`].
    fn unmap(&mut self, page: u64) -> Option<Frame>;

    /// Where `addr` is mapped in this space, or `None` if it isn't.
    fn translate(&self, addr: u64) -> Option<Translation>;

    /// Makes the CPU use this space.
    fn activate(&self);

    /// Ends the space: calls `mapped` with each frame still mapped, and
    /// `free` with each frame of its tables. If the CPU uses this space, it
    /// switches to the kernel's [`PageTables`] first.
    fn destroy(self, mapped: &mut dyn FnMut(Frame), free: &mut dyn FnMut(Frame));
}

/// What user code may do with a page. No page is both writable and
/// executable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protection {
    /// Reads only.
    ReadOnly,
    /// Reads and writes.
    ReadWrite,
    /// Reads and instruction fetches.
    ReadExecute,
}

/// The kernel heap's counters, as [`Platform::heap_stats`] reads them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeapStats {
    /// Bytes in live allocations, by the sizes their layouts ask for.
    pub allocated: usize,
    /// Bytes [`PageTables::grow_heap`] has given the heap, less what
    /// alignment cost.
    pub claimed: usize,
}

/// How fast the timer counts, as [`Platform::start_timer`] found it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerFrequency {
    /// Counts per second.
    pub hz: u64,
    /// Where the frequency came from, such as "CPUID leaf 0x15".
    pub source: &'static str,
}

/// The timer's frequency from each source, as
/// [`Platform::timer_frequencies`] finds it, or `None` from a source that
/// gave none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerFrequencies {
    /// What the CPU reports.
    pub reported: Option<u64>,
    /// What a measurement against another clock gives, and which clock.
    pub measured: Option<TimerFrequency>,
}

/// Why [`PageTables::map`] or [`AddressSpace::map`] failed. It hands the
/// frame back.
#[derive(Debug)]
pub struct MapError {
    /// What was wrong.
    pub kind: MapErrorKind,
    /// The frame that wasn't mapped.
    pub frame: Frame,
}

/// What was wrong with a [`PageTables::map`] or [`AddressSpace::map`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapErrorKind {
    /// The page isn't 4 KiB aligned.
    Unaligned,
    /// The page is outside [`PageTables::kernel_space`].
    OutsideKernelSpace,
    /// The page is outside [`AddressSpace::user_space`].
    OutsideUserSpace,
    /// The page is mapped already.
    Mapped,
    /// `alloc` had no frame for a table.
    NoFrames,
}

/// Where an address is mapped, as [`PageTables::translate`] finds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Translation {
    /// The physical address it maps to.
    pub phys: u64,
    /// Whether writes are allowed.
    pub writable: bool,
    /// Whether instruction fetches are allowed.
    pub executable: bool,
    /// Whether user code may reach it.
    pub user: bool,
}

/// A serial port the kernel writes text to. Writing `\n` starts a new line
/// on the terminal at the other end.
pub trait Serial: fmt::Write + Send {}

/// The kernel's trap handler, which the architecture calls on every trap.
/// When it returns, the trapped code resumes, and a fault runs the faulting
/// instruction again; after a [`Trap::DoubleFault`] or a trap that can't
/// resume, the architecture halts instead, unless the handler called
/// [`TrapFrame::abandon`] after a double fault. The handler may switch
/// threads on a [`Trap::Timer`], which the architecture acknowledges before
/// calling it; the trapped thread resumes when a switch comes back to it.
/// After a trap [`TrapFrame::raised_by_user`], the handler may also never
/// return, ending the thread.
pub type TrapHandler<P> = fn(Trap, &mut <P as Platform>::TrapFrame);

/// Why the CPU trapped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trap {
    /// A breakpoint instruction, as [`Cpu::breakpoint`] runs.
    Breakpoint,
    /// An access the page tables don't allow.
    PageFault {
        /// The address accessed.
        address: u64,
        /// What the access was.
        access: Access,
    },
    /// A fault while the CPU was starting the handler of another, such as
    /// a page fault on a full stack, or a trap taken with the stack pointer
    /// below the bottom of its stack.
    DoubleFault,
    /// A tick of the timer [`Platform::start_timer`] started.
    Timer,
    /// A syscall from user code. [`TrapFrame::syscall`] reads its number
    /// and arguments, and [`TrapFrame::set_syscall_result`] sets what it
    /// returns.
    Syscall,
    /// An interrupt the kernel never asked for, by the architecture's
    /// number for it: a stray legacy line, a spurious interrupt or any
    /// other.
    Stray {
        /// The vector number on x86_64, the interrupt ID on aarch64.
        vector: u32,
    },
    /// Any other exception, by the architecture's number for it and the
    /// error code it gave, or 0 when it gives none.
    Other {
        /// The vector number on x86_64, the exception class on aarch64.
        vector: u8,
        /// The error code on x86_64, the syndrome register on aarch64.
        error: u64,
    },
}

impl fmt::Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trap::Breakpoint => write!(f, "breakpoint"),
            Trap::PageFault { address, access } => {
                write!(f, "page fault on {access} at {address:#x}")
            }
            Trap::DoubleFault => write!(f, "double fault"),
            Trap::Timer => write!(f, "timer interrupt"),
            Trap::Syscall => write!(f, "syscall"),
            Trap::Stray { vector } => write!(f, "stray interrupt on vector {vector:#x}"),
            Trap::Other { vector, error } => {
                write!(f, "exception {vector}, error code {error:#x}")
            }
        }
    }
}

/// What a faulting access was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    /// A data read.
    Read,
    /// A data write.
    Write,
    /// An instruction fetch.
    Execute,
}

impl fmt::Display for Access {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Access::Read => "read",
            Access::Write => "write",
            Access::Execute => "execute",
        })
    }
}

/// The registers saved when the CPU trapped. Displaying it prints all of
/// them.
pub trait TrapFrame: fmt::Display {
    /// The trapped instruction's address, or for a breakpoint the address
    /// after it.
    fn ip(&self) -> u64;

    /// The stack pointer when the CPU trapped.
    fn sp(&self) -> u64;

    /// Whether user code raised the trap: a syscall, or an exception its
    /// instruction caused. Interrupts, NMIs, machine checks and double
    /// faults aren't the doing of the code they arrive in, so they are
    /// never user code's, even when they arrive in it.
    fn raised_by_user(&self) -> bool;

    /// The number and the six arguments of a [`Trap::Syscall`], in the
    /// registers ADR 0007 puts them in.
    fn syscall(&self) -> (u64, [u64; 6]);

    /// Makes a [`Trap::Syscall`] return `error`, 0 for success, and
    /// `value`.
    fn set_syscall_result(&mut self, error: u64, value: u64);

    /// Calls `f` with the link address of the trapped instruction, then
    /// walks the trapped code's frame pointers as [`Cpu::backtrace`] does.
    fn backtrace(&self, f: impl FnMut(u64)) -> BacktraceEnd;

    /// Gives up the probe the running [`Cpu::catch`] runs: once the handler
    /// returns, the CPU carries on where that catch returns, even after a
    /// [`Trap::DoubleFault`]. Returns `false`, and changes nothing, when no
    /// catch is running. It can't tell a trap the probe caused from one in
    /// a handler that interrupted the probe, and gives up that handler too.
    fn abandon(&mut self) -> bool;
}
