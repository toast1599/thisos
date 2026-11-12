//! A process's handle table: the objects the process can reach, each with
//! the rights its handle gives. A handle is a `u32` whose low 16 bits are
//! a slot's index and whose high 16 bits are the slot's generation, which
//! goes up each time the slot is freed, so a handle that was closed fails
//! instead of naming whatever took its slot next. Generations start at 1,
//! so 0 is never a handle, and a slot whose generation would wrap is
//! retired for good.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;

use thisos_abi::{Error, Rights};

/// The most slots a table has: every index a handle's 16 bits hold.
pub const MAX_SLOTS: usize = 1 << 16;

/// A handle table holding objects of type `T`.
pub struct Table<T> {
    slots: Vec<Slot<T>>,
    /// The first free slot, whose entry names the next.
    free: Option<u16>,
}

struct Slot<T> {
    generation: u16,
    entry: Entry<T>,
}

enum Entry<T> {
    Used { object: T, rights: Rights },
    Free { next: Option<u16> },
    Retired,
}

impl<T> Table<T> {
    /// An empty table, which allocates nothing.
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: None,
        }
    }

    /// Puts `object` in the table with `rights`, and returns its handle.
    /// Fails with [`Error::NoHandles`] if every slot is used or retired.
    pub fn insert(&mut self, object: T, rights: Rights) -> Result<u32, Error> {
        let index = if let Some(index) = self.free {
            let slot = &mut self.slots[usize::from(index)];
            let Entry::Free { next } = slot.entry else {
                panic!("free slot {index} isn't free");
            };
            self.free = next;
            slot.entry = Entry::Used { object, rights };
            index
        } else {
            let index = u16::try_from(self.slots.len()).map_err(|_| Error::NoHandles)?;
            self.slots.push(Slot {
                generation: 1,
                entry: Entry::Used { object, rights },
            });
            index
        };
        Ok(handle(index, self.slots[usize::from(index)].generation))
    }

    /// The object `handle` names and its rights.
    pub fn get(&self, handle: u32) -> Result<(&T, Rights), Error> {
        let (index, generation) = split(handle);
        match self.slots.get(usize::from(index)) {
            Some(Slot {
                generation: g,
                entry: Entry::Used { object, rights },
            }) if *g == generation => Ok((object, *rights)),
            _ => Err(Error::BadHandle),
        }
    }

    /// Closes `handle`, and returns its object and rights.
    pub fn remove(&mut self, handle: u32) -> Result<(T, Rights), Error> {
        self.get(handle)?;
        let (index, _) = split(handle);
        let slot = &mut self.slots[usize::from(index)];
        let entry = if let Some(generation) = slot.generation.checked_add(1) {
            slot.generation = generation;
            let next = self.free.replace(index);
            Entry::Free { next }
        } else {
            Entry::Retired
        };
        match core::mem::replace(&mut slot.entry, entry) {
            Entry::Used { object, rights } => Ok((object, rights)),
            Entry::Free { .. } | Entry::Retired => unreachable!("get found it used"),
        }
    }

    /// Leaves `handle` with only `rights`. Fails with
    /// [`Error::BadRights`], changing nothing, if it lacks any of them.
    pub fn restrict(&mut self, handle: u32, rights: Rights) -> Result<(), Error> {
        let (index, generation) = split(handle);
        match self.slots.get_mut(usize::from(index)) {
            Some(Slot {
                generation: g,
                entry: Entry::Used { rights: held, .. },
            }) if *g == generation => {
                if !held.contains(rights) {
                    return Err(Error::BadRights);
                }
                *held = rights;
                Ok(())
            }
            _ => Err(Error::BadHandle),
        }
    }
}

impl<T: Clone> Table<T> {
    /// Makes a new handle to `handle`'s object with `rights`, and returns
    /// it. Fails with [`Error::AccessDenied`] if `handle` lacks
    /// [`Rights::DUPLICATE`], and with [`Error::BadRights`] if it lacks any
    /// of `rights`.
    pub fn duplicate(&mut self, handle: u32, rights: Rights) -> Result<u32, Error> {
        let (object, held) = self.get(handle)?;
        if !held.contains(Rights::DUPLICATE) {
            return Err(Error::AccessDenied);
        }
        if !held.contains(rights) {
            return Err(Error::BadRights);
        }
        let object = object.clone();
        self.insert(object, rights)
    }
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn handle(index: u16, generation: u16) -> u32 {
    u32::from(generation) << 16 | u32::from(index)
}

/// The index and generation in `handle`.
fn split(handle: u32) -> (u16, u16) {
    let [a, b, c, d] = handle.to_le_bytes();
    (u16::from_le_bytes([a, b]), u16::from_le_bytes([c, d]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_BUT_DUPLICATE: Rights = Rights::READ.union(Rights::WRITE).union(Rights::TRANSFER);

    #[test]
    fn a_handle_names_its_object_until_closed() {
        let mut table = Table::new();
        let a = table.insert('a', Rights::ALL).unwrap();
        let b = table.insert('b', Rights::READ).unwrap();
        assert_ne!(a, 0);
        assert_eq!(table.get(a), Ok((&'a', Rights::ALL)));
        assert_eq!(table.get(b), Ok((&'b', Rights::READ)));
        assert_eq!(table.remove(a), Ok(('a', Rights::ALL)));
        assert_eq!(table.get(a), Err(Error::BadHandle));
        assert_eq!(table.remove(a), Err(Error::BadHandle));
        assert_eq!(table.get(b), Ok((&'b', Rights::READ)));
    }

    #[test]
    fn a_closed_handle_doesnt_name_what_takes_its_slot() {
        let mut table = Table::new();
        let old = table.insert('a', Rights::ALL).unwrap();
        table.remove(old).unwrap();
        let new = table.insert('b', Rights::ALL).unwrap();
        assert_eq!(new & 0xffff, old & 0xffff, "the slot is reused");
        assert_eq!(table.get(old), Err(Error::BadHandle));
        assert_eq!(table.get(new), Ok((&'b', Rights::ALL)));
    }

    #[test]
    fn values_no_slot_has_are_bad() {
        let mut table = Table::new();
        let a = table.insert('a', Rights::ALL).unwrap();
        for handle in [0, a & 0xffff, a + 1, u32::MAX] {
            assert_eq!(table.get(handle), Err(Error::BadHandle), "{handle:#x}");
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "65536 table operations take Miri over ten minutes")]
    fn a_slot_is_retired_when_its_generation_would_wrap() {
        let mut table = Table::new();
        let mut handle = table.insert(0, Rights::ALL).unwrap();
        while handle >> 16 < u32::from(u16::MAX) {
            table.remove(handle).unwrap();
            handle = table.insert(0, Rights::ALL).unwrap();
            assert_eq!(handle & 0xffff, 0);
        }
        table.remove(handle).unwrap();
        let next = table.insert(0, Rights::ALL).unwrap();
        assert_eq!(next, 1 << 16 | 1, "slot 0 is retired");
        assert_eq!(table.get(handle), Err(Error::BadHandle));
    }

    #[test]
    #[cfg_attr(miri, ignore = "65536 table operations take Miri over ten minutes")]
    fn a_full_table_fails_and_a_closed_slot_makes_room() {
        let mut table = Table::new();
        let handles: Vec<u32> = (0..MAX_SLOTS)
            .map(|_| table.insert((), Rights::NONE).unwrap())
            .collect();
        assert_eq!(table.insert((), Rights::NONE), Err(Error::NoHandles));
        table.remove(handles[7]).unwrap();
        assert_eq!(table.insert((), Rights::NONE).map(|h| h & 0xffff), Ok(7));
    }

    #[test]
    fn duplicate_needs_its_right_and_gives_a_subset() {
        let mut table = Table::new();
        let a = table.insert('a', Rights::ALL).unwrap();
        let b = table.duplicate(a, Rights::READ).unwrap();
        assert_ne!(a, b);
        assert_eq!(table.get(b), Ok((&'a', Rights::READ)));
        assert_eq!(table.duplicate(b, Rights::READ), Err(Error::AccessDenied));
        let c = table.insert('c', Rights::DUPLICATE | Rights::READ).unwrap();
        assert_eq!(table.duplicate(c, Rights::WRITE), Err(Error::BadRights));
        let d = table.duplicate(c, Rights::NONE).unwrap();
        assert_eq!(table.get(d), Ok((&'c', Rights::NONE)));
    }

    #[test]
    fn restrict_only_drops_rights() {
        let mut table = Table::new();
        let a = table.insert('a', ALL_BUT_DUPLICATE).unwrap();
        assert_eq!(table.restrict(a, Rights::ALL), Err(Error::BadRights));
        assert_eq!(table.get(a), Ok((&'a', ALL_BUT_DUPLICATE)));
        table.restrict(a, Rights::READ).unwrap();
        assert_eq!(table.get(a), Ok((&'a', Rights::READ)));
        assert_eq!(table.restrict(a, Rights::WRITE), Err(Error::BadRights));
        assert_eq!(table.restrict(0, Rights::NONE), Err(Error::BadHandle));
    }
}
