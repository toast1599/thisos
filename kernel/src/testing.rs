//! The test runner for the kernel's `#[test_case]` functions, and for the
//! faulting tests in `tests/`. It ends on a summary of the results with a
//! large PASS or FAIL, which is all the laptop shows, then exits QEMU.

use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};

use spin::Mutex;
use thisos_hal::Platform;

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
static LIMIT: AtomicU64 = AtomicU64::new(10);

/// A test: a `#[test_case]` function, or a function a faulting test passes
/// to [`run`].
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

/// Ends the run with the running test passed and the rest not run. A test
/// that passes by faulting calls this from its trap handler.
pub fn pass() -> ! {
    if let Some(mut run) = RUN.try_lock()
        && let Some(run) = &mut *run
    {
        run.current += 1;
    }
    finish(true)
}

/// Sets how long a test may run before the watchdog stops it, in place of
/// 10 s. For `tests/watchdog.rs`, so that it doesn't wait that long.
pub fn set_watchdog(seconds: u64) {
    LIMIT.store(seconds, Ordering::Relaxed);
}

/// Whether the running test has run past the watchdog's limit. For a test
/// that passes when the watchdog fires.
pub fn watchdog_expired() -> bool {
    overdue().is_some()
}

/// Stops the running test with a report of where it was, once it has run
/// past the watchdog's limit. The kernel's trap handler calls this on every
/// timer interrupt.
pub(crate) fn watchdog(frame: &<Arch as Platform>::TrapFrame) {
    if let Some(test) = overdue() {
        panic::trapped(
            format_args!(
                "test {} timed out after {} s",
                test.name(),
                LIMIT.load(Ordering::Relaxed)
            ),
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
    let limit = LIMIT.load(Ordering::Relaxed) * u64::from(timer::HZ);
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
