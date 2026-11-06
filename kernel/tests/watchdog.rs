//! A test that never returns is stopped by the watchdog, here after 1 s so
//! that the run doesn't wait the usual 10.

#![no_std]
#![no_main]
#![forbid(unsafe_code)]

use core::hint::spin_loop;

use thisos_hal::{BootInfo, Platform, Trap};
use thisos_kernel::{Arch, testing};

thisos_kernel::entry!(main, trap);

fn main(boot: BootInfo<Arch>) -> ! {
    thisos_kernel::init(boot);
    testing::set_watchdog(1);
    testing::run(&[&spins_forever])
}

fn trap(trap: Trap, frame: &mut <Arch as Platform>::TrapFrame) {
    if trap == Trap::Timer && testing::watchdog_expired() {
        testing::pass();
    }
    thisos_kernel::trap(trap, frame);
}

fn spins_forever() {
    loop {
        spin_loop();
    }
}
