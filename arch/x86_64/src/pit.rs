//! Channel 2 of the legacy 8254 PIT, as a clock to measure the APIC timer
//! against. Recent Intel firmware can gate the PIT's clock, so `count`
//! gives up when it doesn't count, after a time the TSC measures: how long
//! a port read takes varies too much between machines to bound it by
//! reads.

use x86_64::instructions::port::Port;

use crate::tsc;

/// The PIT's input clock, in Hz.
pub const HZ: u64 = 1_193_182;

const CHANNEL_2: u16 = 0x42;
const COMMAND: u16 = 0x43;
/// Port 0x61, which holds channel 2's gate and output and the speaker's
/// enable.
const CONTROL: u16 = 0x61;

const GATE: u8 = 1 << 0;
const SPEAKER: u8 = 1 << 1;
const OUTPUT: u8 = 1 << 5;

/// Channel 2, low byte then high byte, mode 0 (output high at terminal
/// count), binary.
const ONE_SHOT: u8 = 0b1011_0000;

/// How many TSC cycles `count` waits before deciding the PIT doesn't count:
/// 4.3 s for a TSC of 1 GHz down to 1.1 s at 4 GHz, where a count takes tens
/// of milliseconds.
const MAX_CYCLES: u64 = 1 << 32;

/// Counts `ticks` of [`HZ`] on channel 2, with the speaker off. It calls
/// `start` right before the count begins, and returns once the count ends,
/// or `false` if it never does.
pub fn count(ticks: u16, start: impl FnOnce()) -> bool {
    let control = read(CONTROL) & !(SPEAKER | GATE);
    write(CONTROL, control);
    write(COMMAND, ONE_SHOT);
    let [low, high] = ticks.to_le_bytes();
    write(CHANNEL_2, low);
    write(CHANNEL_2, high);
    start();
    let started = tsc::read();
    write(CONTROL, control | GATE);
    let ended = loop {
        if read(CONTROL) & OUTPUT != 0 {
            break true;
        }
        if tsc::read().wrapping_sub(started) > MAX_CYCLES {
            break false;
        }
    };
    write(CONTROL, control);
    ended
}

fn read(port: u16) -> u8 {
    // SAFETY: The module reads only port 0x61, whose read has no side
    // effects. Port I/O touches no memory.
    unsafe { Port::<u8>::new(port).read() }
}

fn write(port: u16, value: u8) {
    // SAFETY: The module writes only channel 2, the PIT's command port and
    // port 0x61, keeping 0x61's other bits as read. Channel 2 drives only
    // the speaker, which `count` keeps off, and nothing else in the kernel
    // uses these ports. Port I/O touches no memory.
    unsafe { Port::<u8>::new(port).write(value) };
}
