//! Walking the frame records on the kernel stack. The kernel is built with
//! frame pointers, so each function's prologue stores its caller's x29 and
//! its own return address, x30, as a pair and points x29 at it.

use core::ops::Range;
use core::sync::atomic::{AtomicU64, Ordering};

use thisos_hal::{BacktraceEnd, MAX_FRAMES};

use crate::ptr_at;

// Zero until init, so a walk before then stops at its first frame.
static STACK_START: AtomicU64 = AtomicU64::new(0);
static STACK_END: AtomicU64 = AtomicU64::new(0);

/// Records the kernel stack's addresses. The kernel is linked where it
/// runs, so return addresses are link addresses already.
pub fn init(stack: Range<u64>) {
    STACK_START.store(stack.start, Ordering::Relaxed);
    STACK_END.store(stack.end, Ordering::Relaxed);
}

/// The lowest address of the kernel stack.
pub fn stack_bottom() -> u64 {
    STACK_START.load(Ordering::Relaxed)
}

/// A frame pointer whose two words, the caller's x29 and the return
/// address, lie 8-byte aligned inside the kernel stack. Only `new` makes one.
struct Frame(u64);

impl Frame {
    fn new(fp: u64, stack: &Range<u64>) -> Result<Frame, BacktraceEnd> {
        if fp % 8 != 0 {
            return Err(BacktraceEnd::Misaligned(fp));
        }
        match fp.checked_add(16) {
            Some(end) if fp >= stack.start && end <= stack.end => Ok(Frame(fp)),
            _ => Err(BacktraceEnd::OutsideStack(fp)),
        }
    }

    fn caller(&self) -> u64 {
        // SAFETY: new checked that the word lies 8-byte aligned inside the
        // kernel stack, which stays mapped for the kernel's life. It is a
        // volatile read because the stack below the caller's frames changes
        // under the compiler's feet as the walk's own calls run.
        unsafe { ptr_at::<u64>(self.0).read_volatile() }
    }

    fn return_address(&self) -> u64 {
        // SAFETY: As in caller, for the word after it, which new checked
        // lies inside the stack too.
        unsafe { ptr_at::<u64>(self.0 + 8).read_volatile() }
    }
}

/// Walks the frame records from `fp`, calling `f` with the address of each
/// `bl`, innermost first.
#[expect(
    clippy::inline_always,
    reason = "as its own function, a caller that passes its own x29 tail-calls it after \
              popping its frame record, and walk's prologue writes over that record"
)]
#[inline(always)]
pub fn walk(mut fp: u64, mut f: impl FnMut(u64)) -> BacktraceEnd {
    let stack = STACK_START.load(Ordering::Relaxed)..STACK_END.load(Ordering::Relaxed);
    for _ in 0..MAX_FRAMES {
        let frame = match Frame::new(fp, &stack) {
            Ok(frame) => frame,
            Err(end) => return end,
        };
        let ret = frame.return_address();
        // _start clears x30 before its first call.
        if ret == 0 {
            return BacktraceEnd::Done;
        }
        // Every instruction is 4 bytes, so the bl is the one before the
        // return address.
        f(ret.wrapping_sub(4));
        let caller = frame.caller();
        if caller == 0 {
            return BacktraceEnd::Done;
        }
        if caller <= fp {
            return BacktraceEnd::NotIncreasing(caller);
        }
        fp = caller;
    }
    BacktraceEnd::TooDeep
}
