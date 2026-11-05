//! A kernel stack overflow runs into the guard page below the stack. The CPU
//! can't push the page fault's frame there, so it raises a double fault,
//! which reaches the trap handler on a stack of its own.

#![no_std]
#![no_main]
#![forbid(unsafe_code)]

use core::hint::black_box;

use thisos_hal::{BootInfo, Platform, Trap};
use thisos_kernel::{Arch, testing};

thisos_kernel::entry!(main, trap);

fn main(boot: BootInfo<Arch>) -> ! {
    thisos_kernel::init(boot);
    testing::run(&[&overflows_into_a_double_fault])
}

fn trap(trap: Trap, frame: &mut <Arch as Platform>::TrapFrame) {
    if trap == Trap::DoubleFault {
        testing::pass();
    }
    thisos_kernel::trap(trap, frame);
}

fn overflows_into_a_double_fault() {
    recurse(0);
}

// black_box keeps the recursion from becoming a loop or a tail call, either
// of which would stop the stack growing.
#[inline(never)]
fn recurse(depth: u64) -> u64 {
    if black_box(depth) == u64::MAX {
        return depth;
    }
    black_box(recurse(depth + 1))
}
