//! A reader for flattened device trees, the blob that firmware describes a
//! machine's devices in on aarch64. [`Fdt::new`] checks the whole structure
//! block once, so walking it afterwards can't fail.

#![no_std]
#![forbid(unsafe_code)]

/// The size of the header, which [`total_size`] reads the blob's size from.
pub const HEADER_SIZE: usize = 40;

/// How deep nodes may nest, counting the root.
pub const MAX_DEPTH: usize = 16;

const MAGIC: u32 = 0xd00d_feed;
/// Version 17 added `size_dt_struct`, which `new` relies on.
const VERSION: u32 = 17;

// Structure block tokens.
const BEGIN_NODE: u32 = 1;
const END_NODE: u32 = 2;
const PROP: u32 = 3;
const NOP: u32 = 4;
const END: u32 = 9;

/// What is wrong with a blob.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The blob is shorter than its header, or than the size the header
    /// gives.
    Truncated,
    /// The header doesn't start with the magic number.
    BadMagic(u32),
    /// The blob's version, older than 17, or with a layout 17 can't read.
    Unsupported(u32),
    /// The structure or strings block doesn't lie inside the blob.
    OutOfBounds,
    /// The structure block is malformed at this offset into it.
    Malformed(usize),
    /// Nodes nest deeper than [`MAX_DEPTH`].
    TooDeep,
}

/// The big-endian word at `at` in `bytes`.
fn be32(bytes: &[u8], at: usize) -> Option<u32> {
    let word = bytes.get(at..at.checked_add(4)?)?;
    Some(u32::from_be_bytes([word[0], word[1], word[2], word[3]]))
}

fn header_field(blob: &[u8], at: usize) -> Result<usize, Error> {
    let field = be32(blob, at).ok_or(Error::Truncated)?;
    usize::try_from(field).map_err(|_| Error::OutOfBounds)
}

/// The blob's size, from the first [`HEADER_SIZE`] bytes of it, for a blob
/// that arrives as a pointer.
pub fn total_size(header: &[u8]) -> Result<usize, Error> {
    if header.len() < HEADER_SIZE {
        return Err(Error::Truncated);
    }
    let magic = be32(header, 0).ok_or(Error::Truncated)?;
    if magic != MAGIC {
        return Err(Error::BadMagic(magic));
    }
    header_field(header, 4)
}

/// A checked device tree blob.
#[derive(Clone, Copy)]
pub struct Fdt<'a> {
    structs: &'a [u8],
    strings: &'a [u8],
}

impl<'a> Fdt<'a> {
    /// Checks `blob`'s header and its whole structure block.
    pub fn new(blob: &'a [u8]) -> Result<Self, Error> {
        let size = total_size(blob)?;
        let blob = blob.get(..size).ok_or(Error::Truncated)?;
        let version = be32(blob, 20).ok_or(Error::Truncated)?;
        let last_compatible = be32(blob, 24).ok_or(Error::Truncated)?;
        if version < VERSION || last_compatible > VERSION {
            return Err(Error::Unsupported(version));
        }
        let block = |offset, size| {
            let start = header_field(blob, offset)?;
            let end = start
                .checked_add(header_field(blob, size)?)
                .ok_or(Error::OutOfBounds)?;
            blob.get(start..end).ok_or(Error::OutOfBounds)
        };
        let fdt = Fdt {
            structs: block(8, 36)?,
            strings: block(12, 32)?,
        };
        fdt.check()?;
        Ok(fdt)
    }

    /// Walks the structure block once: one root, nodes nested no deeper
    /// than [`MAX_DEPTH`], properties only before a node's children, and
    /// every token, name and value inside the blob.
    fn check(&self) -> Result<(), Error> {
        let mut cursor = self.cursor();
        let mut depth = 0;
        let mut roots = 0;
        let mut props_allowed = false;
        loop {
            let at = cursor.at;
            match cursor.next()? {
                Token::Begin(_) => {
                    if depth == 0 {
                        roots += 1;
                    }
                    depth += 1;
                    if roots > 1 {
                        return Err(Error::Malformed(at));
                    }
                    if depth > MAX_DEPTH {
                        return Err(Error::TooDeep);
                    }
                    props_allowed = true;
                }
                Token::End => {
                    if depth == 0 {
                        return Err(Error::Malformed(at));
                    }
                    depth -= 1;
                    props_allowed = false;
                }
                Token::Prop { .. } if !props_allowed => return Err(Error::Malformed(at)),
                Token::Prop { .. } | Token::Nop => {}
                Token::Finish if depth == 0 && roots == 1 => return Ok(()),
                Token::Finish => return Err(Error::Malformed(at)),
            }
        }
    }

    fn cursor(&self) -> Cursor<'a> {
        Cursor {
            structs: self.structs,
            strings: self.strings,
            at: 0,
        }
    }

    /// Every node, depth first, starting at the root.
    pub fn nodes(&self) -> Nodes<'a> {
        Nodes {
            cursor: self.cursor(),
            depth: 0,
            cells: [DEFAULT_CELLS; MAX_DEPTH + 1],
        }
    }

    /// The root node.
    pub fn root(&self) -> Option<Node<'a>> {
        self.nodes().next()
    }
}

enum Token<'a> {
    Begin(&'a str),
    End,
    Prop { name: &'a str, value: &'a [u8] },
    Nop,
    Finish,
}

/// A place in the structure block.
#[derive(Clone)]
struct Cursor<'a> {
    structs: &'a [u8],
    strings: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    /// Reads the token at the cursor and moves past it and its padding.
    fn next(&mut self) -> Result<Token<'a>, Error> {
        let start = self.at;
        let bad = Error::Malformed(start);
        let token = be32(self.structs, start).ok_or(bad)?;
        let body = start + 4;
        let (token, end) = match token {
            BEGIN_NODE => {
                let name = c_str(self.structs, body).ok_or(bad)?;
                (Token::Begin(name), body + name.len() + 1)
            }
            END_NODE => (Token::End, body),
            PROP => {
                let len = be32(self.structs, body).ok_or(bad)?;
                let name_offset = be32(self.structs, body + 4).ok_or(bad)?;
                let len = usize::try_from(len).map_err(|_| bad)?;
                let name_offset = usize::try_from(name_offset).map_err(|_| bad)?;
                let value_start = body + 8;
                let value_end = value_start.checked_add(len).ok_or(bad)?;
                let value = self.structs.get(value_start..value_end).ok_or(bad)?;
                let name = c_str(self.strings, name_offset).ok_or(bad)?;
                (Token::Prop { name, value }, value_end)
            }
            NOP => (Token::Nop, body),
            END => (Token::Finish, body),
            _ => return Err(bad),
        };
        self.at = end.next_multiple_of(4);
        Ok(token)
    }
}

/// The NUL-terminated UTF-8 string at `at` in `bytes`.
fn c_str(bytes: &[u8], at: usize) -> Option<&str> {
    let rest = bytes.get(at..)?;
    let len = rest.iter().position(|&b| b == 0)?;
    core::str::from_utf8(rest.get(..len)?).ok()
}

/// How many 32-bit cells an address and a size take in a node's `reg`, as
/// its parent's `#address-cells` and `#size-cells` give them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cells {
    address: u32,
    size: u32,
}

/// What a node without `#address-cells` or `#size-cells` gives its
/// children, as the specification says.
const DEFAULT_CELLS: Cells = Cells {
    address: 2,
    size: 1,
};

/// The nodes of an [`Fdt`], depth first.
pub struct Nodes<'a> {
    cursor: Cursor<'a>,
    /// The depth of the next node to begin.
    depth: usize,
    /// The cells for a node's `reg`, by its depth.
    cells: [Cells; MAX_DEPTH + 1],
}

impl<'a> Iterator for Nodes<'a> {
    type Item = Node<'a>;

    // Fdt::new checked the structure block, so the cursor never fails and
    // the depth stays within the arrays.
    fn next(&mut self) -> Option<Node<'a>> {
        loop {
            match self.cursor.next().ok()? {
                Token::Begin(name) => {
                    let depth = self.depth;
                    self.depth += 1;
                    let node = Node {
                        name,
                        depth,
                        props: self.cursor.clone(),
                        cells: *self.cells.get(depth)?,
                    };
                    *self.cells.get_mut(depth + 1)? = Cells {
                        address: node.cell("#address-cells").unwrap_or(DEFAULT_CELLS.address),
                        size: node.cell("#size-cells").unwrap_or(DEFAULT_CELLS.size),
                    };
                    return Some(node);
                }
                Token::End => self.depth = self.depth.checked_sub(1)?,
                Token::Prop { .. } | Token::Nop => {}
                Token::Finish => return None,
            }
        }
    }
}

/// A node of an [`Fdt`].
#[derive(Clone)]
pub struct Node<'a> {
    name: &'a str,
    depth: usize,
    /// The cursor at the node's first property.
    props: Cursor<'a>,
    cells: Cells,
}

impl<'a> Node<'a> {
    /// The node's name with its unit address, such as `intc@8000000`. The
    /// root's is empty.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// How many nodes enclose this one: 0 for the root.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// The node's properties, by name and value.
    pub fn properties(&self) -> impl Iterator<Item = (&'a str, &'a [u8])> {
        let mut cursor = self.props.clone();
        core::iter::from_fn(move || {
            loop {
                match cursor.next().ok()? {
                    Token::Prop { name, value } => return Some((name, value)),
                    Token::Nop => {}
                    Token::Begin(_) | Token::End | Token::Finish => return None,
                }
            }
        })
    }

    /// The value of the property `name`.
    pub fn property(&self, name: &str) -> Option<&'a [u8]> {
        self.properties()
            .find(|&(prop, _)| prop == name)
            .map(|(_, value)| value)
    }

    /// The property `name` as one 32-bit cell.
    fn cell(&self, name: &str) -> Option<u32> {
        let value = self.property(name)?;
        if value.len() != 4 {
            return None;
        }
        be32(value, 0)
    }

    /// The strings in the node's `compatible`, most specific first.
    pub fn compatible(&self) -> impl Iterator<Item = &'a str> {
        self.property("compatible")
            .unwrap_or(&[])
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .filter_map(|s| core::str::from_utf8(s).ok())
    }

    /// The address ranges in the node's `reg`, read with its parent's
    /// `#address-cells` and `#size-cells`. `None` without a `reg`, with cell
    /// counts that don't fit 64 bits, or with a length that isn't a whole
    /// number of ranges.
    pub fn reg(&self) -> Option<Reg<'a>> {
        let value = self.property("reg")?;
        let address = usize::try_from(self.cells.address).ok()?;
        let size = usize::try_from(self.cells.size).ok()?;
        if !(1..=2).contains(&address) || size > 2 || value.len() % (4 * (address + size)) != 0 {
            return None;
        }
        Some(Reg {
            value,
            address,
            size,
        })
    }
}

/// An address range from a `reg` property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    /// The first address.
    pub address: u64,
    /// The size in bytes.
    pub size: u64,
}

/// The ranges in a node's `reg`, as [`Node::reg`] reads them.
pub struct Reg<'a> {
    value: &'a [u8],
    /// Cells per address and per size.
    address: usize,
    size: usize,
}

impl Iterator for Reg<'_> {
    type Item = Region;

    fn next(&mut self) -> Option<Region> {
        let (address, rest) = self.value.split_at_checked(4 * self.address)?;
        let (size, rest) = rest.split_at_checked(4 * self.size)?;
        self.value = rest;
        Some(Region {
            address: number(address)?,
            size: number(size)?,
        })
    }
}

/// The big-endian number in `cells`, up to two of them.
fn number(cells: &[u8]) -> Option<u64> {
    (0..cells.len())
        .step_by(4)
        .try_fold(0, |n, at| Some((n << 32) | u64::from(be32(cells, at)?)))
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec::Vec;

    use super::*;

    /// Writes a blob token by token.
    #[derive(Default)]
    struct Builder {
        structs: Vec<u8>,
        strings: Vec<u8>,
    }

    impl Builder {
        fn word(&mut self, word: u32) -> &mut Self {
            self.structs.extend(word.to_be_bytes());
            self
        }

        fn pad(&mut self) {
            while self.structs.len() % 4 != 0 {
                self.structs.push(0);
            }
        }

        fn begin(&mut self, name: &str) -> &mut Self {
            self.word(BEGIN_NODE);
            self.structs.extend(name.as_bytes());
            self.structs.push(0);
            self.pad();
            self
        }

        fn end(&mut self) -> &mut Self {
            self.word(END_NODE)
        }

        fn prop(&mut self, name: &str, value: &[u8]) -> &mut Self {
            let offset = u32::try_from(self.strings.len()).unwrap();
            self.strings.extend(name.as_bytes());
            self.strings.push(0);
            self.word(PROP)
                .word(u32::try_from(value.len()).unwrap())
                .word(offset);
            self.structs.extend(value);
            self.pad();
            self
        }

        fn cells(&mut self, name: &str, cells: &[u32]) -> &mut Self {
            let value: Vec<u8> = cells.iter().flat_map(|c| c.to_be_bytes()).collect();
            self.prop(name, &value)
        }

        /// The blob, with the END token added, an empty memory
        /// reservation block, and `version` in the header.
        fn blob_version(&mut self, version: u32) -> Vec<u8> {
            self.word(END);
            let rsvmap = HEADER_SIZE;
            let structs = rsvmap + 16;
            let strings = structs + self.structs.len();
            let total = strings + self.strings.len();
            let header = [
                MAGIC,
                u32::try_from(total).unwrap(),
                u32::try_from(structs).unwrap(),
                u32::try_from(strings).unwrap(),
                u32::try_from(rsvmap).unwrap(),
                version,
                16,
                0,
                u32::try_from(self.strings.len()).unwrap(),
                u32::try_from(self.structs.len()).unwrap(),
            ];
            let mut blob: Vec<u8> = header.iter().flat_map(|w| w.to_be_bytes()).collect();
            blob.extend([0; 16]);
            blob.extend(&self.structs);
            blob.extend(&self.strings);
            blob
        }

        fn blob(&mut self) -> Vec<u8> {
            self.blob_version(VERSION)
        }
    }

    /// A tree shaped like part of QEMU's virt machine.
    fn virt() -> Vec<u8> {
        Builder::default()
            .begin("")
            .prop("compatible", b"linux,dummy-virt\0")
            .cells("#address-cells", &[2])
            .cells("#size-cells", &[2])
            .begin("cpus")
            .cells("#address-cells", &[1])
            .cells("#size-cells", &[0])
            .begin("cpu@0")
            .cells("reg", &[0])
            .end()
            .end()
            .begin("intc@8000000")
            .prop("compatible", b"arm,gic-v3\0")
            .cells(
                "reg",
                &[0, 0x0800_0000, 0, 0x1_0000, 0, 0x080a_0000, 0, 0xf6_0000],
            )
            .end()
            .begin("timer")
            .prop("compatible", b"arm,armv8-timer\0arm,armv7-timer\0")
            .cells("interrupts", &[1, 13, 4, 1, 14, 4, 1, 11, 4, 1, 10, 4])
            .end()
            .end()
            .blob()
    }

    fn node<'a>(fdt: &Fdt<'a>, name: &str) -> Node<'a> {
        fdt.nodes().find(|n| n.name() == name).unwrap()
    }

    #[test]
    fn walks_every_node_depth_first() {
        let blob = virt();
        let fdt = Fdt::new(&blob).unwrap();
        let nodes: Vec<_> = fdt.nodes().map(|n| (n.name(), n.depth())).collect();
        assert_eq!(
            nodes,
            [
                ("", 0),
                ("cpus", 1),
                ("cpu@0", 2),
                ("intc@8000000", 1),
                ("timer", 1)
            ]
        );
    }

    #[test]
    fn reads_properties_and_compatible_strings() {
        let blob = virt();
        let fdt = Fdt::new(&blob).unwrap();
        let root = fdt.root().unwrap();
        assert!(root.compatible().eq(["linux,dummy-virt"]));
        let timer = node(&fdt, "timer");
        assert!(
            timer
                .compatible()
                .eq(["arm,armv8-timer", "arm,armv7-timer"])
        );
        assert_eq!(timer.property("interrupts").map(<[u8]>::len), Some(48));
        assert_eq!(timer.property("reg"), None);
        let names: Vec<_> = timer.properties().map(|(name, _)| name).collect();
        assert_eq!(names, ["compatible", "interrupts"]);
    }

    #[test]
    fn reg_uses_the_parents_cells() {
        let blob = virt();
        let fdt = Fdt::new(&blob).unwrap();
        let gic: Vec<_> = node(&fdt, "intc@8000000").reg().unwrap().collect();
        assert_eq!(
            gic,
            [
                Region {
                    address: 0x0800_0000,
                    size: 0x1_0000
                },
                Region {
                    address: 0x080a_0000,
                    size: 0xf6_0000
                },
            ]
        );
        let cpu: Vec<_> = node(&fdt, "cpu@0").reg().unwrap().collect();
        assert_eq!(
            cpu,
            [Region {
                address: 0,
                size: 0
            }]
        );
    }

    #[test]
    fn reg_rejects_partial_ranges_and_wide_cells() {
        let blob = Builder::default()
            .begin("")
            .cells("#address-cells", &[2])
            .cells("#size-cells", &[2])
            .begin("short")
            .cells("reg", &[0, 1, 0])
            .end()
            .begin("bus")
            .cells("#address-cells", &[3])
            .begin("wide")
            .cells("reg", &[0, 0, 1, 0])
            .end()
            .end()
            .end()
            .blob();
        let fdt = Fdt::new(&blob).unwrap();
        assert!(node(&fdt, "short").reg().is_none());
        assert!(node(&fdt, "wide").reg().is_none());
    }

    #[test]
    fn rejects_bad_headers() {
        let blob = virt();
        assert_eq!(
            Fdt::new(&blob[..HEADER_SIZE - 1]).err(),
            Some(Error::Truncated)
        );
        assert_eq!(
            Fdt::new(&blob[..blob.len() - 1]).err(),
            Some(Error::Truncated)
        );
        let mut bad = blob.clone();
        bad[0] = 0;
        assert_eq!(Fdt::new(&bad).err(), Some(Error::BadMagic(0x000d_feed)));
        let old = Builder::default().begin("").end().blob_version(16);
        assert_eq!(Fdt::new(&old).err(), Some(Error::Unsupported(16)));
        let mut outside = blob.clone();
        // size_dt_struct past the end of the blob.
        outside[36..40].copy_from_slice(&0x10_0000u32.to_be_bytes());
        assert_eq!(Fdt::new(&outside).err(), Some(Error::OutOfBounds));
        assert_eq!(total_size(&blob), Ok(blob.len()));
    }

    #[test]
    fn rejects_malformed_structure() {
        let unclosed = Builder::default().begin("").blob();
        assert!(matches!(Fdt::new(&unclosed), Err(Error::Malformed(_))));
        let two_roots = Builder::default().begin("").end().begin("").end().blob();
        assert!(matches!(Fdt::new(&two_roots), Err(Error::Malformed(_))));
        let extra_end = Builder::default().begin("").end().end().blob();
        assert!(matches!(Fdt::new(&extra_end), Err(Error::Malformed(_))));
        let late_prop = Builder::default()
            .begin("")
            .begin("child")
            .end()
            .prop("late", &[])
            .end()
            .blob();
        assert!(matches!(Fdt::new(&late_prop), Err(Error::Malformed(_))));
        let mut bad_name = Builder::default();
        bad_name.begin("").word(PROP).word(0).word(0x1000).end();
        assert!(matches!(
            Fdt::new(&bad_name.blob()),
            Err(Error::Malformed(_))
        ));
        let mut deep = Builder::default();
        for _ in 0..=MAX_DEPTH {
            deep.begin("n");
        }
        for _ in 0..=MAX_DEPTH {
            deep.end();
        }
        assert_eq!(Fdt::new(&deep.blob()).err(), Some(Error::TooDeep));
        let mut deepest = Builder::default();
        for _ in 0..MAX_DEPTH {
            deepest.begin("n");
        }
        for _ in 0..MAX_DEPTH {
            deepest.end();
        }
        let deepest = deepest.blob();
        let fdt = Fdt::new(&deepest).unwrap();
        assert_eq!(fdt.nodes().last().map(|n| n.depth()), Some(MAX_DEPTH - 1));
    }
}
