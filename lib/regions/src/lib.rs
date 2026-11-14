//! What an address space's mappings and a memory object's uses keep track
//! of: [`Regions`] maps address ranges that never overlap to whatever each
//! mapping holds, and [`Uses`] counts an object's writable and executable
//! mappings, so that no object is mapped both ways at once.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use core::ops::Range;

use thisos_abi::Error;
use thisos_hal::Protection;

/// Address ranges that don't overlap, each with a value of type `T`.
pub struct Regions<T> {
    /// In order of start.
    regions: Vec<(Range<u64>, T)>,
}

impl<T> Regions<T> {
    /// No regions, which allocates nothing.
    pub const fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    /// Adds `range` with `value`, and returns the value where it now is.
    /// Fails, handing `value` back, if `range` is empty or overlaps a
    /// region.
    pub fn insert(&mut self, range: Range<u64>, value: T) -> Result<&mut T, T> {
        if range.is_empty() {
            return Err(value);
        }
        let i = self.regions.partition_point(|(r, _)| r.start < range.start);
        let before = i.checked_sub(1).map(|j| &self.regions[j].0);
        let after = self.regions.get(i).map(|(r, _)| r);
        if before.is_some_and(|r| r.end > range.start) || after.is_some_and(|r| r.start < range.end)
        {
            return Err(value);
        }
        self.regions.insert(i, (range, value));
        Ok(&mut self.regions[i].1)
    }

    /// Removes the region that starts at `start`, and returns its range and
    /// value, or `None` if no region starts there.
    pub fn remove(&mut self, start: u64) -> Option<(Range<u64>, T)> {
        let i = self
            .regions
            .binary_search_by_key(&start, |(r, _)| r.start)
            .ok()?;
        Some(self.regions.remove(i))
    }

    /// Removes every region, in order of address.
    pub fn take_all(&mut self) -> impl Iterator<Item = (Range<u64>, T)> {
        core::mem::take(&mut self.regions).into_iter()
    }

    /// Every region, in order of address.
    pub fn iter(&self) -> impl Iterator<Item = &(Range<u64>, T)> {
        self.regions.iter()
    }
}

impl<T> Default for Regions<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// An object's writable and executable mappings, by count.
#[derive(Debug, Default)]
pub struct Uses {
    writable: usize,
    executable: usize,
}

impl Uses {
    /// Counts a new mapping with `protection`. Fails with
    /// [`Error::Conflict`], counting nothing, if it is writable while an
    /// executable mapping exists, or the reverse.
    pub fn add(&mut self, protection: Protection) -> Result<(), Error> {
        match protection {
            Protection::ReadOnly => {}
            Protection::ReadWrite if self.executable > 0 => return Err(Error::Conflict),
            Protection::ReadWrite => self.writable += 1,
            Protection::ReadExecute if self.writable > 0 => return Err(Error::Conflict),
            Protection::ReadExecute => self.executable += 1,
        }
        Ok(())
    }

    /// Stops counting a mapping that [`Uses::add`] counted with
    /// `protection`.
    pub fn remove(&mut self, protection: Protection) {
        let count = match protection {
            Protection::ReadOnly => return,
            Protection::ReadWrite => &mut self.writable,
            Protection::ReadExecute => &mut self.executable,
        };
        let Some(less) = count.checked_sub(1) else {
            panic!("removing a {protection:?} use that wasn't added");
        };
        *count = less;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn starts(regions: &Regions<char>) -> Vec<(u64, char)> {
        regions.iter().map(|(r, v)| (r.start, *v)).collect()
    }

    #[test]
    fn regions_stay_in_order_and_never_overlap() {
        let mut regions = Regions::new();
        regions.insert(0x3000..0x5000, 'b').unwrap();
        regions.insert(0x1000..0x2000, 'a').unwrap();
        regions.insert(0x5000..0x6000, 'c').unwrap();
        regions.insert(0x2000..0x3000, 'd').unwrap();
        assert_eq!(
            starts(&regions),
            [(0x1000, 'a'), (0x2000, 'd'), (0x3000, 'b'), (0x5000, 'c')]
        );
        for range in [
            0x0..0x1001,
            0x4fff..0x5000,
            0x3000..0x5000,
            0x2fff..0x3001,
            0x0..0x7000,
            0x5fff..0x7000,
        ] {
            assert_eq!(regions.insert(range.clone(), 'x'), Err('x'), "{range:x?}");
        }
        assert_eq!(regions.insert(0x4000..0x4000, 'x'), Err('x'), "empty");
        assert_eq!(regions.insert(0x6000..0x7000, 'e'), Ok(&mut 'e'));
        assert_eq!(regions.iter().count(), 5);
    }

    #[test]
    fn remove_takes_only_a_region_that_starts_there() {
        let mut regions = Regions::new();
        regions.insert(0x1000..0x3000, 'a').unwrap();
        regions.insert(0x3000..0x4000, 'b').unwrap();
        assert_eq!(regions.remove(0x2000), None);
        assert_eq!(regions.remove(0x1000), Some((0x1000..0x3000, 'a')));
        assert_eq!(regions.remove(0x1000), None);
        regions.insert(0x2000..0x3000, 'c').unwrap();
        let all: Vec<_> = regions.take_all().collect();
        assert_eq!(all, [(0x2000..0x3000, 'c'), (0x3000..0x4000, 'b')]);
        assert_eq!(regions.iter().count(), 0);
    }

    #[test]
    fn an_object_is_never_writable_and_executable_at_once() {
        let mut uses = Uses::default();
        uses.add(Protection::ReadOnly).unwrap();
        uses.add(Protection::ReadWrite).unwrap();
        uses.add(Protection::ReadWrite).unwrap();
        assert_eq!(uses.add(Protection::ReadExecute), Err(Error::Conflict));
        uses.remove(Protection::ReadWrite);
        assert_eq!(uses.add(Protection::ReadExecute), Err(Error::Conflict));
        uses.remove(Protection::ReadWrite);
        uses.add(Protection::ReadExecute).unwrap();
        assert_eq!(uses.add(Protection::ReadWrite), Err(Error::Conflict));
        uses.add(Protection::ReadOnly).unwrap();
        uses.remove(Protection::ReadExecute);
        uses.add(Protection::ReadWrite).unwrap();
    }
}
