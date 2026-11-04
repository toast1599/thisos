//! The QEMU test runner for the kernel's `#[test_case]` functions.

use spin::Mutex;
use thisos_hal::Platform;

use crate::{Arch, logger};

/// The test that is running, for the panic handler to name.
static CURRENT: Mutex<Option<&'static str>> = Mutex::new(None);

/// A `#[test_case]` function.
pub trait Test {
    fn name(&self) -> &'static str;
    fn run(&self);
}

impl<T: Fn()> Test for T {
    fn name(&self) -> &'static str {
        core::any::type_name::<T>()
    }

    fn run(&self) {
        self();
    }
}

/// Runs the tests in order, then exits QEMU with success. A failing test
/// panics, and the panic handler exits QEMU with failure.
pub fn run(tests: &[&dyn Test]) -> ! {
    for test in tests {
        *CURRENT.lock() = Some(test.name());
        log::info!("test {}", test.name());
        test.run();
    }
    log::info!("{} tests passed", tests.len());
    Arch::exit_qemu(true)
}

/// Names the failed test after its panic report. The logger may be the
/// thing that panicked, so this goes through the panic path.
pub fn report_failure() {
    if let Some(Some(name)) = CURRENT.try_lock().map(|current| *current) {
        logger::panic(format_args!("test {name} failed"), &"");
    }
}
