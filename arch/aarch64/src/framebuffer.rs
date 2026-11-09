//! The framebuffer's memory, reached through a raw pointer with volatile
//! stores. No Rust reference to it exists: the loader's mapping of all
//! physical memory maps the same frames a second time, and the compiler
//! must not treat device memory as ordinary RAM.

use core::ops::Range;

use thisos_hal::Pixels;

/// The framebuffer's pixels. Only `new` makes one, so `base` is aligned
/// and the `len` pixels from it are the framebuffer, mapped for the rest of
/// the kernel's life and reached through nothing but the two handles `new`
/// makes. It points at pixel pairs, which have the strictest alignment of
/// the stores.
#[derive(Debug)]
pub struct FramebufferPixels {
    base: *mut u64,
    len: usize,
}

// SAFETY: The pixels are reached only through this value and the one other
// handle `new` makes, through volatile stores alone, so moving either to
// another thread moves no reference with it.
unsafe impl Send for FramebufferPixels {}

impl FramebufferPixels {
    /// Two handles to the pixels in `range`: one for the kernel's console
    /// and one for a panic that can't use it. `None` when the range doesn't
    /// start 8-byte aligned, which pair stores need. Both only store, so
    /// two of them interleave pixels but never break each other's stores.
    ///
    /// # Safety
    ///
    /// `range` is the framebuffer, mapped for the rest of the kernel's life,
    /// and nothing else in the kernel reads or writes it, through this
    /// mapping or any other. Call this at most once.
    pub unsafe fn new(range: Range<*mut u8>) -> Option<[Self; 2]> {
        if range.start.addr() % 8 != 0 {
            return None;
        }
        let len = range.end.addr().checked_sub(range.start.addr())? / 4;
        let pixels = || Self {
            base: range.start.cast(),
            len,
        };
        Some([pixels(), pixels()])
    }
}

impl Pixels for FramebufferPixels {
    fn len(&self) -> usize {
        self.len
    }

    fn store(&mut self, index: usize, pixel: u32) {
        assert!(
            index < self.len,
            "pixel {index} outside the framebuffer's {}",
            self.len
        );
        // SAFETY: index is in range, so the pixel is inside the framebuffer,
        // which new's caller says is mapped and reached only through this,
        // and 4-byte aligned, as base is.
        unsafe {
            self.base
                .cast::<u32>()
                .wrapping_add(index)
                .write_volatile(pixel);
        }
    }

    fn store_pair(&mut self, index: usize, pixels: u64) {
        assert!(
            index % 2 == 0 && index + 1 < self.len,
            "pixel pair {index} misaligned or outside the framebuffer's {}",
            self.len
        );
        // SAFETY: As in store, for both pixels. base is 8-byte aligned, and
        // an even index is a whole number of pairs from it.
        unsafe {
            self.base.wrapping_add(index / 2).write_volatile(pixels);
        }
    }
}
