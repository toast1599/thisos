//! Walking the frame pointers on the kernel stack. The kernel is built with
//! frame pointers, so each function's prologue pushes its caller's rbp right
//! below the return address and points rbp at it.

use core::ops::Range;
use core::sync::atomic::{AtomicU64, Ordering};

use thisos_hal::{BacktraceEnd, MAX_FRAMES};
use x86_64::VirtAddr;

// Zero until init, so a walk before then stops at its first frame.
static STACK_START: AtomicU64 = AtomicU64::new(0);
static STACK_END: AtomicU64 = AtomicU64::new(0);
static IMAGE_OFFSET: AtomicU64 = AtomicU64::new(0);

/// Records the kernel stack's addresses and where the loader put the kernel
/// image, which the walk subtracts to give link addresses.
pub fn init(stack: Range<u64>, image_offset: u64) {
    STACK_START.store(stack.start, Ordering::Relaxed);
    STACK_END.store(stack.end, Ordering::Relaxed);
    IMAGE_OFFSET.store(image_offset, Ordering::Relaxed);
}

/// A frame pointer whose two words, the caller's rbp and the return
/// address, lie 8-byte aligned inside the kernel stack. Only `new` makes one.
struct Frame(u64);

impl Frame {
    fn new(rbp: u64, stack: &Range<u64>) -> Result<Frame, BacktraceEnd> {
        if rbp % 8 != 0 {
            return Err(BacktraceEnd::Misaligned(rbp));
        }
        match rbp.checked_add(16) {
            Some(end) if rbp >= stack.start && end <= stack.end => Ok(Frame(rbp)),
            _ => Err(BacktraceEnd::OutsideStack(rbp)),
        }
    }

    fn caller(&self) -> u64 {
        // SAFETY: The word is aligned and inside the kernel stack, which the
        // loader maps in full and only this core uses. A volatile read makes
        // no reference, so it can't alias one. On a chain of prologues it
        // holds the caller's rbp; on a chain broken by code without frame
        // pointers it holds whatever bits the RAM has, which the walk only
        // checks and prints.
        unsafe { VirtAddr::new(self.0).as_ptr::<u64>().read_volatile() }
    }

    fn return_address(&self) -> u64 {
        // SAFETY: As in caller: the word 8 bytes up is also aligned and
        // inside the kernel stack.
        unsafe { VirtAddr::new(self.0 + 8).as_ptr::<u64>().read_volatile() }
    }
}

/// Walks the frames from `rbp` outwards; see `Cpu::backtrace`.
pub fn walk(mut rbp: u64, mut f: impl FnMut(u64)) -> BacktraceEnd {
    let stack = STACK_START.load(Ordering::Relaxed)..STACK_END.load(Ordering::Relaxed);
    let offset = IMAGE_OFFSET.load(Ordering::Relaxed);
    for _ in 0..MAX_FRAMES {
        let frame = match Frame::new(rbp, &stack) {
            Ok(frame) => frame,
            Err(end) => return end,
        };
        let ret = frame.return_address();
        // The loader enters the kernel with rbp zeroed and a zero pushed as
        // the return address, so the outermost frame ends in zeros.
        if ret == 0 {
            return BacktraceEnd::Done;
        }
        // One byte back from the return address is inside the call. The
        // return address itself can be past the caller's end when the callee
        // never returns, as with panic_fmt, and addr2line then names nothing.
        f(ret.wrapping_sub(offset).wrapping_sub(1));
        let caller = frame.caller();
        if caller == 0 {
            return BacktraceEnd::Done;
        }
        if caller <= rbp {
            return BacktraceEnd::NotIncreasing(caller);
        }
        rbp = caller;
    }
    BacktraceEnd::TooDeep
}
