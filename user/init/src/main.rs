//! The first process.

#![no_std]
#![no_main]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use thisos_rt::{Channel, Handle, Memory, Process, println};

thisos_rt::entry!(main);

fn main(process: Handle<Process>) -> u64 {
    let value = process.value();
    drop(process);
    // A message to itself, carrying a handle, so that loading init checks
    // channels work from Rust.
    let (first, second) = match Handle::<Channel>::create() {
        Ok(endpoints) => endpoints,
        Err(e) => panic!("creating a channel failed: {e:?}"),
    };
    let memory = match Handle::<Memory>::create(4096) {
        Ok(memory) => memory,
        Err(e) => panic!("creating a memory object failed: {e:?}"),
    };
    if let Err((e, _)) = first.send(b"ping", vec![memory.into_unknown()]) {
        panic!("sending failed: {e:?}");
    }
    let mut bytes = Vec::new();
    let handles = match second.receive(&mut bytes) {
        Ok(handles) => handles,
        Err(e) => panic!("receiving failed: {e:?}"),
    };
    let memory: Vec<Handle<Memory>> = handles.into_iter().map(Handle::cast).collect();
    // Formatted on the heap, so that loading init checks it works.
    let greeting = alloc::format!(
        "hello from init, holding handle {value:#x}; sent itself {:?} and {} handle over a channel",
        core::str::from_utf8(&bytes).unwrap_or("bytes that aren't UTF-8"),
        memory.len(),
    );
    println!("{greeting}");
    0
}
