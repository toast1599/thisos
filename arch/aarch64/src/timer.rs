//! The tick: the virtual timer, CNTV, firing when the virtual count
//! reaches its compare value, which each tick moves one period on.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use aarch64_cpu::asm::barrier;
use aarch64_cpu::registers::{
    CNTFRQ_EL0, CNTV_CTL_EL0, CNTV_CVAL_EL0, CNTVCT_EL0, Readable, Writeable,
};
use thisos_hal::TimerFrequency;

/// Counts between ticks, or 0 until `start`.
static PERIOD: AtomicU64 = AtomicU64::new(0);

/// The timer's interrupt ID, or none until `start`.
static INTID: AtomicU32 = AtomicU32::new(u32::MAX);

/// Starts ticking `hz` times a second, with the timer's interrupt at
/// `intid`, and returns how fast the count runs. Panics if `CNTFRQ_EL0`
/// gives no frequency, or one below `hz`.
pub fn start(hz: u32, intid: u32) -> TimerFrequency {
    let frequency = CNTFRQ_EL0.get();
    assert!(frequency != 0, "CNTFRQ_EL0 gives no timer frequency");
    let period = frequency / u64::from(hz);
    assert!(
        period != 0,
        "the timer counts at {frequency} Hz, too slow for {hz} Hz"
    );
    PERIOD.store(period, Ordering::Relaxed);
    INTID.store(intid, Ordering::Relaxed);
    CNTV_CVAL_EL0.set(count() + period);
    CNTV_CTL_EL0.write(CNTV_CTL_EL0::ENABLE::SET + CNTV_CTL_EL0::IMASK::CLEAR);
    barrier::isb(barrier::SY);
    TimerFrequency {
        hz: frequency,
        source: "CNTFRQ_EL0",
    }
}

/// Whether `intid` is the timer's interrupt.
pub fn is_timer(intid: u32) -> bool {
    intid == INTID.load(Ordering::Relaxed)
}

/// Moves the compare value one period on, or past now to the next period
/// boundary if interrupts were off for longer than a period, so that the
/// ticks missed meanwhile count as one. Call it before the interrupt ends:
/// the timer holds its interrupt up until the compare value is ahead of
/// the count.
pub fn rearm() {
    let period = PERIOD.load(Ordering::Relaxed);
    let compare = CNTV_CVAL_EL0.get();
    let behind = count().saturating_sub(compare) / period;
    CNTV_CVAL_EL0.set(compare + period * (behind + 1));
    barrier::isb(barrier::SY);
}

/// The virtual count now.
fn count() -> u64 {
    // Without it, the read may happen earlier than the code before it.
    barrier::isb(barrier::SY);
    CNTVCT_EL0.get()
}
