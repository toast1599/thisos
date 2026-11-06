//! The local APIC in x2APIC mode, and its timer. x2APIC reaches the
//! registers through MSRs, so it needs no MMIO mapping, and it works
//! whether the firmware left the APIC in xAPIC or in x2APIC mode.

use core::hint::spin_loop;
use thisos_hal::{TimerFrequencies, TimerFrequency};

use x86_64::instructions::interrupts;
use x86_64::registers::model_specific::Msr;

use crate::once::Once;
use crate::{pit, tsc};

/// The timer's vector, the first after the legacy PICs' 0x20-0x2f.
pub const TIMER_VECTOR: u64 = 0x30;

/// The spurious interrupt's vector, whose low four bits older APICs fix
/// at 1.
const SPURIOUS_VECTOR: u64 = 0xff;

const IA32_APIC_BASE: u32 = 0x1b;
const ENABLED: u64 = 1 << 11;
const X2APIC_MODE: u64 = 1 << 10;

// Each x2APIC register is MSR 0x800 plus its xAPIC offset divided by 16.
const TASK_PRIORITY: u32 = 0x808;
const EOI: u32 = 0x80b;
const SPURIOUS: u32 = 0x80f;
const LVT_TIMER: u32 = 0x832;
const INITIAL_COUNT: u32 = 0x838;
const CURRENT_COUNT: u32 = 0x839;
const DIVIDE: u32 = 0x83e;

const SOFTWARE_ENABLE: u64 = 1 << 8;
const MASKED: u64 = 1 << 16;
const PERIODIC: u64 = 1 << 17;
const DIVIDE_BY_1: u64 = 0b1011;

/// How long a measurement against the PIT counts: 50 ms.
const PIT_TICKS: u16 = 59_659;

/// The timer's count when a measurement starts, the most it holds.
const MEASURE_FROM: u64 = 0xffff_ffff;

/// The local APIC once it is in x2APIC mode. Only `X2Apic::enable` makes
/// one, so every register its methods use exists.
#[derive(Clone, Copy)]
struct X2Apic(());

impl X2Apic {
    /// Puts the APIC in x2APIC mode, enables it with the spurious vector
    /// and a task priority of 0, and leaves the timer masked. Panics if the
    /// CPU has no x2APIC.
    fn enable() -> Self {
        let ecx = core::arch::x86_64::__cpuid(1).ecx;
        assert!(ecx & (1 << 21) != 0, "the CPU has no x2APIC");
        let mut base = Msr::new(IA32_APIC_BASE);
        // SAFETY: CPUID reported x2APIC, so IA32_APIC_BASE exists, and
        // reading it changes nothing.
        let value = unsafe { base.read() };
        // The APIC can't go from disabled to x2APIC mode in one write.
        if value & ENABLED == 0 {
            // SAFETY: Enabling the APIC in xAPIC mode keeps its base, so it
            // changes no memory mapping, and its interrupts stay off while
            // interrupts are off, which they are until the kernel starts
            // the timer.
            unsafe { base.write(value | ENABLED) };
        }
        if value & X2APIC_MODE == 0 {
            // SAFETY: CPUID reported x2APIC, and the APIC is enabled, from
            // which x2APIC mode is a valid transition. It stops decoding
            // its MMIO page, which nothing in the kernel maps.
            unsafe { base.write(value | ENABLED | X2APIC_MODE) };
        }
        let apic = X2Apic(());
        apic.write(SPURIOUS, SOFTWARE_ENABLE | SPURIOUS_VECTOR);
        apic.write(TASK_PRIORITY, 0);
        apic.write(DIVIDE, DIVIDE_BY_1);
        apic.write(LVT_TIMER, MASKED | TIMER_VECTOR);
        apic
    }

    #[expect(
        clippy::unused_self,
        reason = "an X2Apic is the proof that the APIC is in x2APIC mode"
    )]
    fn read(self, msr: u32) -> u64 {
        // SAFETY: The APIC is in x2APIC mode, so the register MSRs this
        // module passes exist, and reading them changes nothing.
        unsafe { Msr::new(msr).read() }
    }

    #[expect(
        clippy::unused_self,
        reason = "an X2Apic is the proof that the APIC is in x2APIC mode"
    )]
    fn write(self, msr: u32, value: u64) {
        let mut msr = Msr::new(msr);
        // SAFETY: The APIC is in x2APIC mode, so the register MSRs this
        // module passes exist. Its writes configure the APIC, its timer
        // and EOI, which touch no memory.
        unsafe { msr.write(value) };
    }

    /// Runs the timer periodically from `initial`, unmasked.
    fn run(self, initial: u32) {
        self.write(LVT_TIMER, PERIODIC | TIMER_VECTOR);
        self.write(INITIAL_COUNT, u64::from(initial));
    }

    fn stop(self) {
        self.write(LVT_TIMER, MASKED | TIMER_VECTOR);
        self.write(INITIAL_COUNT, 0);
    }

    /// Stops the timer, then measures how fast it counts against the PIT,
    /// or returns `None` if the PIT doesn't count. The timer stays stopped.
    fn measure_against_pit(self) -> Option<u64> {
        self.stop();
        let ended = pit::count(PIT_TICKS, || self.write(INITIAL_COUNT, MEASURE_FROM));
        let left = self.read(CURRENT_COUNT);
        self.stop();
        // A count of 0 means the timer ran out first, so the time is unknown.
        (ended && left != 0).then(|| (MEASURE_FROM - left) * pit::HZ / u64::from(PIT_TICKS))
    }

    /// Stops the timer, then measures how fast it counts over 50 ms of a TSC
    /// that counts at `tsc_hz`. The timer stays stopped.
    fn measure_against_tsc(self, tsc_hz: u64) -> Option<u64> {
        self.stop();
        let cycles = tsc_hz / 20;
        self.write(INITIAL_COUNT, MEASURE_FROM);
        let started = tsc::read();
        while tsc::read().wrapping_sub(started) < cycles {
            spin_loop();
        }
        let left = self.read(CURRENT_COUNT);
        let elapsed = tsc::read().wrapping_sub(started);
        self.stop();
        let counted = u128::from(MEASURE_FROM - left);
        let hz = counted * u128::from(tsc_hz) / u128::from(elapsed);
        (left != 0).then(|| u64::try_from(hz).ok()).flatten()
    }
}

/// The crystal frequency CPUID leaf 0x15 reports, which the timer counts at
/// when divided by 1, or `None` when the leaf doesn't report it. The SDM
/// ties the timer to the crystal only when the leaf also gives the TSC's
/// ratio to it, in EBX.
fn reported() -> Option<u64> {
    if core::arch::x86_64::__cpuid(0).eax < 0x15 {
        return None;
    }
    let leaf = core::arch::x86_64::__cpuid(0x15);
    (leaf.ebx != 0 && leaf.ecx != 0).then(|| u64::from(leaf.ecx))
}

static APIC: Once<X2Apic> = Once::new();

/// The timer's initial count, for restarting it after a measurement.
static INITIAL: Once<u32> = Once::new();

/// Starts the timer at `hz`, from CPUID's frequency, or the PIT's where
/// CPUID gives none. Panics if neither does, or if `hz` is too slow or too
/// fast for the timer's 32-bit count.
pub fn start(hz: u32) -> TimerFrequency {
    assert!(APIC.get().is_none(), "the timer is started already");
    let apic = APIC.init(X2Apic::enable());
    let frequency = if let Some(counts) = reported() {
        TimerFrequency {
            hz: counts,
            source: "CPUID leaf 0x15",
        }
    } else if let Some(counts) = apic.measure_against_pit() {
        TimerFrequency {
            hz: counts,
            source: "PIT channel 2",
        }
    } else {
        panic!("neither CPUID leaf 0x15 nor the PIT gives the timer's frequency");
    };
    let initial = frequency.hz / u64::from(hz);
    let Some(initial) = u32::try_from(initial).ok().filter(|&count| count != 0) else {
        panic!("{hz} Hz needs a timer count of {initial}, which doesn't fit");
    };
    apic.run(*INITIAL.init(initial));
    frequency
}

/// Measures the timer with interrupts off, then restarts it. It measures
/// against the TSC where CPUID gives the TSC's frequency, since the PIT can
/// be gated there, as on the test laptop, and against the PIT otherwise.
pub fn frequencies() -> TimerFrequencies {
    let (Some(&apic), Some(&initial)) = (APIC.get(), INITIAL.get()) else {
        panic!("the timer isn't started");
    };
    let measured = interrupts::without_interrupts(|| {
        let measured = match tsc::hz() {
            Some(tsc_hz) => apic.measure_against_tsc(tsc_hz).map(|hz| TimerFrequency {
                hz,
                source: "the TSC",
            }),
            None => apic.measure_against_pit().map(|hz| TimerFrequency {
                hz,
                source: "PIT channel 2",
            }),
        };
        apic.run(initial);
        measured
    });
    TimerFrequencies {
        reported: reported(),
        measured,
    }
}

/// Tells the APIC that the timer interrupt's handler is done.
pub fn eoi() {
    if let Some(apic) = APIC.get() {
        apic.write(EOI, 0);
    }
}
