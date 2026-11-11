//! A reader for the ELF files the kernel loads: little-endian 64-bit
//! executables for x86_64, linked at a fixed address, which need no
//! relocations. [`Elf::new`] checks the header and every program header
//! once, so reading the segments afterwards can't fail.

#![no_std]
#![forbid(unsafe_code)]

use core::ops::Range;

/// The size of the pages segments are loaded into.
pub const PAGE_SIZE: u64 = 4096;

const HEADER_SIZE: usize = 64;
const PROGRAM_HEADER_SIZE: usize = 56;
const MAGIC: &[u8] = b"\x7fELF";
const CLASS_64: u8 = 2;
const LITTLE_ENDIAN: u8 = 1;
const VERSION: u8 = 1;
const EXECUTABLE: u16 = 2;
const X86_64: u16 = 62;

// Program header types.
const LOAD: u32 = 1;
const DYNAMIC: u32 = 2;
const INTERP: u32 = 3;
const TLS: u32 = 7;

// Program header flags.
const EXECUTE: u32 = 1;
const WRITE: u32 = 2;

/// What is wrong with a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The file ends before its header, its program headers or a load
    /// segment's bytes do.
    Truncated,
    /// The file doesn't start with the ELF magic number.
    BadMagic,
    /// The file is valid ELF but not what the kernel loads, as the named
    /// field or segment shows.
    Unsupported(&'static str),
    /// Program header `n` is a load segment with more bytes in the file
    /// than in memory, no bytes in memory, or pages past the end of the
    /// address space.
    BadSegment(usize),
    /// Program header `n` is a load segment both writable and executable.
    WritableAndExecutable(usize),
    /// Program header `n` is a load segment whose pages overlap or come
    /// before those of the load segment before it.
    Overlap(usize),
    /// The entry point isn't in an executable load segment.
    BadEntry(u64),
}

fn le<const N: usize>(bytes: &[u8], at: usize) -> Option<[u8; N]> {
    bytes.get(at..at.checked_add(N)?)?.try_into().ok()
}

fn le16(bytes: &[u8], at: usize) -> Option<u16> {
    le(bytes, at).map(u16::from_le_bytes)
}

fn le32(bytes: &[u8], at: usize) -> Option<u32> {
    le(bytes, at).map(u32::from_le_bytes)
}

fn le64(bytes: &[u8], at: usize) -> Option<u64> {
    le(bytes, at).map(u64::from_le_bytes)
}

/// A checked ELF file.
#[derive(Clone, Copy)]
pub struct Elf<'a> {
    file: &'a [u8],
    program_headers: &'a [u8],
    entry: u64,
}

impl<'a> Elf<'a> {
    /// Checks `file`'s header and its program headers.
    pub fn new(file: &'a [u8]) -> Result<Self, Error> {
        let header = file.get(..HEADER_SIZE).ok_or(Error::Truncated)?;
        if !header.starts_with(MAGIC) {
            return Err(Error::BadMagic);
        }
        let field = |at| le16(header, at).ok_or(Error::Truncated);
        if header[4] != CLASS_64 {
            return Err(Error::Unsupported("class"));
        }
        if header[5] != LITTLE_ENDIAN {
            return Err(Error::Unsupported("byte order"));
        }
        if header[6] != VERSION || le32(header, 20) != Some(VERSION.into()) {
            return Err(Error::Unsupported("version"));
        }
        if field(16)? != EXECUTABLE {
            return Err(Error::Unsupported("type"));
        }
        if field(18)? != X86_64 {
            return Err(Error::Unsupported("machine"));
        }
        if usize::from(field(54)?) != PROGRAM_HEADER_SIZE {
            return Err(Error::Unsupported("program header size"));
        }
        let offset = le64(header, 32).ok_or(Error::Truncated)?;
        let offset = usize::try_from(offset).map_err(|_| Error::Truncated)?;
        let size = usize::from(field(56)?) * PROGRAM_HEADER_SIZE;
        let end = offset.checked_add(size).ok_or(Error::Truncated)?;
        let elf = Elf {
            file,
            program_headers: file.get(offset..end).ok_or(Error::Truncated)?,
            entry: le64(header, 24).ok_or(Error::Truncated)?,
        };
        elf.check()?;
        Ok(elf)
    }

    /// Checks every program header: load segments inside the file, in
    /// order and on pages of their own, none writable and executable, one
    /// of them holding the entry point, and nothing that needs dynamic
    /// linking or thread-local storage.
    fn check(&self) -> Result<(), Error> {
        let mut previous_end = 0;
        let mut entry_found = false;
        for (n, header) in self
            .program_headers
            .as_chunks::<PROGRAM_HEADER_SIZE>()
            .0
            .iter()
            .enumerate()
        {
            let Some(segment) = segment(self.file, n, header)? else {
                continue;
            };
            if segment.writable && segment.executable {
                return Err(Error::WritableAndExecutable(n));
            }
            let pages = segment.page_range();
            if pages.start < previous_end {
                return Err(Error::Overlap(n));
            }
            previous_end = pages.end;
            entry_found |= segment.executable && segment.contains(self.entry);
        }
        if !entry_found {
            return Err(Error::BadEntry(self.entry));
        }
        Ok(())
    }

    /// Where the program starts.
    pub fn entry(&self) -> u64 {
        self.entry
    }

    /// The load segments, in order of address.
    pub fn segments(&self) -> impl Iterator<Item = Segment<'a>> {
        let file = self.file;
        // new checked every header, so segment never fails here.
        self.program_headers
            .as_chunks::<PROGRAM_HEADER_SIZE>()
            .0
            .iter()
            .enumerate()
            .filter_map(move |(n, header)| segment(file, n, header).ok().flatten())
    }
}

/// Reads program header `n`: `None` for a type the loader ignores, and an
/// error for a type it can't load, or a load segment outside the file or
/// the address space.
fn segment<'a>(file: &'a [u8], n: usize, header: &[u8]) -> Result<Option<Segment<'a>>, Error> {
    let word = |at| le64(header, at).ok_or(Error::Truncated);
    let (Some(kind), Some(flags)) = (le32(header, 0), le32(header, 4)) else {
        return Err(Error::Truncated);
    };
    match kind {
        LOAD => {}
        DYNAMIC => return Err(Error::Unsupported("dynamic segment")),
        INTERP => return Err(Error::Unsupported("interpreter")),
        TLS => return Err(Error::Unsupported("thread-local storage")),
        _ => return Ok(None),
    }
    let (offset, address, file_size, size) = (word(8)?, word(16)?, word(32)?, word(40)?);
    if file_size > size
        || size == 0
        || address
            .checked_add(size)
            .and_then(|end| end.checked_next_multiple_of(PAGE_SIZE))
            .is_none()
    {
        return Err(Error::BadSegment(n));
    }
    // A segment of .bss alone takes nothing from the file, and linkers may
    // give it any offset, past the file's end included.
    let data = if file_size == 0 {
        &[]
    } else {
        let start = usize::try_from(offset).map_err(|_| Error::Truncated)?;
        let end = start
            .checked_add(usize::try_from(file_size).map_err(|_| Error::Truncated)?)
            .ok_or(Error::Truncated)?;
        file.get(start..end).ok_or(Error::Truncated)?
    };
    Ok(Some(Segment {
        address,
        size,
        data,
        writable: flags & WRITE != 0,
        executable: flags & EXECUTE != 0,
    }))
}

/// A load segment of an [`Elf`]: `size` bytes at `address`, the first of
/// them from the file and the rest zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment<'a> {
    /// The address of the first byte.
    pub address: u64,
    /// The bytes in memory.
    pub size: u64,
    /// The bytes the file holds for the start of the segment.
    pub data: &'a [u8],
    /// Whether the program may write it.
    pub writable: bool,
    /// Whether the program may run it.
    pub executable: bool,
}

impl<'a> Segment<'a> {
    /// The pages the segment touches.
    pub fn page_range(&self) -> Range<u64> {
        let start = self.address - self.address % PAGE_SIZE;
        // Elf::new checked that neither overflows.
        start..(self.address + self.size).next_multiple_of(PAGE_SIZE)
    }

    fn contains(&self, address: u64) -> bool {
        (self.address..self.address + self.size).contains(&address)
    }

    /// Each page the segment touches, with the file's bytes for it.
    pub fn pages(&self) -> impl Iterator<Item = Page<'a>> {
        let segment = *self;
        // Elf::new read the length from a u64 no larger than size.
        let file_end = segment.address + u64::try_from(segment.data.len()).unwrap_or(segment.size);
        self.page_range().step_by(4096).map(move |page| {
            let start = page.max(segment.address);
            let end = (page + PAGE_SIZE).min(file_end);
            if start >= end {
                return Page {
                    address: page,
                    offset: 0,
                    data: &[],
                };
            }
            // Both are under a page, or inside data, so they fit a usize.
            let index = |address: u64, base: u64| usize::try_from(address - base).unwrap_or(0);
            Page {
                address: page,
                offset: index(start, page),
                data: &segment.data[index(start, segment.address)..index(end, segment.address)],
            }
        })
    }
}

/// One page of a [`Segment`]: `data` belongs `offset` bytes into the page
/// at `address`, and every other byte of it is zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Page<'a> {
    /// The page's address.
    pub address: u64,
    /// Where `data` starts in the page.
    pub offset: usize,
    /// The file's bytes for the page.
    pub data: &'a [u8],
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec;
    use std::vec::Vec;

    use super::*;

    const R: u32 = 4;
    const RX: u32 = R | EXECUTE;
    const RW: u32 = R | WRITE;

    /// A program header, by its fields in file order but `paddr`, which is
    /// `address` again.
    #[derive(Clone, Copy)]
    struct Header {
        kind: u32,
        flags: u32,
        offset: u64,
        address: u64,
        file_size: u64,
        size: u64,
    }

    fn load(flags: u32, offset: u64, address: u64, file_size: u64, size: u64) -> Header {
        Header {
            kind: LOAD,
            flags,
            offset,
            address,
            file_size,
            size,
        }
    }

    /// A file with `headers` right after its header, `entry`, and `len`
    /// bytes in all, each byte past the headers its offset's low byte.
    fn file(entry: u64, headers: &[Header], len: usize) -> Vec<u8> {
        let mut file: Vec<u8> = (0..len).map(|i| i.to_le_bytes()[0]).collect();
        let mut header = vec![0; HEADER_SIZE];
        header[..4].copy_from_slice(MAGIC);
        header[4] = CLASS_64;
        header[5] = LITTLE_ENDIAN;
        header[6] = VERSION;
        header[16..18].copy_from_slice(&EXECUTABLE.to_le_bytes());
        header[18..20].copy_from_slice(&X86_64.to_le_bytes());
        header[20..24].copy_from_slice(&1u32.to_le_bytes());
        header[24..32].copy_from_slice(&entry.to_le_bytes());
        header[32..40].copy_from_slice(&64u64.to_le_bytes());
        header[52..54].copy_from_slice(&64u16.to_le_bytes());
        header[54..56].copy_from_slice(&56u16.to_le_bytes());
        header[56..58].copy_from_slice(&u16::try_from(headers.len()).unwrap().to_le_bytes());
        for h in headers {
            header.extend(h.kind.to_le_bytes());
            header.extend(h.flags.to_le_bytes());
            for word in [
                h.offset,
                h.address,
                h.address,
                h.file_size,
                h.size,
                PAGE_SIZE,
            ] {
                header.extend(word.to_le_bytes());
            }
        }
        file[..header.len()].copy_from_slice(&header);
        file
    }

    /// Laid out as lld lays out init: read-only data, code and data with
    /// .bss, each at the file offset's place in its page, and a header the
    /// loader ignores.
    fn init() -> Vec<u8> {
        let headers = [
            Header {
                kind: 6,
                ..load(R, 0x40, 0x20_0040, 0x118, 0x118)
            },
            load(R, 0, 0x20_0000, 0x1f4, 0x1f4),
            load(RX, 0x200, 0x20_1200, 0x23, 0x23),
            load(RW, 0x228, 0x20_2228, 0x8, 0xdd8),
        ];
        file(0x20_1208, &headers, 0x300)
    }

    #[test]
    fn reads_an_executable_like_init() {
        let file = init();
        let elf = Elf::new(&file).unwrap();
        assert_eq!(elf.entry(), 0x20_1208);
        let segments: Vec<_> = elf.segments().collect();
        assert_eq!(segments.len(), 3);
        let code = segments[1];
        assert_eq!(code.address, 0x20_1200);
        assert_eq!(code.data, &file[0x200..0x223]);
        assert!(code.executable && !code.writable);
        let data = segments[2];
        assert!(data.writable && !data.executable);
        assert_eq!(data.page_range(), 0x20_2000..0x20_3000);
        let pages: Vec<_> = data.pages().collect();
        assert_eq!(
            pages,
            [Page {
                address: 0x20_2000,
                offset: 0x228,
                data: &file[0x228..0x230],
            }]
        );
    }

    #[test]
    fn splits_a_segment_into_pages() {
        // 0x1800 file bytes from the middle of a page, then 0x2000 zero
        // bytes, over four pages.
        let headers = [load(RX, 0x100, 0x40_0800, 0x1800, 0x3800)];
        let file = file(0x40_0800, &headers, 0x1900);
        let elf = Elf::new(&file).unwrap();
        let pages: Vec<_> = elf.segments().next().unwrap().pages().collect();
        assert_eq!(
            pages,
            [
                Page {
                    address: 0x40_0000,
                    offset: 0x800,
                    data: &file[0x100..0x900],
                },
                Page {
                    address: 0x40_1000,
                    offset: 0,
                    data: &file[0x900..0x1900],
                },
                Page {
                    address: 0x40_2000,
                    offset: 0,
                    data: &[],
                },
                Page {
                    address: 0x40_3000,
                    offset: 0,
                    data: &[],
                },
            ]
        );
    }

    #[test]
    fn a_segment_of_bss_alone_may_start_past_the_file() {
        let headers = [
            load(RX, 0, 0x1000, 0x10, 0x10),
            load(RW, 0x1000, 0x2000, 0, 0x100),
        ];
        let file = file(0x1000, &headers, 0x300);
        let elf = Elf::new(&file).unwrap();
        let bss = elf.segments().nth(1).unwrap();
        assert_eq!(bss.data, &[]);
        assert_eq!(
            bss.pages().collect::<Vec<_>>(),
            [Page {
                address: 0x2000,
                offset: 0,
                data: &[],
            }]
        );
    }

    #[test]
    fn rejects_bad_headers() {
        let good = init();
        assert_eq!(Elf::new(&good[..63]).err(), Some(Error::Truncated));
        let changed = |at: usize, bytes: &[u8]| {
            let mut file = good.clone();
            file[at..at + bytes.len()].copy_from_slice(bytes);
            Elf::new(&file).err()
        };
        assert_eq!(changed(1, b"X"), Some(Error::BadMagic));
        assert_eq!(changed(4, &[1]), Some(Error::Unsupported("class")));
        assert_eq!(changed(5, &[2]), Some(Error::Unsupported("byte order")));
        assert_eq!(changed(20, &[2]), Some(Error::Unsupported("version")));
        // A position-independent executable.
        assert_eq!(changed(16, &[3]), Some(Error::Unsupported("type")));
        assert_eq!(changed(18, &[183]), Some(Error::Unsupported("machine")));
        assert_eq!(
            changed(54, &[64]),
            Some(Error::Unsupported("program header size"))
        );
        // Program headers past the end of the file.
        assert_eq!(changed(56, &[100]), Some(Error::Truncated));
        assert_eq!(changed(32, &[0xff; 8]), Some(Error::Truncated));
        assert_eq!(changed(24, &[0; 8]), Some(Error::BadEntry(0)));
    }

    #[test]
    fn rejects_bad_segments() {
        let check = |headers: &[Header], entry| Elf::new(&file(entry, headers, 0x300)).err();
        let code = load(RX, 0, 0x1000, 0x10, 0x10);
        assert_eq!(check(&[code], 0x1000), None);
        assert_eq!(check(&[code], 0x1010), Some(Error::BadEntry(0x1010)));
        // The entry point in a segment that isn't executable.
        let data = load(RW, 0, 0x2000, 0x10, 0x10);
        assert_eq!(check(&[code, data], 0x2000), Some(Error::BadEntry(0x2000)));
        let past_file = load(RX, 0x2f0, 0x1000, 0x20, 0x20);
        assert_eq!(check(&[past_file], 0x1000), Some(Error::Truncated));
        let more_in_file = load(RX, 0, 0x1000, 0x20, 0x10);
        assert_eq!(check(&[more_in_file], 0x1000), Some(Error::BadSegment(0)));
        let empty = load(R, 0, 0x2000, 0, 0);
        assert_eq!(check(&[code, empty], 0x1000), Some(Error::BadSegment(1)));
        let past_end = load(R, 0, u64::MAX - 0x100, 0x10, 0x10);
        assert_eq!(check(&[code, past_end], 0x1000), Some(Error::BadSegment(1)));
        let writable_code = load(RX | WRITE, 0, 0x1000, 0x10, 0x10);
        assert_eq!(
            check(&[writable_code], 0x1000),
            Some(Error::WritableAndExecutable(0))
        );
        // Another segment on the code's page, and one before it.
        let same_page = load(RW, 0, 0x1800, 0x10, 0x10);
        assert_eq!(check(&[code, same_page], 0x1000), Some(Error::Overlap(1)));
        let before = load(RW, 0, 0, 0x10, 0x10);
        assert_eq!(check(&[code, before], 0x1000), Some(Error::Overlap(1)));
        for (kind, name) in [
            (DYNAMIC, "dynamic segment"),
            (INTERP, "interpreter"),
            (TLS, "thread-local storage"),
        ] {
            let other = Header { kind, ..data };
            assert_eq!(
                check(&[code, other], 0x1000),
                Some(Error::Unsupported(name))
            );
        }
    }
}
