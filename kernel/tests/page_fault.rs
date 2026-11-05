//! A read of an unmapped page reaches the trap handler as a page fault, with
//! the address and the kind of access.

#![no_std]
#![no_main]
#![forbid(unsafe_code)]

use thisos_hal::{Access, BootInfo, Platform, Trap};
use thisos_kernel::{Arch, testing};

thisos_kernel::entry!(main, trap);

fn main(boot: BootInfo<Arch>) -> ! {
    thisos_kernel::init(boot);
    testing::run(&[&reads_the_guard_page])
}

fn trap(trap: Trap, frame: &mut <Arch as Platform>::TrapFrame) {
    let expected = Trap::PageFault {
        address: Arch::guard_page(),
        access: Access::Read,
    };
    if trap == expected {
        testing::pass();
    }
    thisos_kernel::trap(trap, frame);
}

fn reads_the_guard_page() {
    Arch::read_guard_page();
}
