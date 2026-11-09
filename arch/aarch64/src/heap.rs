//! The kernel heap: talc, as the global allocator. It has no memory until
//! page tables are ported, so every allocation fails until then.

use talc::TalcLock;
use talc::source::Manual;
use thisos_hal::HeapStats;

#[global_allocator]
static TALC: TalcLock<spin::Mutex<()>, Manual> = TalcLock::new(Manual);

/// Talc's counters. Takes talc's lock, so trap handlers never call it.
pub fn stats() -> HeapStats {
    let talc = TALC.lock();
    let counters = talc.counters();
    HeapStats {
        allocated: counters.allocated_bytes,
        claimed: counters.claimed_bytes,
    }
}
