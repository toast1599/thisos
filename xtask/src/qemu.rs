//! Booting images in QEMU.

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

use crate::build::{self, Kernel};
use crate::cmd::exec;
use crate::root;

const OVMF: Source = Source::EDK2_STABLE202605_R1;

/// How long a test binary may run before QEMU is killed. TCG on a CI
/// runner boots OVMF in well under this.
const TEST_TIMEOUT: Duration = Duration::from_secs(60);

/// The status QEMU exits with when the kernel reports success through
/// isa-debug-exit: (0x10 << 1) | 1.
const TEST_SUCCESS: i32 = 33;

/// Builds, then boots the image in QEMU with a window and serial on the
/// terminal.
pub fn run(release: bool, gdb: bool) -> Result<()> {
    let image = build::build(release, Kernel::Main)?;
    let mut qemu = qemu(&image.path, &root().join("target/ovmf-vars.fd"))?;
    if gdb {
        qemu.args(["-s", "-S"]);
    }
    exec(&mut qemu)
}

/// Boots a kernel test ELF headless, as cargo's runner, and fails unless the
/// kernel reports success before the timeout.
pub fn test(elf: &Path) -> Result<()> {
    let image = with_extension(elf, "img");
    bootloader::DiskImageBuilder::new(elf.to_path_buf())
        .create_uefi_image(&image)
        .with_context(|| format!("writing {}", image.display()))?;

    let mut qemu = qemu(&image, &with_extension(elf, "vars.fd"))?;
    qemu.args(["-display", "none"])
        .args(["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"])
        // QEMU would otherwise put the terminal in raw mode, and a kill on
        // timeout leaves it that way.
        .stdin(Stdio::null());
    let mut child = qemu.spawn().context("running qemu-system-x86_64")?;
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
fn qemu(image: &Path, vars: &Path) -> Result<Command> {
    let ovmf = Prebuilt::fetch(OVMF, root().join("target/ovmf")).context("fetching OVMF")?;
    let code = ovmf.get_file(Arch::X64, FileType::Code);
    fs::copy(ovmf.get_file(Arch::X64, FileType::Vars), vars)
        .with_context(|| format!("copying OVMF vars to {}", vars.display()))?;

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.args(["-machine", "q35", "-m", "512M", "-smp", "1"])
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
    if kvm_usable() {
        qemu.args(["-accel", "kvm", "-cpu", "host"]);
    } else {
        // The kernel needs x2APIC, which TCG emulates but its default
        // qemu64 model leaves off.
        qemu.args(["-accel", "tcg", "-cpu", "qemu64,+x2apic"]);
    }
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
