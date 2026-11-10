//! The way in from Limine: the requests it answers, `_start` and the boot
//! stack it switches to, and the kernel's `BootInfo` built from the answers.

use core::arch::global_asm;
use core::ops::Range;
use core::ptr::addr_of;
use core::sync::atomic::{AtomicU64, Ordering};

use limine::BaseRevision;
use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest};
use thisos_hal::{BootInfo, Framebuffer, PixelFormat, TrapHandler};

use crate::framebuffer::FramebufferPixels;
use crate::memory::MemoryMap;
use crate::{Aarch64, backtrace, ptr_at, to_u64};

// Limine finds these by their magic numbers anywhere in the kernel's
// writable data, and writes its answers into them before it jumps to
// _start. boot_info reads every one of them, which also keeps the linker
// from dropping them.
static BASE_REVISION: BaseRevision = BaseRevision::new();
static MEMORY_MAP: MemmapRequest = MemmapRequest::new();
static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();
static HHDM: HhdmRequest = HhdmRequest::new();

/// Where Limine mapped all physical memory, which `boot_info` records. Zero
/// until then: Limine's mapping is in the upper half, never at 0.
static PHYS_OFFSET: AtomicU64 = AtomicU64::new(0);

/// Where Limine mapped all physical memory.
pub(crate) fn phys_offset() -> u64 {
    let offset = PHYS_OFFSET.load(Ordering::Relaxed);
    assert!(offset != 0, "Limine didn't map physical memory");
    offset
}

// The framebuffer's virtual addresses, which boot_info records. Empty when
// there is no framebuffer the console draws, as before then.
static FRAMEBUFFER_START: AtomicU64 = AtomicU64::new(0);
static FRAMEBUFFER_END: AtomicU64 = AtomicU64::new(0);

/// The framebuffer's virtual addresses, or an empty range without one.
pub(crate) fn framebuffer_range() -> Range<u64> {
    FRAMEBUFFER_START.load(Ordering::Relaxed)..FRAMEBUFFER_END.load(Ordering::Relaxed)
}

/// The kernel's code, between the symbols linker.ld defines around
/// `.text`.
pub(crate) fn text() -> Range<u64> {
    to_u64(addr_of!(__text_start).addr())..to_u64(addr_of!(__text_end).addr())
}

/// The boot stack's size. Its base is aligned to twice this, so that the
/// exception vectors can tell an overflow by one bit of SP.
const STACK_SIZE: usize = 64 << 10;

// Limine enters at EL1 with the MMU on, interrupts masked and SP_EL0 on a
// stack of its own, which has nothing unmapped below it. _start moves to
// SP_EL1, which the kernel runs on, on the boot stack, and clears the frame
// record registers so that a backtrace ends here. linker.ld gives the boot
// stack a segment of its own with an unmapped page below it, its guard.
global_asm!(
    r#"
    .section .boot_stack, "aw", %nobits
    .balign {align}
    __boot_stack_bottom:
    .space {size}
    __boot_stack_top:

    .section .text._start, "ax"
    .global _start
    _start:
        msr spsel, #1
        adrp x9, __boot_stack_top
        add x9, x9, :lo12:__boot_stack_top
        mov sp, x9
        mov x29, xzr
        mov x30, xzr
        bl __thisos_entry
    1:  wfi
        b 1b

    .global __boot_stack_bottom
    .global __boot_stack_top
    "#,
    align = const 2 * STACK_SIZE,
    size = const STACK_SIZE,
);

unsafe extern "C" {
    // Only their addresses are used.
    safe static __boot_stack_bottom: u8;
    safe static __boot_stack_top: u8;
    safe static __text_start: u8;
    safe static __text_end: u8;
}

/// The second framebuffer handle, until a panic takes it.
pub(crate) static PANIC_FRAMEBUFFER: spin::Mutex<Option<Framebuffer<FramebufferPixels>>> =
    spin::Mutex::new(None);

/// Builds the kernel's [`BootInfo`] from Limine's answers, and sets up the
/// devices it holds.
///
/// # Safety
///
/// Call this once, from the entry point, on the boot stack, before anything
/// else reads Limine's answers or uses the devices they name.
#[doc(hidden)]
pub unsafe fn boot_info(_trap: TrapHandler<Aarch64>) -> BootInfo<Aarch64> {
    let bottom = addr_of!(__boot_stack_bottom).addr();
    let top = addr_of!(__boot_stack_top).addr();
    backtrace::init(to_u64(bottom)..to_u64(top));
    // Before anything that can panic, so that a panic finds the panic
    // path's handle to the screen.
    let framebuffer = FRAMEBUFFER
        .response()
        .and_then(|response| response.framebuffers().first().copied())
        .and_then(framebuffer);
    assert!(
        BASE_REVISION.is_supported(),
        "Limine doesn't support base revision {}",
        BaseRevision::MAX_SUPPORTED
    );
    let Some(memory) = MEMORY_MAP.response() else {
        panic!("Limine gave no memory map");
    };
    let Some(hhdm) = HHDM.response() else {
        panic!("Limine didn't map physical memory");
    };
    PHYS_OFFSET.store(hhdm.offset, Ordering::Relaxed);
    if let Some(fb) = &framebuffer {
        let range = fb.pixels.range();
        FRAMEBUFFER_START.store(range.start, Ordering::Relaxed);
        FRAMEBUFFER_END.store(range.end, Ordering::Relaxed);
    }
    BootInfo {
        serial: None,
        // take_page_tables makes it normal non-cacheable, which gathers
        // stores as write-combining does.
        framebuffer_write_combining: framebuffer.is_some(),
        framebuffer,
        memory: MemoryMap::new(memory.entries()),
    }
}

fn framebuffer(fb: &limine::framebuffer::Framebuffer) -> Option<Framebuffer<FramebufferPixels>> {
    if fb.bpp != 32 || fb.memory_model != limine::framebuffer::FRAMEBUFFER_RGB {
        return None;
    }
    // A pixel is a little-endian word, so a red shift of 0 puts red in the
    // first byte.
    let format = match (fb.red_mask_shift, fb.green_mask_shift, fb.blue_mask_shift) {
        (0, 8, 16) => PixelFormat::Rgb,
        (16, 8, 0) => PixelFormat::Bgr,
        _ => return None,
    };
    let width = usize::try_from(fb.width).ok()?;
    let height = usize::try_from(fb.height).ok()?;
    let pitch = usize::try_from(fb.pitch).ok()?;
    if pitch % 4 != 0 || width > pitch / 4 {
        return None;
    }
    let size = pitch.checked_mul(height)?;
    // Limine maps the framebuffer in its physical-memory mapping and gives
    // that address.
    let start = ptr_at::<u8>(to_u64(fb.address().addr()));
    // SAFETY: Limine maps the framebuffer for the kernel's life, boot_info
    // runs once and hands these handles to nothing else, and no kernel code
    // reads or writes the framebuffer's frames through the physical-memory
    // mapping. The limine crate's as_slice functions, which would make
    // references to it, are never called.
    let [pixels, panic_pixels] =
        unsafe { FramebufferPixels::new(start..start.wrapping_add(size))? };
    let framebuffer = |pixels| Framebuffer {
        pixels,
        width,
        height,
        stride: pitch / 4,
        format,
    };
    *PANIC_FRAMEBUFFER.lock() = Some(framebuffer(panic_pixels));
    Some(framebuffer(pixels))
}
