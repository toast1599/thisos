//! The test runner for the kernel's `#[test_case]` functions. It ends on a
//! summary of the results with a large PASS or FAIL, which is all the
//! laptop shows, then exits QEMU.

use core::fmt::{self, Write};
use core::hint::spin_loop;

use spin::Mutex;
use thisos_hal::{Cpu, Platform, Probe, Trap, TrapFrame};

use crate::{Arch, logger, panic, timer};

/// The tests and the index of the one running, or of none once all passed,
/// for the panic handler's summary. `None` until the tests start.
static RUN: Mutex<Option<Run>> = Mutex::new(None);

#[derive(Clone, Copy)]
struct Run {
    tests: &'static [&'static dyn Test],
    current: usize,
    /// The test that expects a panic, which runs after all the others.
    last: Option<usize>,
    /// The tick the running test started at.
    started: u64,
}

/// How long a test may run, in seconds, before the watchdog stops it.
const LIMIT: u64 = 10;

/// What ends the probe the running test runs, set while it runs.
static EXPECTED: Mutex<Option<Expectation>> = Mutex::new(None);

#[derive(Clone, Copy)]
enum Expectation {
    /// A trap the function accepts.
    Trap(fn(Trap) -> bool),
    /// The first timer interrupt at or after this tick.
    Timeout(u64),
}

/// A `#[test_case]` function, or an [`ExpectPanic`].
pub trait Test: Sync {
    /// The function's path.
    fn name(&self) -> &'static str;
    /// Calls the function.
    fn run(&self);
    /// What the message of the panic the test passes by contains, for a
    /// test that passes by panicking.
    fn expected_panic(&self) -> Option<&'static str> {
        None
    }
}

/// A `#[test_case]` const for a test that passes by panicking with a
/// message that contains `message`. A panic ends the run, so the runner
/// runs it after every other test, and allows only one.
pub struct ExpectPanic {
    /// The test's path, as a function's would read.
    pub name: &'static str,
    /// The test.
    pub run: fn(),
    /// Part of the panic message it expects.
    pub message: &'static str,
}

impl Test for ExpectPanic {
    fn name(&self) -> &'static str {
        self.name
    }

    fn run(&self) {
        (self.run)();
    }

    fn expected_panic(&self) -> Option<&'static str> {
        Some(self.message)
    }
}

impl<T: Fn() + Sync> Test for T {
    fn name(&self) -> &'static str {
        core::any::type_name::<T>()
    }

    fn run(&self) {
        self();
    }
}

/// Runs the tests in order, the one that expects a panic last, then reports
/// success. A failing test panics, and the panic handler calls
/// [`panicked`].
pub fn run(tests: &'static [&'static dyn Test]) -> ! {
    let mut panics = (0..tests.len()).filter(|&i| {
        tests
            .get(i)
            .is_some_and(|test| test.expected_panic().is_some())
    });
    let last = panics.next();
    assert!(
        panics.next().is_none(),
        "at most one test may expect a panic, since a panic ends the run"
    );
    let order = (0..tests.len()).filter(|&i| Some(i) != last).chain(last);
    for (current, test) in order.filter_map(|i| Some((i, *tests.get(i)?))) {
        *RUN.lock() = Some(Run {
            tests,
            current,
            last,
            started: timer::ticks(),
        });
        log::info!("test {}", test.name());
        test.run();
        assert!(
            Some(current) != last,
            "the test returned without the panic it expects"
        );
    }
    // Before the log line, so that a panic in it doesn't blame the last test.
    *RUN.lock() = Some(Run {
        tests,
        current: tests.len(),
        last,
        started: timer::ticks(),
    });
    log::info!("{} tests passed", tests.len());
    finish(true)
}

/// Names the failed test after its panic report, then reports failure.
pub fn fail() -> ! {
    finish(false)
}

/// Ends the run after a panic's report: with success when the running test
/// expects a panic whose message contains what it names, which also makes
/// it the last, and with failure otherwise.
pub fn panicked(message: &dyn fmt::Display) -> ! {
    let passed = RUN.try_lock().is_some_and(|mut run| {
        let Some(run) = run.as_mut() else {
            return false;
        };
        let expected = run
            .tests
            .get(run.current)
            .and_then(|test| test.expected_panic());
        let passed = expected.is_some_and(|expected| contains(message, expected));
        if passed {
            run.current = run.tests.len();
        }
        passed
    });
    finish(passed)
}

/// Whether `message`, as far as its first 256 bytes, contains `part`.
/// Without the heap, which a panic may have left locked.
fn contains(message: &dyn fmt::Display, part: &str) -> bool {
    struct Prefix {
        bytes: [u8; 256],
        len: usize,
    }
    impl Write for Prefix {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for &byte in s.as_bytes() {
                if let Some(slot) = self.bytes.get_mut(self.len) {
                    *slot = byte;
                    self.len += 1;
                }
            }
            Ok(())
        }
    }
    let mut prefix = Prefix {
        bytes: [0; 256],
        len: 0,
    };
    let _ = write!(prefix, "{message}");
    let written = prefix.bytes.get(..prefix.len).unwrap_or_default();
    part.is_empty()
        || written
            .windows(part.len())
            .any(|window| window == part.as_bytes())
}

/// Far more spins than 100 ms takes under TCG, and well under the
/// watchdog's 10 s on the test laptop, where a spin costs 83 to 123 TSC
/// cycles at 2995 MHz: 3.7 to 5.5 s.
const MAX_SPINS: u64 = 1 << 27;

/// Spins until `done` holds, and panics after [`MAX_SPINS`].
pub fn spin_until(what: &str, done: impl Fn() -> bool) {
    for _ in 0..MAX_SPINS {
        if done() {
            return;
        }
        spin_loop();
    }
    panic!("{what} didn't happen within {MAX_SPINS} spins");
}

/// Runs `probe`, and returns once it traps with a trap `expected` accepts.
/// Any other trap fails the test as usual, and a probe that never traps
/// runs into the watchdog, except one that runs with interrupts off.
pub fn expect_trap(expected: fn(Trap) -> bool, probe: Probe) {
    catch(Expectation::Trap(expected), probe);
}

/// Runs `probe`, and returns once it has run for `seconds`, as the watchdog
/// would stop a hung test.
pub fn expect_timeout(seconds: u64, probe: Probe) {
    let at = timer::ticks() + seconds * u64::from(timer::HZ);
    catch(Expectation::Timeout(at), probe);
}

fn catch(expectation: Expectation, probe: Probe) {
    *EXPECTED.lock() = Some(expectation);
    Arch::catch(probe);
    *EXPECTED.lock() = None;
}

/// Abandons the running probe when `trap` is what its test expects, and
/// returns whether it did. The kernel's trap handler calls this first.
/// Through `try_lock`, because a timer interrupt can come while `catch`
/// holds the lock; no probe runs then.
pub(crate) fn caught(trap: Trap, frame: &mut <Arch as Platform>::TrapFrame) -> bool {
    let Some(expected) = EXPECTED.try_lock().and_then(|expected| *expected) else {
        return false;
    };
    let matches = match expected {
        Expectation::Trap(accepts) => accepts(trap),
        Expectation::Timeout(at) => trap == Trap::Timer && timer::ticks() >= at,
    };
    matches && frame.abandon()
}

/// Stops the running test with a report of where it was, once it has run
/// past the watchdog's limit. The kernel's trap handler calls this on every
/// timer interrupt.
pub(crate) fn watchdog(frame: &<Arch as Platform>::TrapFrame) {
    if let Some(test) = overdue() {
        panic::trapped(
            format_args!("test {} timed out after {LIMIT} s", test.name()),
            frame,
        );
    }
}

/// The running test, if it has run past the watchdog's limit. Through
/// `try_lock`, because it runs in the timer's handler, which may interrupt
/// `run` holding the lock; that tick then doesn't check.
fn overdue() -> Option<&'static dyn Test> {
    let run = (*RUN.try_lock()?)?;
    let test = *run.tests.get(run.current)?;
    let limit = LIMIT * u64::from(timer::HZ);
    (timer::ticks() - run.started >= limit).then_some(test)
}

fn finish(pass: bool) -> ! {
    if let Some(run) = RUN.try_lock().map(|run| *run) {
        if let Some(test) = run
            .and_then(|run| run.tests.get(run.current))
            .filter(|_| !pass)
        {
            // The logger may be the thing that panicked, so this goes
            // through the panic path.
            logger::panic(format_args!("test {} failed", test.name()), &"");
        }
        logger::test_summary(&Summary { run, pass }, pass);
    }
    Arch::exit_qemu(pass)
}

struct Summary {
    run: Option<Run>,
    pass: bool,
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "summary:")?;
        let Some(run) = self.run else {
            return writeln!(f, "  panic before the tests started");
        };
        for (i, test) in run.tests.iter().enumerate() {
            writeln!(f, "  {} {}", run.result(i), test.name())?;
        }
        if !self.pass && run.current == run.tests.len() {
            writeln!(f, "  panic after the last test")?;
        }
        Ok(())
    }
}

impl Run {
    /// What the summary says of test `i`. The tests ran in order, except the
    /// one that expects a panic, which ran last.
    fn result(&self, i: usize) -> &'static str {
        if self.current == self.tests.len() {
            "ok"
        } else if i == self.current {
            "FAILED"
        } else if Some(self.current) == self.last || (i < self.current && Some(i) != self.last) {
            "ok"
        } else {
            "not run"
        }
    }
}

#[cfg(test)]
mod tests {
    use thisos_hal::Probe;

    // The watchdog's own report ends the run, so this stands in a 1 s
    // timeout for its 10 s one and checks the timer stops the probe.
    #[test_case]
    fn the_watchdog_stops_a_hung_test() {
        super::expect_timeout(1, Probe::Spin);
    }
}
