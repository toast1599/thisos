//! The first process.

#![no_std]
#![no_main]
#![forbid(unsafe_code)]

thisos_rt::entry!(main);

fn main(process: thisos_rt::Handle) -> u64 {
    thisos_rt::println!("hello from init, holding handle {:#x}", process.value());
    drop(process);
    0
}
