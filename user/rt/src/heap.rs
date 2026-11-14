//! The program's heap: talc, as the global allocator, over memory objects
//! that it maps one after another from [`BASE`] whenever talc runs out.
//! Each is at least [`STEP`] bytes, or the request rounded up to pages.
//! Nothing else maps there, and only talc reads or writes the pages.

use core::alloc::Layout;
use core::ptr::NonNull;

use talc::TalcLock;
use talc::base::Talc;
use talc::base::binning::Binning;
use talc::source::Source;
use thisos_abi::{Rights, Syscall};

/// Where the heap starts: far above the program, which links at
/// `0x20_0000`, and far below the stack at the end of user space.
const BASE: u64 = 0x4000_0000;

/// The least a growth maps.
const STEP: usize = 64 << 10;

#[global_allocator]
static TALC: TalcLock<spin::Mutex<()>, Objects> = TalcLock::new(Objects {
    mapped: 0,
    end: None,
});

/// The heap's pages, mapped from [`BASE`] up to `mapped` bytes above it,
/// and where talc's heap over them ends.
#[derive(Debug)]
struct Objects {
    mapped: usize,
    /// The address talc's heap ends at, or `None` before its first claim.
    /// It lies at most a chunk below `BASE + mapped`, for alignment.
    end: Option<usize>,
}

// SAFETY: acquire calls no allocator, and claims and extends only over
// pages it has just mapped, writable, right after the ones before, which
// only talc uses from then on.
unsafe impl Source for Objects {
    fn acquire<B: Binning>(talc: &mut Talc<Self, B>, layout: Layout) -> Result<(), ()> {
        let len = layout
            .size()
            .checked_next_multiple_of(4096)
            .ok_or(())?
            .max(STEP);
        let mapped = talc.source.mapped;
        let addr = u64::try_from(mapped)
            .ok()
            .and_then(|mapped| BASE.checked_add(mapped))
            .ok_or(())?;
        map_new(addr, len)?;
        let mapped = mapped + len;
        talc.source.mapped = mapped;
        let base = base();
        talc.source.end = match talc.source.end {
            // A failed claim, too small for talc's metadata, takes the
            // pages so far again on the next acquire.
            // SAFETY: Every page from base to mapped is mapped writable,
            // and only talc reads or writes them from here on.
            None => unsafe { talc.claim(base, mapped) }.map(|end| end.addr().get()),
            Some(end) => {
                let Some(end) = NonNull::new(base.with_addr(end)) else {
                    unreachable!("talc's heap ends above BASE");
                };
                // SAFETY: As for claim; talc's heap ends at end, and the
                // pages between it and base + mapped are mapped and talc's
                // alone.
                let end = unsafe { talc.extend(end, base.wrapping_add(mapped)) };
                Some(end.addr().get())
            }
        };
        Ok(())
    }
}

/// Maps a new memory object of `len` bytes, readable and writable, at
/// `addr`, and closes its handle: the mapping holds the object.
fn map_new(addr: u64, len: usize) -> Result<(), ()> {
    let len = u64::try_from(len).map_err(|_| ())?;
    let handle = crate::syscall(Syscall::MemoryCreate, [len, 0, 0, 0, 0]).map_err(|_| ())?;
    let rights = u64::from(Rights::READ.union(Rights::WRITE).bits());
    let mapped = crate::map_syscall([handle, 0, len, addr, rights]);
    if let Err(e) = crate::syscall(Syscall::Close, [handle, 0, 0, 0, 0]) {
        panic!("closing the heap's memory object failed: {e:?}");
    }
    mapped.map(|_| ()).map_err(|_| ())
}

/// The heap's base, the one place the crate turns an integer into a
/// pointer: memory it maps itself through the map syscall, which no Rust
/// pointer reaches.
#[expect(
    clippy::disallowed_methods,
    reason = "the heap's pages come from the map syscall, never from a Rust pointer"
)]
fn base() -> *mut u8 {
    let Ok(base) = usize::try_from(BASE) else {
        unreachable!("x86_64 addresses are 64 bits");
    };
    core::ptr::with_exposed_provenance_mut(base)
}
