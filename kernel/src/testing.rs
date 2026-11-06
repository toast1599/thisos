//! The test runner for the kernel's `#[test_case]` functions. It ends on a
//! summary of the results with a large PASS or FAIL, which is all the
//! laptop shows, then exits QEMU.

use core::fmt;

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

/// A `#[test_case]` function.
pub trait Test: Sync {
    /// The function's path.
    fn name(&self) -> &'static str;
    /// Calls the function.
    fn run(&self);
}

impl<T: Fn() + Sync> Test for T {
    fn name(&self) -> &'static str {
        core::any::type_name::<T>()
    }

    fn run(&self) {
        self();
    }
}

/// Runs the tests in order, then reports success. A failing test panics,
/// and the panic handler calls [`fail`].
pub fn run(tests: &'static [&'static dyn Test]) -> ! {
    for (current, test) in tests.iter().enumerate() {
        *RUN.lock() = Some(Run {
            tests,
            current,
            started: timer::ticks(),
        });
        log::info!("test {}", test.name());
        test.run();
    }
    // Before the log line, so that a panic in it doesn't blame the last test.
    *RUN.lock() = Some(Run {
        tests,
        current: tests.len(),
        started: timer::ticks(),
    });
    log::info!("{} tests passed", tests.len());
    finish(true)
}

/// Names the failed test after its panic report, then reports failure.
pub fn fail() -> ! {
    finish(false)
}

/// Runs `probe`, and returns once it traps with a trap `expected` accepts.
/// Any other trap fails the test as usual, and a probe that never traps
/// runs into the watchdog.
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
            let result = match i.cmp(&run.current) {
                core::cmp::Ordering::Less => "ok",
                core::cmp::Ordering::Equal => "FAILED",
                core::cmp::Ordering::Greater => "not run",
            };
            writeln!(f, "  {result} {}", test.name())?;
        }
        if !self.pass && run.current == run.tests.len() {
            writeln!(f, "  panic after the last test")?;
        }
        Ok(())
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
