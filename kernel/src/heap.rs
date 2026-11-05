//! The kernel heap, which `alloc`'s boxes and collections come from. The
//! arch crate holds the allocator; this module gives it its memory, once,
//! at boot. Trap handlers never allocate: the heap's lock is a spin lock,
//! and a handler that found it held by the code it interrupted would never
//! return.

use thisos_hal::Platform;

use crate::{Arch, frames, paging};

/// The heap's size, which `init` maps in full.
const SIZE: usize = 4 << 20;

/// Maps the heap's [`SIZE`] bytes and logs what the allocator claimed.
/// `kernel::init` calls this once, after `paging::init`.
pub fn init() {
    for _ in 0..SIZE / 4096 {
        let Some(frame) = frames::alloc() else {
            panic!("no free frame for the heap");
        };
        if let Err(e) = paging::grow_heap(frame) {
            panic!("growing the heap failed: {:?}", e.kind);
        }
    }
    let claimed = Arch::heap_stats().claimed;
    log::info!("heap: {} KiB", claimed >> 10);
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;

    fn allocated() -> usize {
        Arch::heap_stats().allocated
    }

    #[test_case]
    fn boxes_and_vectors_hold_their_values_and_leak_nothing() {
        let before = allocated();
        {
            let b = Box::new(0x1234_5678_9abc_def0u64);
            let mut v = Vec::new();
            for i in 0..10_000u64 {
                v.push(i * 3);
            }
            assert!(allocated() > before);
            assert_eq!(*b, 0x1234_5678_9abc_def0);
            assert!(v.iter().zip(0..).all(|(&x, i)| x == i * 3));
        }
        assert_eq!(allocated(), before);
    }

    #[test_case]
    fn heap_memory_is_writable_and_no_execute() {
        let v = vec![0u8; 1 << 20];
        for addr in [v.as_ptr().addr(), v.as_ptr().addr() + v.len() - 1] {
            let Ok(addr) = u64::try_from(addr) else {
                panic!("{addr:#x} doesn't fit a u64");
            };
            let Some(t) = paging::translate(addr) else {
                panic!("heap address {addr:#x} isn't mapped");
            };
            assert!(t.writable && !t.executable, "{addr:#x}: {t:?}");
        }
    }

    #[test_case]
    fn freed_memory_is_reused() {
        // Three quarters of the heap, ten times over, fits only if each
        // block is freed for the next.
        for i in 0..10u8 {
            let v = vec![i; SIZE / 4 * 3];
            assert!(v.iter().all(|&x| x == i));
        }
    }

    #[test_case]
    fn a_reservation_past_the_heap_fails() {
        let before = allocated();
        let mut v = Vec::<u8>::new();
        assert!(v.try_reserve(SIZE + 1).is_err());
        assert_eq!(allocated(), before);
    }
}
