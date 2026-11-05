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

    /// Builds page tables of the kernel's own, with frames from `alloc`,
    /// that map everything the loader mapped for the kernel at the same
    /// addresses and nothing else, and switches to them. Returns `None` if
    /// they were taken already. Panics if `alloc` runs out of frames.
    fn take_page_tables(alloc: &mut dyn FnMut() -> Option<Frame>) -> Option<Self::PageTables>;

    /// The address of the unmapped page right below the kernel stack, which
    /// a stack overflow runs into.
    fn guard_page() -> u64;

    /// Reads the first byte of [`Platform::guard_page`], which traps with
    /// [`Trap::PageFault`]. A trap handler that returns makes it fault
    /// again. For tests.
    fn read_guard_page() -> !;

    /// A second handle to the serial port, for a panic that finds the first
    /// one locked. It returns `Some` at most once, and `None` when no device
    /// answers. Anything else using the port at the same time interleaves
    /// its bytes with this one's.
    fn panic_serial() -> Option<Self::Serial>;

    /// The kernel heap's counters. It takes the heap's lock, so trap
    /// handlers never call it.
    fn heap_stats() -> HeapStats;

    /// Ends a QEMU test run through QEMU's debug exit device, reporting
    /// whether the tests passed. It touches the device only when it runs
    /// under a hypervisor; on real hardware it only halts.
    fn exit_qemu(success: bool) -> !;
}

/// The page tables the CPU uses, of which there is one set for now. Pages
/// are 4 KiB.
pub trait PageTables: Send {
    /// The virtual addresses [`PageTables::map`] maps into. The loader
    /// mapped nothing there.
    fn kernel_space(&self) -> Range<u64>;

    /// Maps the page at `page` to `frame`, no-execute, and writable if
    /// `writable`. New tables on the way come from `alloc`. The frame stays
    /// mapped for good.
    fn map(
        &mut self,
        page: u64,
        frame: Frame,
        writable: bool,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError>;

    /// Maps `frame` at the page after the heap's last one, writable and
    /// no-execute, and adds the page to the heap the global allocator
    /// allocates from. The heap's pages are outside
    /// [`PageTables::kernel_space`]. New tables on the way come from
    /// `alloc`. Panics if the heap's range is full.
    fn grow_heap(
        &mut self,
        frame: Frame,
        alloc: &mut dyn FnMut() -> Option<Frame>,
    ) -> Result<(), MapError>;

    /// Where `addr` is mapped, or `None` if it isn't.
    fn translate(&self, addr: u64) -> Option<Translation>;
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

/// Why [`PageTables::map`] failed. It hands the frame back.
#[derive(Debug)]
pub struct MapError {
    /// What was wrong.
    pub kind: MapErrorKind,
    /// The frame that wasn't mapped.
    pub frame: Frame,
}

/// What was wrong with a [`PageTables::map`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapErrorKind {
    /// The page isn't 4 KiB aligned.
    Unaligned,
    /// The page is outside [`PageTables::kernel_space`].
    OutsideKernelSpace,
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
}

/// A serial port the kernel writes text to. Writing `\n` starts a new line
/// on the terminal at the other end.
pub trait Serial: fmt::Write + Send {}

/// The kernel's trap handler, which the architecture calls on every trap.
/// When it returns, the trapped code resumes, and a fault runs the faulting
/// instruction again; after a [`Trap::DoubleFault`] or a trap that can't
/// resume, the architecture halts instead.
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
    /// a page fault on a full stack.
    DoubleFault,
    /// Any other exception, by the architecture's vector number and the
    /// error code it pushed, or 0 when it pushes none.
    Other {
        /// The vector number.
        vector: u8,
        /// The error code.
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

    /// Calls `f` with the link address of the trapped instruction, then
    /// walks the trapped code's frame pointers as [`Cpu::backtrace`] does.
    fn backtrace(&self, f: impl FnMut(u64)) -> BacktraceEnd;
}
