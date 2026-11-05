//! The traits the ThisOS kernel needs from an architecture. Each
//! architecture crate implements them, and the kernel uses nothing else
//! from the hardware.

#![no_std]
#![forbid(unsafe_code)]

use core::fmt;

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

    /// Ends a QEMU test run through QEMU's debug exit device, reporting
    /// whether the tests passed. It touches the device only when it runs
    /// under a hypervisor; on real hardware it only halts.
    fn exit_qemu(success: bool) -> !;
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
