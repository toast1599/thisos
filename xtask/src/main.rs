use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

const USAGE: &str = "usage: cargo xtask <command> [options]

commands:
  build [--release]          build the kernel and write target/thisos.img
  run [--release] [--gdb]    build, then boot the image in QEMU
                             (--gdb: gdb stub on tcp 1234, start paused)";

const OVMF: Source = Source::EDK2_STABLE202605_R1;

struct Options {
    release: bool,
    gdb: bool,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((cmd, flags)) = args.split_first() else {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    };
    let result = match cmd.as_str() {
        "help" | "-h" | "--help" => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        "build" => parse(flags, false)
            .and_then(|opts| build(&opts).map(|img| println!("{}", img.display()))),
        "run" => parse(flags, true).and_then(|opts| run(&opts)),
        _ => Err(format!("unknown command: {cmd}\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn parse(flags: &[String], allow_gdb: bool) -> Result<Options, String> {
    let mut opts = Options {
        release: false,
        gdb: false,
    };
    for flag in flags {
        match flag.as_str() {
            "--release" => opts.release = true,
            "--gdb" if allow_gdb => opts.gdb = true,
            _ => return Err(format!("unknown option: {flag}\n\n{USAGE}")),
        }
    }
    Ok(opts)
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repository root")
}

/// Builds the kernel and writes the UEFI disk image, returning its path.
fn build(opts: &Options) -> Result<PathBuf, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.current_dir(root())
        .args(["build", "-p", "thisos-kernel"]);
    if opts.release {
        cmd.arg("--release");
    }
    let status = cmd.status().map_err(|e| format!("running cargo: {e}"))?;
    if !status.success() {
        return Err("kernel build failed".into());
    }

    let profile = if opts.release { "release" } else { "debug" };
    let kernel = root()
        .join("target/x86_64-unknown-none")
        .join(profile)
        .join("thisos-kernel");
    let image = root().join("target/thisos.img");
    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&image)
        .map_err(|e| format!("writing {}: {e:#}", image.display()))?;
    Ok(image)
}

fn run(opts: &Options) -> Result<(), String> {
    let image = build(opts)?;

    let ovmf = Prebuilt::fetch(OVMF, root().join("target/ovmf"))
        .map_err(|e| format!("fetching OVMF: {e}"))?;
    let code = ovmf.get_file(Arch::X64, FileType::Code);
    // The firmware writes its variables, so each run starts from a fresh copy.
    let vars = root().join("target/ovmf-vars.fd");
    fs::copy(ovmf.get_file(Arch::X64, FileType::Vars), &vars)
        .map_err(|e| format!("copying OVMF vars to {}: {e}", vars.display()))?;

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
        qemu.args(["-accel", "tcg"]);
    }
    if opts.gdb {
        qemu.args(["-s", "-S"]);
    }

    let status = qemu
        .status()
        .map_err(|e| format!("running qemu-system-x86_64: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("QEMU exited with {status}"))
    }
}

fn kvm_usable() -> bool {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/kvm")
        .is_ok()
}
