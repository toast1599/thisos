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
    pub framebuffer: Option<Framebuffer<'static>>,
    /// Whether the architecture made stores to the framebuffer
    /// write-combining. Without it each store is its own bus transaction
    /// where the firmware maps the framebuffer uncached.
    pub framebuffer_write_combining: bool,
}

/// A linear framebuffer of 4-byte pixels, as the firmware set it up. Rows
/// are `stride` pixels apart, and `buffer` holds at least `height` of them.
pub struct Framebuffer<'a> {
    /// The pixels, row after row.
    pub buffer: &'a mut [u8],
    /// Visible pixels per row.
    pub width: usize,
    /// Rows.
    pub height: usize,
    /// Pixels from the start of one row to the start of the next.
    pub stride: usize,
    /// The order of the color bytes in each pixel.
    pub format: PixelFormat,
}

// By hand, so that Debug doesn't print megabytes of pixels.
impl fmt::Debug for Framebuffer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Framebuffer")
            .field("len", &self.buffer.len())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .field("format", &self.format)
            .finish()
    }
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
