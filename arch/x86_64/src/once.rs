//! A static that is written once, then shared for the rest of the kernel's
//! life: what the CPU's descriptor tables need, since loading one takes a
//! `&'static` reference to it.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, Ordering};

const EMPTY: u8 = 0;
const WRITING: u8 = 1;
const READY: u8 = 2;

/// A value stored at most once. Only the caller that moves `state` from
/// `EMPTY` to `WRITING` writes `value`, and nothing reads `value` until
/// `state` is `READY`, after which nothing writes it.
pub struct Once<T> {
    state: AtomicU8,
    value: UnsafeCell<MaybeUninit<T>>,
}

// SAFETY: A Once hands out only shared references to its value, and only
// once it is written; the state's Release and Acquire order that write
// before every read. A value written on one thread and read on others has
// to be Send and Sync.
unsafe impl<T: Send + Sync> Sync for Once<T> {}

impl<T> Once<T> {
    /// An empty `Once`.
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(EMPTY),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Stores `value` unless a value is already stored, in which case it
    /// drops `value`. Returns the stored value.
    pub fn init(&'static self, value: T) -> &'static T {
        if self
            .state
            .compare_exchange(EMPTY, WRITING, Ordering::Acquire, Ordering::Acquire)
            .is_ok()
        {
            // SAFETY: Moving state from EMPTY to WRITING makes this the only
            // writer, and no reader looks at value before READY.
            unsafe {
                (*self.value.get()).write(value);
            }
            self.state.store(READY, Ordering::Release);
        }
        loop {
            if let Some(value) = self.get() {
                return value;
            }
            core::hint::spin_loop();
        }
    }

    /// The stored value, or `None` before one is stored.
    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) != READY {
            return None;
        }
        // MaybeUninit<T> has the layout of T.
        let value = self.value.get().cast::<T>();
        // SAFETY: READY means the value was written, and nothing writes it
        // again, so a shared reference to it aliases no write.
        Some(unsafe { &*value })
    }
}
