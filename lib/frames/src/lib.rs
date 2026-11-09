//! The physical frame allocator: a bitmap with one bit per 4 KiB frame, set
//! while the frame is free. It covers physical memory from 0 up to a limit
//! fixed by its size, so that it needs no memory of its own before there is
//! any to allocate from.

#![no_std]
// Denied, not forbidden, for `Frame::reclaim` alone, which declares an
// unsafe function and has no unsafe block.
#![deny(unsafe_code)]

use core::ops::Range;

/// The size of a frame, in bytes.
pub const FRAME_SIZE: u64 = 4096;

/// Frames below 1 MiB are never handed out: frame 0, and the real-mode
/// memory that starting another core needs.
pub const FLOOR: u64 = 0x10_0000;

/// A frame of physical memory, which its holder may use. Only [`Frames`]
/// makes one, and it never hands out a frame that is already held, apart
/// from [`Frame::reclaim`], which gives back one that page tables held.
/// Dropping a `Frame` without [`Frames::free`] leaks the frame.
#[derive(Debug)]
#[must_use = "dropping a Frame leaks it"]
pub struct Frame(u64);

impl Frame {
    /// The frame's physical address.
    pub fn start(&self) -> u64 {
        self.0
    }

    /// Gives up the `Frame` for its physical address, for a holder that
    /// keeps only the address, as a page-table entry does. Only
    /// [`Frame::reclaim`] makes a `Frame` of it again.
    pub fn into_start(self) -> u64 {
        self.0
    }

    /// The frame at `start` again, from a holder that kept only its
    /// address.
    ///
    /// # Safety
    ///
    /// `start` must come from [`Frame::into_start`], and nothing may hold
    /// the frame any more: no other `Frame` for it exists, and the holder
    /// that kept the address no longer uses it.
    #[expect(
        unsafe_code,
        reason = "reclaiming makes a Frame that Frames didn't hand out, so the caller has to show nothing else holds it"
    )]
    pub unsafe fn reclaim(start: u64) -> Self {
        Self(start)
    }
}

/// The free frames among the first `WORDS * 64`.
#[derive(Debug)]
pub struct Frames<const WORDS: usize> {
    words: [u64; WORDS],
    /// No word before this one has a free frame.
    next: usize,
    free: usize,
}

impl<const WORDS: usize> Frames<WORDS> {
    /// An allocator with no free frames. It is all zeroes, so a static one
    /// takes no space in the kernel image.
    pub const fn new() -> Self {
        Self {
            words: [0; WORDS],
            next: 0,
            free: 0,
        }
    }

    /// The address after the last frame it covers.
    pub fn limit(&self) -> u64 {
        address(WORDS * 64)
    }

    /// The number of free frames.
    pub fn free_frames(&self) -> usize {
        self.free
    }

    /// Makes free every whole frame in `range` from [`FLOOR`] up to
    /// [`Frames::limit`], and returns the number of bytes of `range` above
    /// the limit, which it leaves out. Panics if one of the frames is free
    /// already, which means two regions of the memory map overlap.
    pub fn add(&mut self, range: Range<u64>) -> u64 {
        let limit = self.limit();
        let above = range.end.saturating_sub(range.start.max(limit));
        let first = range.start.max(FLOOR).div_ceil(FRAME_SIZE);
        let end = range.end.min(limit) / FRAME_SIZE;
        for frame in first..end {
            let (word, bit) = position(frame);
            assert!(
                self.words[word] & bit == 0,
                "frame {:#x} added twice",
                frame * FRAME_SIZE
            );
            self.words[word] |= bit;
            self.free += 1;
            self.next = self.next.min(word);
        }
        above
    }

    /// Takes the free frame with the lowest address, or returns `None` when
    /// none is free.
    pub fn alloc(&mut self) -> Option<Frame> {
        let Some(offset) = self.words[self.next..].iter().position(|&w| w != 0) else {
            self.next = WORDS;
            return None;
        };
        let word = self.next + offset;
        self.next = word;
        let bit = self.words[word].trailing_zeros();
        self.words[word] &= !(1 << bit);
        self.free -= 1;
        Some(Frame(address(word * 64) + u64::from(bit) * FRAME_SIZE))
    }

    /// Makes `frame` free again. Panics if it is free already, which only a
    /// frame from a second allocator can cause, or if this allocator doesn't
    /// cover it.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "taking the Frame stops its holder from using it after the free"
    )]
    pub fn free(&mut self, frame: Frame) {
        let (word, bit) = position(frame.0 / FRAME_SIZE);
        assert!(
            self.words[word] & bit == 0,
            "frame {:#x} freed twice",
            frame.0
        );
        self.words[word] |= bit;
        self.free += 1;
        self.next = self.next.min(word);
    }
}

impl<const WORDS: usize> Default for Frames<WORDS> {
    fn default() -> Self {
        Self::new()
    }
}

/// The address of frame number `frame`.
fn address(frame: usize) -> u64 {
    u64::try_from(frame).expect("usize is 64 bits on every ThisOS target") * FRAME_SIZE
}

/// The word and the bit within it of frame number `frame`.
fn position(frame: u64) -> (usize, u64) {
    let word = usize::try_from(frame / 64).expect("usize is 64 bits on every ThisOS target");
    (word, 1 << (frame % 64))
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec::Vec;

    use super::*;

    /// 512 frames: a limit of 2 MiB, and 256 frames above the floor.
    type Small = Frames<8>;

    fn drain(frames: &mut Small) -> Vec<u64> {
        let mut starts = Vec::new();
        while let Some(frame) = frames.alloc() {
            starts.push(frame.start());
        }
        starts
    }

    #[test]
    fn add_takes_whole_frames_between_floor_and_limit() {
        let mut frames = Small::new();
        assert_eq!(frames.add(0x8_0000..0x10_2800), 0);
        assert_eq!(frames.add(0x1f_e800..0x30_0000), 0x10_0000);
        assert_eq!(frames.add(0x40_0000..0x50_0000), 0x10_0000);
        assert_eq!(frames.free_frames(), 3);
        assert_eq!(drain(&mut frames), [0x10_0000, 0x10_1000, 0x1f_f000]);
    }

    #[test]
    fn alloc_hands_out_each_frame_once() {
        let mut frames = Small::new();
        frames.add(0..0x20_0000);
        assert_eq!(frames.free_frames(), 256);
        let mut starts = drain(&mut frames);
        assert_eq!(starts.len(), 256);
        assert_eq!(frames.free_frames(), 0);
        assert!(starts.iter().all(|s| s % FRAME_SIZE == 0 && *s >= FLOOR));
        starts.sort_unstable();
        starts.dedup();
        assert_eq!(starts.len(), 256);
        assert!(frames.alloc().is_none());
    }

    #[test]
    fn freed_frames_are_reused() {
        let mut frames = Small::new();
        frames.add(0x10_0000..0x10_4000);
        let [a, b, c, d] = [(); 4].map(|()| frames.alloc().unwrap());
        assert!(frames.alloc().is_none());
        let start = b.start();
        frames.free(b);
        assert_eq!(frames.free_frames(), 1);
        assert_eq!(frames.alloc().unwrap().start(), start);
        frames.free(d);
        frames.free(a);
        assert_eq!(drain(&mut frames), [0x10_0000, 0x10_3000]);
        frames.free(c);
        assert_eq!(frames.free_frames(), 1);
    }

    #[test]
    #[should_panic(expected = "frame 0x100000 freed twice")]
    fn double_free_panics() {
        let mut first = Small::new();
        let mut second = Small::new();
        first.add(0x10_0000..0x10_1000);
        second.add(0x10_0000..0x10_1000);
        let a = first.alloc().unwrap();
        let b = second.alloc().unwrap();
        first.free(a);
        first.free(b);
    }

    #[test]
    #[should_panic(expected = "frame 0x101000 added twice")]
    fn overlapping_regions_panic() {
        let mut frames = Small::new();
        frames.add(0x10_0000..0x10_2000);
        frames.add(0x10_1000..0x10_3000);
    }
}
