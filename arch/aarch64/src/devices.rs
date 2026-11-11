//! The devices the arch crate drives itself, as the device tree Limine
//! hands over names them.

use core::ops::Range;

use thisos_fdt::{Fdt, Node, Region};

use crate::once::Once;

/// The size of the boot CPU's redistributor, its two 64 KiB frames: the
/// only part of the redistributor region the kernel maps, since SMP is a
/// non-goal.
pub const REDISTRIBUTOR_SIZE: u64 = 0x2_0000;

/// The distributor's size.
pub const DISTRIBUTOR_SIZE: u64 = 0x1_0000;

/// The physical addresses of the GIC's registers, the virtual timer's
/// interrupt, and whether the machine is QEMU's virt.
#[derive(Clone, Copy, Debug)]
pub struct Devices {
    /// The GIC distributor's registers.
    pub distributor: u64,
    /// The boot CPU's GIC redistributor's registers.
    pub redistributor: u64,
    /// The virtual timer's interrupt ID, a private peripheral interrupt.
    pub timer: u32,
    /// Whether the root is compatible with `linux,dummy-virt`, as only
    /// QEMU's virt machine's is.
    pub qemu_virt: bool,
}

impl Devices {
    /// The physical addresses of the registers the kernel maps.
    pub fn registers(&self) -> [Range<u64>; 2] {
        [
            self.distributor..self.distributor + DISTRIBUTOR_SIZE,
            self.redistributor..self.redistributor + REDISTRIBUTOR_SIZE,
        ]
    }
}

static DEVICES: Once<Devices> = Once::new();

/// Whether the machine is QEMU's virt, and `false` before `init` has read
/// the device tree. Never panics, for the panic path.
pub fn qemu_virt() -> bool {
    DEVICES.get().is_some_and(|devices| devices.qemu_virt)
}

/// The devices `init` found.
pub fn get() -> Devices {
    let Some(devices) = DEVICES.get() else {
        panic!("boot_info hasn't read the device tree");
    };
    *devices
}

/// Finds the devices in `fdt`. Panics if one is missing or isn't
/// described as QEMU's virt machine describes it.
pub fn init(fdt: &Fdt<'_>) {
    let gic = compatible(fdt, "arm,gic-v3");
    let interrupt_cells = gic.property("#interrupt-cells");
    assert!(
        interrupt_cells == Some(&3u32.to_be_bytes()[..]),
        "the GIC's #interrupt-cells is {interrupt_cells:?}, not 3"
    );
    let Some(mut reg) = gic.reg() else {
        panic!("the GIC has no reg it can read");
    };
    let (Some(distributor), Some(redistributor)) = (reg.next(), reg.next()) else {
        panic!("the GIC's reg lacks the distributor or the redistributors");
    };
    check_size("distributor", distributor, DISTRIBUTOR_SIZE);
    check_size("redistributor", redistributor, REDISTRIBUTOR_SIZE);

    // Its interrupts are the secure and non-secure physical timers', then
    // the virtual timer's, then the hypervisor's, each as type, number and
    // flags.
    let timer = compatible(fdt, "arm,armv8-timer");
    let Some(interrupts) = timer.property("interrupts") else {
        panic!("the timer has no interrupts");
    };
    let cells: [u32; 3] = core::array::from_fn(|i| {
        interrupts
            .get(24 + 4 * i..28 + 4 * i)
            .and_then(|cell| cell.try_into().ok())
            .map_or(u32::MAX, u32::from_be_bytes)
    });
    let [kind, number, _] = cells;
    // Type 1 is a private peripheral interrupt, whose IDs start at 16.
    assert!(
        kind == 1 && number < 16,
        "the virtual timer's interrupt isn't a PPI: {cells:?}"
    );
    DEVICES.init(Devices {
        distributor: distributor.address,
        redistributor: redistributor.address,
        timer: 16 + number,
        qemu_virt: fdt
            .root()
            .is_some_and(|root| root.compatible().any(|c| c == "linux,dummy-virt")),
    });
}

/// The first node compatible with `name`.
fn compatible<'a>(fdt: &Fdt<'a>, name: &str) -> Node<'a> {
    let Some(node) = fdt
        .nodes()
        .find(|node| node.compatible().any(|c| c == name))
    else {
        panic!("the device tree has no {name} node");
    };
    node
}

fn check_size(what: &str, region: Region, size: u64) {
    assert!(
        region.address % 4096 == 0 && region.size >= size,
        "the GIC's {what} at {:#x}, {:#x} bytes, is smaller than {size:#x} or unaligned",
        region.address,
        region.size
    );
}
