//! Booting the image in QEMU.

use std::fs::{self, OpenOptions};
use std::process::Command;

use anyhow::{Context, Result};
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

use crate::cmd::exec;
use crate::{build, root};

const OVMF: Source = Source::EDK2_STABLE202605_R1;

/// Builds, then boots the image in QEMU with a window and serial on the
/// terminal.
pub fn run(release: bool, gdb: bool) -> Result<()> {
    let image = build::build(release)?;

    let ovmf = Prebuilt::fetch(OVMF, root().join("target/ovmf")).context("fetching OVMF")?;
    let code = ovmf.get_file(Arch::X64, FileType::Code);
    // The firmware writes its variables, so each run starts from a fresh copy.
    let vars = root().join("target/ovmf-vars.fd");
    fs::copy(ovmf.get_file(Arch::X64, FileType::Vars), &vars)
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
            image.path.display()
        ))
        .args(["-device", "usb-storage,bus=xhci.0,drive=stick"])
        .args(["-serial", "stdio", "-no-reboot"]);
    if kvm_usable() {
        qemu.args(["-accel", "kvm", "-cpu", "host"]);
    } else {
        qemu.args(["-accel", "tcg"]);
    }
    if gdb {
        qemu.args(["-s", "-S"]);
    }
    exec(&mut qemu)
}

fn kvm_usable() -> bool {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/kvm")
        .is_ok()
}
