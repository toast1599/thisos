//! The framebuffer's memory, reached through a raw pointer with volatile
//! stores. No Rust reference to it exists: the loader's mapping of all
//! physical memory maps the same frames a second time, and the compiler
//! must not treat device memory as ordinary RAM.

use core::ops::Range;

use thisos_hal::Pixels;
use x86_64::VirtAddr;

/// The framebuffer's pixels. Only `new` makes one, so `base` is aligned
/// and the `len` pixels from it are the framebuffer, mapped for the rest of
/// the kernel's life and reached through nothing else. It points at pixel
/// pairs, which have the strictest alignment of the stores.
#[derive(Debug)]
pub struct FramebufferPixels {
    base: *mut u64,
    len: usize,
}

// SAFETY: The pixels are reached only through this value, so moving it to
// another thread moves the only access with it.
unsafe impl Send for FramebufferPixels {}

impl FramebufferPixels {
    /// The pixels in `range`, or `None` when it doesn't start 8-byte
    /// aligned, which pair stores need.
    ///
    /// # Safety
    ///
    /// `range` is the framebuffer, mapped for the rest of the kernel's life,
    /// and nothing else in the kernel reads or writes it, through this
    /// mapping or any other. Call this at most once.
    pub unsafe fn new(range: Range<*mut u8>) -> Option<Self> {
        if range.start.addr() % 8 != 0 {
            return None;
        }
        let len = range.end.addr().checked_sub(range.start.addr())? / 4;
        Some(Self {
            base: range.start.cast(),
            len,
        })
    }

    /// The pixels' virtual addresses.
    pub fn range(&self) -> Range<VirtAddr> {
        let pixels = self.base.cast::<u32>();
        VirtAddr::from_ptr(pixels)..VirtAddr::from_ptr(pixels.wrapping_add(self.len))
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
