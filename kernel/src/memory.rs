//! Memory objects: zeroed frames that address spaces map, as many times
//! as handles and mappings ask. An object frees its frames when the last
//! handle and mapping holding it go, and an address space holds a mapping
//! until no page maps its frames, so no frame is freed while user code
//! reaches it. No object is mapped writable and executable at once.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::Deref;

use spin::Mutex;
use thisos_abi::Error;
use thisos_frames::Frame;
use thisos_hal::{Platform, Protection};
use thisos_regions::Uses;

use crate::{Arch, frames};

pub struct MemoryObject {
    frames: Vec<Frame>,
    /// Taken with interrupts on, and never by a trap handler: mapping and
    /// unmapping take it.
    uses: Mutex<Uses>,
}

impl MemoryObject {
    /// An object of `pages` zeroed frames. Fails with [`Error::NoMemory`]
    /// if the frames or the heap run short.
    pub fn new(pages: usize) -> Result<Self, Error> {
        let mut object = Self {
            frames: Vec::new(),
            uses: Mutex::new(Uses::default()),
        };
        object
            .frames
            .try_reserve_exact(pages)
            .map_err(|_| Error::NoMemory)?;
        for _ in 0..pages {
            // Dropping the object frees the frames taken so far.
            let mut frame = frames::alloc().ok_or(Error::NoMemory)?;
            Arch::fill_frame(&mut frame, 0, &[]);
            object.frames.push(frame);
        }
        Ok(object)
    }

    pub fn pages(&self) -> usize {
        self.frames.len()
    }

    /// The frames, to fill before anything shares or maps the object.
    pub fn frames_mut(&mut self) -> &mut [Frame] {
        &mut self.frames
    }
}

impl Drop for MemoryObject {
    fn drop(&mut self) {
        for frame in self.frames.drain(..) {
            frames::free(frame);
        }
    }
}

/// An address space's hold on the object one of its mappings maps, which
/// counts toward the object's writable or executable mappings until it
/// drops.
pub struct Mapping {
    object: Arc<MemoryObject>,
    protection: Protection,
}

impl Mapping {
    /// A hold on `object` for a mapping with `protection`. Fails with
    /// [`Error::Conflict`] if the object's other mappings forbid it.
    pub fn new(object: Arc<MemoryObject>, protection: Protection) -> Result<Self, Error> {
        object.uses.lock().add(protection)?;
        Ok(Self { object, protection })
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        self.object.uses.lock().remove(self.protection);
    }
}

impl Deref for Mapping {
    type Target = [Frame];

    fn deref(&self) -> &[Frame] {
        &self.object.frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn an_object_frees_its_frames_after_its_last_holder() {
        let before = frames::free_frames();
        let Ok(object) = MemoryObject::new(3) else {
            panic!("no memory for 3 pages");
        };
        assert_eq!(frames::free_frames(), before - 3);
        let object = Arc::new(object);
        let Ok(mapping) = Mapping::new(Arc::clone(&object), Protection::ReadOnly) else {
            panic!("a read-only mapping conflicted");
        };
        drop(object);
        assert_eq!(mapping.len(), 3);
        assert_eq!(frames::free_frames(), before - 3);
        drop(mapping);
        assert_eq!(frames::free_frames(), before);
    }

    #[test_case]
    fn an_object_too_big_for_memory_takes_nothing() {
        let before = frames::free_frames();
        assert!(matches!(
            MemoryObject::new(before + 1),
            Err(Error::NoMemory)
        ));
        assert_eq!(frames::free_frames(), before);
        assert!(matches!(
            MemoryObject::new(usize::MAX),
            Err(Error::NoMemory)
        ));
    }

    #[test_case]
    fn mappings_never_make_an_object_writable_and_executable() {
        let Ok(object) = MemoryObject::new(1) else {
            panic!("no memory for a page");
        };
        let object = Arc::new(object);
        let map = |protection| Mapping::new(Arc::clone(&object), protection);
        let Ok(writable) = map(Protection::ReadWrite) else {
            panic!("the first mapping conflicted");
        };
        assert!(matches!(map(Protection::ReadExecute), Err(Error::Conflict)));
        drop(writable);
        assert!(map(Protection::ReadExecute).is_ok());
    }
}
