//! The GIC, version 3: its distributor, the boot CPU's redistributor and the CPU
//! interface's system registers. Only the virtual timer's interrupt is
//! enabled; every other private and shared interrupt stays off.

use core::arch::asm;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, Ordering};

use aarch64_cpu::asm::barrier;
use aarch64_cpu::registers::{MPIDR_EL1, Readable};

use crate::devices::{DISTRIBUTOR_SIZE, Devices, REDISTRIBUTOR_SIZE};
use crate::ptr_at;

// Distributor registers.
const GICD_CTLR: u64 = 0x0;
const GICD_TYPER: u64 = 0x4;
const GICD_ICENABLER: u64 = 0x180;

// GICD_CTLR's bits as both its views give them: with one security state,
// ARE and EnableGrp1, and as non-secure software sees two, ARE_NS and
// EnableGrp1A.
const CTLR_ARE: u32 = 1 << 4;
const CTLR_ENABLE_GROUP_1: u32 = 1 << 1;
/// `GICD_CTLR`: a write is still taking effect.
const GICD_CTLR_RWP: u32 = 1 << 31;

// Redistributor registers: the RD_base frame, then the SGI_base frame 64
// KiB above it.
const GICR_CTLR: u64 = 0x0;
/// The upper half of `GICR_TYPER`, which holds the CPU's affinity.
const GICR_TYPER_AFFINITY: u64 = 0xc;
const GICR_WAKER: u64 = 0x14;
const SGI_BASE: u64 = 0x1_0000;
const GICR_IGROUPR0: u64 = SGI_BASE + 0x80;
const GICR_ISENABLER0: u64 = SGI_BASE + 0x100;
const GICR_ICENABLER0: u64 = SGI_BASE + 0x180;
const GICR_IPRIORITYR: u64 = SGI_BASE + 0x400;
const GICR_IGRPMODR0: u64 = SGI_BASE + 0xd00;
/// `GICR_CTLR`: a write to `ICENABLER0` is still taking effect.
const GICR_CTLR_RWP: u32 = 1 << 3;
const WAKER_PROCESSOR_SLEEP: u32 = 1 << 1;
const WAKER_CHILDREN_ASLEEP: u32 = 1 << 2;

/// The timer's priority: any value below `ICC_PMR_EL1`'s 0xff passes.
const PRIORITY: u32 = 0x80;

/// Interrupt IDs from here to 1023 are special: they name no interrupt,
/// and nothing ends them.
const SPECIAL: u32 = 1020;

/// One device's registers: `size` bytes of device memory from `base`.
struct Registers {
    base: *mut u32,
    size: u64,
}

impl Registers {
    /// The registers at `base`.
    ///
    /// # Safety
    ///
    /// The `size` bytes from `base` must be one device's registers, mapped
    /// as device memory for good, which nothing else reads or writes while
    /// this lives.
    unsafe fn new(base: u64, size: u64) -> Self {
        Registers {
            base: ptr_at(base),
            size,
        }
    }

    /// The 32-bit register `offset` bytes in.
    fn register(&self, offset: u64) -> *mut u32 {
        assert!(
            offset % 4 == 0 && offset + 4 <= self.size,
            "register {offset:#x} is outside the device's {:#x} bytes or unaligned",
            self.size
        );
        let Ok(index) = usize::try_from(offset / 4) else {
            unreachable!("aarch64 addresses are 64 bits");
        };
        self.base.wrapping_add(index)
    }

    fn read(&self, offset: u64) -> u32 {
        // SAFETY: register checked that the word is inside the registers,
        // which new's caller guarantees are mapped device memory that only
        // this reaches.
        unsafe { self.register(offset).read_volatile() }
    }

    fn write(&self, offset: u64, value: u32) {
        // SAFETY: As for read.
        unsafe { self.register(offset).write_volatile(value) }
    }

    /// Spins until the bits `mask` of register `offset` are clear.
    fn wait(&self, offset: u64, mask: u32) {
        while self.read(offset) & mask != 0 {
            spin_loop();
        }
    }
}

/// Sets up the distributor and the boot CPU's redistributor and CPU
/// interface at `phys_offset` plus the addresses in `devices`, to deliver
/// the timer's interrupt to this CPU as an IRQ. Panics if called twice, or
/// if the first redistributor isn't this CPU's.
pub fn init(phys_offset: u64, devices: &Devices) {
    static DONE: AtomicBool = AtomicBool::new(false);
    assert!(
        !DONE.swap(true, Ordering::Relaxed),
        "the GIC is set up already"
    );
    // SAFETY: take_page_tables mapped the distributor as device memory for
    // good, at phys_offset plus its address, and DONE makes this the only
    // place that reaches it, once.
    let distributor =
        unsafe { Registers::new(phys_offset + devices.distributor, DISTRIBUTOR_SIZE) };
    // SAFETY: As for the distributor.
    let redistributor =
        unsafe { Registers::new(phys_offset + devices.redistributor, REDISTRIBUTOR_SIZE) };

    // Affinity routing may change only while both groups are off.
    distributor.write(GICD_CTLR, 0);
    distributor.wait(GICD_CTLR, GICD_CTLR_RWP);
    // ITLinesNumber counts the blocks of 32 interrupts past the first,
    // which is the private interrupts' and lives in the redistributor.
    let blocks = (distributor.read(GICD_TYPER) & 0x1f) + 1;
    for block in 1..blocks {
        distributor.write(GICD_ICENABLER + 4 * u64::from(block), u32::MAX);
    }
    distributor.wait(GICD_CTLR, GICD_CTLR_RWP);
    distributor.write(GICD_CTLR, CTLR_ARE | CTLR_ENABLE_GROUP_1);
    distributor.wait(GICD_CTLR, GICD_CTLR_RWP);

    let affinity = redistributor.read(GICR_TYPER_AFFINITY);
    // Aff0 to Aff2 are MPIDR's low three bytes, and Aff3 its fifth.
    let mpidr = MPIDR_EL1.get().to_le_bytes();
    let cpu = u32::from_le_bytes([mpidr[0], mpidr[1], mpidr[2], mpidr[4]]);
    assert_eq!(
        affinity, cpu,
        "the first redistributor is CPU {affinity:#x}'s, not the boot CPU {cpu:#x}'s"
    );
    let waker = redistributor.read(GICR_WAKER);
    redistributor.write(GICR_WAKER, waker & !WAKER_PROCESSOR_SLEEP);
    redistributor.wait(GICR_WAKER, WAKER_CHILDREN_ASLEEP);

    let timer = 1 << devices.timer;
    redistributor.write(GICR_ICENABLER0, u32::MAX);
    redistributor.wait(GICR_CTLR, GICR_CTLR_RWP);
    // Group 1, and with two security states, the non-secure one.
    let group = redistributor.read(GICR_IGROUPR0);
    redistributor.write(GICR_IGROUPR0, group | timer);
    let mode = redistributor.read(GICR_IGRPMODR0);
    redistributor.write(GICR_IGRPMODR0, mode & !timer);
    // One priority byte per interrupt, four to a register.
    let priority = GICR_IPRIORITYR + u64::from(devices.timer / 4 * 4);
    let shift = 8 * (devices.timer % 4);
    let priorities = redistributor.read(priority) & !(0xff << shift);
    redistributor.write(priority, priorities | (PRIORITY << shift));
    redistributor.write(GICR_ISENABLER0, timer);

    // SAFETY: Setting ICC_SRE_EL1.SRE (S3_0_C12_C12_5, bit 0) only switches
    // the CPU interface to the system registers that the rest of this
    // module uses.
    unsafe {
        asm!(
            "mrs {tmp}, S3_0_C12_C12_5",
            "orr {tmp}, {tmp}, #1",
            "msr S3_0_C12_C12_5, {tmp}",
            "isb",
            tmp = out(reg) _,
            options(nostack, preserves_flags),
        );
    }
    let sre: u64;
    // SAFETY: Reading ICC_SRE_EL1 changes nothing.
    unsafe {
        asm!("mrs {}, S3_0_C12_C12_5", out(reg) sre, options(nomem, nostack, preserves_flags));
    }
    assert!(sre & 1 != 0, "EL2 keeps the GIC's system registers off");
    // SAFETY: ICC_PMR_EL1 (S3_0_C4_C6_0) at 0xff lets every priority
    // through; interrupts stay masked in DAIF until the kernel turns them
    // on.
    unsafe {
        asm!("msr S3_0_C4_C6_0, {}", in(reg) 0xff_u64, options(nomem, nostack, preserves_flags));
    }
    // SAFETY: ICC_BPR1_EL1 (S3_0_C12_C12_3) at 0 only decides which
    // priorities preempt each other, and the kernel never nests interrupts.
    unsafe {
        asm!(
            "msr S3_0_C12_C12_3, xzr",
            options(nomem, nostack, preserves_flags)
        );
    }
    // SAFETY: Clearing ICC_CTLR_EL1.EOImode (S3_0_C12_C12_4, bit 1) makes a
    // write to EOIR deactivate the interrupt too, as end expects.
    unsafe {
        asm!(
            "mrs {tmp}, S3_0_C12_C12_4",
            "bic {tmp}, {tmp}, #2",
            "msr S3_0_C12_C12_4, {tmp}",
            tmp = out(reg) _,
            options(nostack, preserves_flags),
        );
    }
    // SAFETY: Setting ICC_IGRPEN1_EL1 (S3_0_C12_C12_7) lets group 1
    // interrupts reach this CPU, still masked in DAIF.
    unsafe {
        asm!("msr S3_0_C12_C12_7, {}", in(reg) 1_u64, options(nomem, nostack, preserves_flags));
    }
    barrier::isb(barrier::SY);
}

/// The group an interrupt was signalled in: group 1 as an IRQ, group 0 as
/// an FIQ.
#[derive(Clone, Copy, Debug)]
pub enum Group {
    /// FIQ.
    Zero,
    /// IRQ.
    One,
}

/// An interrupt this CPU acknowledged. Until [`Acknowledged::end`],
/// nothing of its priority or lower reaches the CPU.
#[derive(Debug)]
#[must_use = "an acknowledged interrupt blocks others until it ends"]
pub struct Acknowledged {
    intid: u32,
    group: Group,
}

/// Acknowledges the highest-priority pending interrupt in `group`, or
/// returns the special interrupt ID that the CPU interface gave instead,
/// such as 1023 when nothing is pending.
pub fn acknowledge(group: Group) -> Result<Acknowledged, u32> {
    let iar: u64;
    match group {
        // SAFETY: Reading ICC_IAR0_EL1 (S3_0_C12_C8_0) marks the interrupt
        // it returns active, which the caller ends through Acknowledged.
        Group::Zero => unsafe {
            asm!("mrs {}, S3_0_C12_C8_0", out(reg) iar, options(nostack, preserves_flags));
        },
        // SAFETY: As for ICC_IAR0_EL1, with ICC_IAR1_EL1 (S3_0_C12_C12_0).
        Group::One => unsafe {
            asm!("mrs {}, S3_0_C12_C12_0", out(reg) iar, options(nostack, preserves_flags));
        },
    }
    // The ID is the low 24 bits.
    let [a, b, c, ..] = iar.to_le_bytes();
    let intid = u32::from_le_bytes([a, b, c, 0]);
    if (SPECIAL..1024).contains(&intid) {
        return Err(intid);
    }
    Ok(Acknowledged { intid, group })
}

impl Acknowledged {
    /// The interrupt's ID.
    pub fn intid(&self) -> u32 {
        self.intid
    }

    /// Ends the interrupt: drops the CPU's priority and deactivates it.
    pub fn end(self) {
        let intid = u64::from(self.intid);
        match self.group {
            // SAFETY: Writing ICC_EOIR0_EL1 (S3_0_C12_C8_1) ends the
            // interrupt acknowledge returned, which is still active.
            Group::Zero => unsafe {
                asm!("msr S3_0_C12_C8_1, {}", in(reg) intid, options(nostack, preserves_flags));
            },
            // SAFETY: As for ICC_EOIR0_EL1, with ICC_EOIR1_EL1
            // (S3_0_C12_C12_1).
            Group::One => unsafe {
                asm!("msr S3_0_C12_C12_1, {}", in(reg) intid, options(nostack, preserves_flags));
            },
        }
    }
}
