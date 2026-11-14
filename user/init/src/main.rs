//! The first process.

#![no_std]
#![no_main]
#![forbid(unsafe_code)]

extern crate alloc;

thisos_rt::entry!(main);

fn main(process: thisos_rt::Handle) -> u64 {
    // Formatted on the heap, so that loading init checks it works.
    let greeting = alloc::format!("hello from init, holding handle {:#x}", process.value());
    thisos_rt::println!("{greeting}");
    drop(process);
    0
}
