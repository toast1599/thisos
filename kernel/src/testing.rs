//! The test runner for the kernel's `#[test_case]` functions, and for the
//! faulting tests in `tests/`. It ends on a summary of the results with a
//! large PASS or FAIL, which is all the laptop shows, then exits QEMU.

use core::fmt;

use spin::Mutex;
use thisos_hal::Platform;

use crate::{Arch, logger};

/// The tests and the index of the one running, or of none once all passed,
/// for the panic handler's summary. `None` until the tests start.
static RUN: Mutex<Option<Run>> = Mutex::new(None);

#[derive(Clone, Copy)]
struct Run {
    tests: &'static [&'static dyn Test],
    current: usize,
}

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
        *RUN.lock() = Some(Run { tests, current });
        log::info!("test {}", test.name());
        test.run();
    }
    // Before the log line, so that a panic in it doesn't blame the last test.
    *RUN.lock() = Some(Run {
        tests,
        current: tests.len(),
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
