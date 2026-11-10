//! Booting images in QEMU.

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use ovmf_prebuilt::{FileType, Prebuilt, Source};

use crate::build::{self, Arch, Kernel};
use crate::cmd::exec;
use crate::root;

const OVMF: Source = Source::EDK2_STABLE202605_R1;

/// How long a test binary may run before QEMU is killed. TCG on a CI
/// runner boots OVMF in well under this.
const TEST_TIMEOUT: Duration = Duration::from_secs(60);

/// The status QEMU exits with when the kernel reports success: through
/// isa-debug-exit, (0x10 << 1) | 1, and through semihosting the same number,
/// so that one constant covers both.
const TEST_SUCCESS: i32 = 33;

/// Builds, then boots the image in QEMU with a window and serial on the
/// terminal.
pub fn run(arch: Arch, release: bool, gdb: bool) -> Result<()> {
    let image = build::build(arch, release, Kernel::Main)?;
    let vars = root().join(format!("target/ovmf-vars{}.fd", arch.suffix()));
    let mut qemu = qemu(arch, &image.path, &vars)?;
    if gdb {
        qemu.args(["-s", "-S"]);
    }
    exec(&mut qemu)
}

/// Boots a kernel test ELF headless, as cargo's runner, and fails unless the
/// kernel reports success before the timeout.
pub fn test(arch: Arch, elf: &Path) -> Result<()> {
    let image = with_extension(elf, "img");
    arch.write_image(elf, None, &image)?;

    let mut qemu = qemu(arch, &image, &with_extension(elf, "vars.fd"))?;
    qemu.args(["-display", "none"]);
    match arch {
        Arch::X86_64 => qemu.args(["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]),
        // target=native lets the guest open host files too, which only our
        // own test kernels get to do.
        Arch::Aarch64 => qemu.args(["-semihosting-config", "enable=on,target=native"]),
    };
    // QEMU would otherwise put the terminal in raw mode, and a kill on
    // timeout leaves it that way.
    qemu.stdin(Stdio::null());
    let mut child = qemu
        .spawn()
        .with_context(|| format!("running {}", qemu.get_program().display()))?;
    let deadline = Instant::now() + TEST_TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait().context("waiting for QEMU")? {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().context("killing QEMU")?;
            child.wait().context("waiting for QEMU")?;
            bail!(
                "{} timed out after {} s",
                elf.display(),
                TEST_TIMEOUT.as_secs()
            );
        }
        thread::sleep(Duration::from_millis(100));
    };
    if status.code() != Some(TEST_SUCCESS) {
        bail!(
            "{} failed: QEMU exited with {status}, and only {TEST_SUCCESS} is a pass",
            elf.display()
        );
    }
    Ok(())
}

/// A QEMU command that boots `image` under OVMF, with serial on standard
/// output. The firmware writes its variables, so each boot starts from a
/// fresh copy at `vars`.
fn qemu(arch: Arch, image: &Path, vars: &Path) -> Result<Command> {
    let firmware = match arch {
        Arch::X86_64 => ovmf_prebuilt::Arch::X64,
        Arch::Aarch64 => ovmf_prebuilt::Arch::Aarch64,
    };
    let ovmf = Prebuilt::fetch(OVMF, root().join("target/ovmf")).context("fetching OVMF")?;
    let code = ovmf.get_file(firmware, FileType::Code);
    fs::copy(ovmf.get_file(firmware, FileType::Vars), vars)
        .with_context(|| format!("copying OVMF vars to {}", vars.display()))?;

    let mut qemu = match arch {
        Arch::X86_64 => {
            let mut qemu = Command::new("qemu-system-x86_64");
            qemu.args(["-machine", "q35"]);
            if kvm_usable() {
                qemu.args(["-accel", "kvm", "-cpu", "host"]);
            } else {
                // The kernel needs x2APIC, SMEP and SMAP, which TCG emulates
                // but its default qemu64 model leaves off.
                qemu.args(["-accel", "tcg", "-cpu", "qemu64,+x2apic,+smep,+smap"]);
            }
            qemu
        }
        Arch::Aarch64 => {
            // The host is x86_64, so aarch64 always runs under TCG. ramfb
            // is a plain framebuffer that edk2 hands on as GOP, with no
            // virtio. Without ACPI tables, edk2 hands Limine the device tree
            // instead, which the kernel finds its devices in.
            let mut qemu = Command::new("qemu-system-aarch64");
            qemu.args(["-machine", "virt,gic-version=3,acpi=off"])
                .args(["-cpu", "cortex-a76", "-device", "ramfb"]);
            qemu
        }
    };
    qemu.args(["-m", "512M", "-smp", "1"])
        .arg("-drive")
        .arg(format!(
            "if=pflash,format=raw,readonly=on,file={}",
            code.display()
        ))
        .arg("-drive")
        .arg(format!("if=pflash,format=raw,file={}", vars.display()))
        .args(["-device", "qemu-xhci,id=xhci"])
        .arg("-drive")
        .arg(format!(
            "if=none,id=stick,format=raw,file={}",
            image.display()
        ))
        .args(["-device", "usb-storage,bus=xhci.0,drive=stick"])
        .args(["-serial", "stdio", "-no-reboot"]);
    Ok(qemu)
}

fn with_extension(path: &Path, extension: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".");
    name.push(extension);
    name.into()
}

fn kvm_usable() -> bool {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/kvm")
        .is_ok()
}
