//! The kernel's frame allocator, seeded from the Usable regions of the
//! loader's memory map. Trap handlers never call it: its lock is a spin
//! lock, and a handler that found it held by the code it interrupted would
//! never return.

#[cfg(test)]
use core::sync::atomic::{AtomicUsize, Ordering};

use spin::Mutex;
use thisos_frames::{Frame, Frames};
use thisos_hal::{MemoryKind, Platform};

use crate::Arch;

type MemoryMap = <Arch as Platform>::MemoryMap;

/// Physical memory up to 64 GiB: bytes, over 4 KiB frames, over 64 frames
/// a word. That is a 2 MiB bitmap.
const WORDS: usize = (64 << 30) / 4096 / 64;

static FRAMES: Mutex<Frames<WORDS>> = Mutex::new(Frames::new());

/// The memory map, for reading it again after `init`.
static MEMORY: Mutex<Option<MemoryMap>> = Mutex::new(None);

/// The free frames right after `init`, before the page tables took some.
#[cfg(test)]
static SEEDED: AtomicUsize = AtomicUsize::new(0);

/// Makes the Usable regions of `memory` free, and logs how much memory
/// that is. `kernel::init` calls this once.
pub fn init(memory: MemoryMap) {
    let mut frames = FRAMES.lock();
    let (mut regions, mut above) = (0, 0);
    for region in memory.clone().filter(|r| r.kind == MemoryKind::Usable) {
        regions += 1;
        above += frames.add(region.start..region.end);
    }
    #[cfg(test)]
    SEEDED.store(frames.free_frames(), Ordering::Relaxed);
    // 256 frames to the MiB.
    let free = frames.free_frames() / 256;
    log::info!("memory: {free} MiB free in {regions} usable regions");
    if above > 0 {
        log::warn!(
            "memory: {} MiB above {} GiB ignored",
            above >> 20,
            frames.limit() >> 30
        );
    }
    *MEMORY.lock() = Some(memory);
}

/// Takes a free frame, or returns `None` when there is none.
pub fn alloc() -> Option<Frame> {
    FRAMES.lock().alloc()
}

/// Makes `frame` free again.
pub fn free(frame: Frame) {
    FRAMES.lock().free(frame);
}

/// The number of free frames.
#[cfg(test)]
pub fn free_frames() -> usize {
    FRAMES.lock().free_frames()
}

/// The memory map `init` was given.
#[cfg(test)]
pub fn memory() -> MemoryMap {
    let Some(memory) = MEMORY.lock().clone() else {
        panic!("frames::init didn't run");
    };
    memory
}

#[cfg(test)]
mod tests {
    use thisos_frames::{FLOOR, FRAME_SIZE};

    use super::*;

    const BATCH: usize = 4096;

    static HELD: Mutex<[Option<Frame>; BATCH]> = Mutex::new([const { None }; BATCH]);

    #[test_case]
    fn seeded_frames_match_the_memory_map() {
        let limit = FRAMES.lock().limit();
        let expected: u64 = memory()
            .filter(|r| r.kind == MemoryKind::Usable)
            .map(|r| {
                let first = r.start.max(FLOOR).div_ceil(FRAME_SIZE);
                let end = r.end.min(limit) / FRAME_SIZE;
                end.saturating_sub(first)
            })
            .sum();
        let seeded = SEEDED.load(Ordering::Relaxed);
        assert_eq!(u64::try_from(seeded), Ok(expected));
    }

    #[test_case]
    fn frames_are_distinct_usable_and_reused() {
        let before = FRAMES.lock().free_frames();
        let mut held = HELD.lock();
        for _ in 0..2 {
            for slot in held.iter_mut() {
                *slot = alloc();
            }
            assert_eq!(FRAMES.lock().free_frames(), before - BATCH);
            held.sort_unstable_by_key(|f| f.as_ref().map(Frame::start));
            let mut last = None;
            for frame in held.iter() {
                let Some(frame) = frame else {
                    panic!("fewer than {BATCH} frames free");
                };
                let start = frame.start();
                assert!(last < Some(start), "frame {start:#x} handed out twice");
                assert!(
                    start % FRAME_SIZE == 0 && start >= FLOOR,
                    "frame {start:#x}"
                );
                assert!(
                    memory().any(|r| r.kind == MemoryKind::Usable
                        && r.start <= start
                        && start + FRAME_SIZE <= r.end),
                    "frame {start:#x} outside the usable regions"
                );
                last = Some(start);
            }
            for slot in held.iter_mut() {
                if let Some(frame) = slot.take() {
                    free(frame);
                }
            }
            assert_eq!(FRAMES.lock().free_frames(), before);
        }
    }
}
