//! The two legacy 8259 PICs. The kernel takes no interrupts from them, so
//! `disable` moves their vectors to 0x20-0x2f, where a stray line can't pass
//! for an exception, and masks every line.

use x86_64::instructions::port::Port;

const PRIMARY: u16 = 0x20;
const SECONDARY: u16 = 0xa0;

/// The primary PIC's first vector. The secondary's eight follow its eight.
const BASE: u8 = 0x20;

/// Remaps both PICs to vectors 0x20-0x2f and masks all their lines.
///
/// # Safety
///
/// Call this once, with interrupts off, before anything else uses the PICs'
/// ports.
pub unsafe fn disable() {
    // ICW1: edge-triggered, cascaded, ICW4 follows. ICW2: the vector base.
    // ICW3: the secondary on the primary's line 2. ICW4: 8086 mode. Then
    // OCW1, the masks.
    let writes = [
        (PRIMARY, 0x11),
        (SECONDARY, 0x11),
        (PRIMARY + 1, BASE),
        (SECONDARY + 1, BASE + 8),
        (PRIMARY + 1, 1 << 2),
        (SECONDARY + 1, 2),
        (PRIMARY + 1, 0x01),
        (SECONDARY + 1, 0x01),
        (PRIMARY + 1, 0xff),
        (SECONDARY + 1, 0xff),
    ];
    for (port, value) in writes {
        // SAFETY: These are the PICs' own ports, which the caller guarantees
        // nothing else uses, and the sequence is the 8259's initialization.
        // With interrupts off, no IRQ arrives while it is half done. Port
        // I/O touches no memory.
        unsafe { Port::<u8>::new(port).write(value) };
        settle();
    }
}

/// Whether both PICs mask every line.
pub fn masked() -> bool {
    [PRIMARY + 1, SECONDARY + 1].into_iter().all(|port| {
        // SAFETY: Outside an initialization sequence, which only `disable`
        // runs, reading a PIC's data port returns its mask and changes
        // nothing.
        let mask = unsafe { Port::<u8>::new(port).read() };
        mask == 0xff
    })
}

/// Waits about a microsecond, which chipset PICs want between commands, by
/// writing the POST code port, which takes about that long and does nothing
/// else.
fn settle() {
    // SAFETY: Port 0x80 is the POST code port, which only displays the byte
    // on a debug card, if there is one. Port I/O touches no memory.
    unsafe { Port::<u8>::new(0x80).write(0) };
}
