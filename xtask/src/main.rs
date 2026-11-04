use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

const USAGE: &str = "usage: cargo xtask <command> [options]

commands:
  build [--release]          build the kernel and write target/thisos.img
  run [--release] [--gdb]    build, then boot the image in QEMU
                             (--gdb: gdb stub on tcp 1234, start paused)
  flash [--release] [--device PATH] [--yes]
                             build, then write the image to a USB stick,
                             verify it and power the stick off
                             (--yes: don't ask before writing)
  font                       rasterize the console font into lib/console/src/font";

const OVMF: Source = Source::EDK2_STABLE202605_R1;

/// The console's glyph sizes in pixels per em, smallest first.
const FONT_SIZES: [u32; 4] = [12, 16, 24, 32];

struct Options {
    release: bool,
    gdb: bool,
}

struct FlashOptions {
    build: Options,
    device: Option<String>,
    yes: bool,
}

/// A whole disk, as `lsblk` describes it.
struct Disk {
    path: String,
    size: u64,
    model: String,
    serial: String,
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
        "flash" => parse_flash(flags).and_then(|opts| flash(&opts)),
        "font" => match flags.first() {
            Some(flag) => Err(format!("unknown option: {flag}\n\n{USAGE}")),
            None => font(),
        },
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

fn parse_flash(flags: &[String]) -> Result<FlashOptions, String> {
    let mut opts = FlashOptions {
        build: Options {
            release: false,
            gdb: false,
        },
        device: None,
        yes: false,
    };
    let mut flags = flags.iter();
    while let Some(flag) = flags.next() {
        match flag.as_str() {
            "--release" => opts.build.release = true,
            "--yes" => opts.yes = true,
            "--device" => opts.device = Some(flags.next().ok_or("--device needs a path")?.clone()),
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

/// Rasterizes printable ASCII at each of `FONT_SIZES` into one file of
/// 8-bit coverage per size, glyph after glyph, each a cell of rows top to
/// bottom, and writes the `font.rs` that embeds them.
fn font() -> Result<(), String> {
    let ttf_path = root().join("xtask/assets/JetBrainsMono-Regular.ttf");
    let ttf = fs::read(&ttf_path).map_err(|e| format!("reading {}: {e}", ttf_path.display()))?;
    let font = fontdue::Font::from_bytes(ttf, fontdue::FontSettings::default())
        .map_err(|e| format!("parsing {}: {e}", ttf_path.display()))?;
    let out = root().join("lib/console/src/font");
    fs::create_dir_all(&out).map_err(|e| format!("creating {}: {e}", out.display()))?;

    let chars = ' '..='~';
    let count = chars.clone().count();
    let mut statics = String::new();
    let mut entries = String::new();
    for px in FONT_SIZES {
        let size = px as f32;
        let line = font
            .horizontal_line_metrics(size)
            .ok_or("the font has no horizontal line metrics")?;
        let width = font.metrics('M', size).advance_width.round() as usize;
        let height = line.new_line_size.ceil() as usize;
        let baseline = line.ascent.round() as i32;

        let mut glyphs = vec![0u8; count * width * height];
        let mut clipped = String::new();
        for (cell, c) in glyphs.chunks_exact_mut(width * height).zip(chars.clone()) {
            let (metrics, bitmap) = font.rasterize(c, size);
            if metrics.advance_width.round() as usize != width {
                return Err(format!("{c:?} at {px} px isn't {width} px wide"));
            }
            // fontdue's ymin is the bitmap's bottom edge, upwards from the baseline.
            let top = baseline - metrics.ymin - metrics.height as i32;
            let mut lost = false;
            for (gy, row) in bitmap.chunks_exact(metrics.width.max(1)).enumerate() {
                for (gx, &coverage) in row.iter().enumerate() {
                    let x = usize::try_from(metrics.xmin + gx as i32).ok();
                    let y = usize::try_from(top + gy as i32).ok();
                    match (x, y) {
                        (Some(x), Some(y)) if x < width && y < height => {
                            cell[y * width + x] = coverage;
                        }
                        _ => lost |= coverage != 0,
                    }
                }
            }
            if lost {
                clipped.push(c);
            }
        }

        let file = out.join(format!("{px}.bin"));
        fs::write(&file, &glyphs).map_err(|e| format!("writing {}: {e}", file.display()))?;
        println!("{px} px: {width}x{height} cells, clipped: {clipped:?}");
        statics.push_str(&format!(
            "static GLYPHS_{px}: &[u8; {count} * {width} * {height}] = include_bytes!(\"font/{px}.bin\");\n"
        ));
        entries.push_str(&format!(
            "    Font {{\n        width: {width},\n        height: {height},\n        glyphs: GLYPHS_{px},\n    }},\n"
        ));
    }

    let rs = format!(
        "//! JetBrains Mono Regular, printable ASCII, as 8-bit coverage.\n\
         //! Generated by `cargo xtask font`; don't edit.\n\
         \n\
         use crate::Font;\n\
         \n\
         {statics}\n\
         /// The sizes, smallest first.\n\
         pub(crate) static FONTS: [Font; {}] = [\n\
         {entries}];\n",
        FONT_SIZES.len()
    );
    let file = root().join("lib/console/src/font.rs");
    fs::write(&file, rs).map_err(|e| format!("writing {}: {e}", file.display()))
}

/// Builds the image, writes it to a USB stick with `sudo dd`, reads it back
/// to check it, moves the backup GPT to the end of the stick and powers the
/// stick off.
fn flash(opts: &FlashOptions) -> Result<(), String> {
    let image = build(&opts.build)?;
    let data = fs::read(&image).map_err(|e| format!("reading {}: {e}", image.display()))?;

    let disks = usb_disks()?;
    let disk = match (&opts.device, disks.as_slice()) {
        (Some(path), _) => disks
            .iter()
            .find(|disk| disk.path == *path)
            .ok_or_else(|| {
                format!("{path} isn't a whole disk on USB that is removable or hotplug")
            })?,
        (None, [disk]) => disk,
        (None, []) => return Err("no removable USB disk found".into()),
        (None, _) => {
            let list: Vec<String> = disks.iter().map(describe).collect();
            return Err(format!(
                "more than one USB disk; pick one with --device:\n  {}",
                list.join("\n  ")
            ));
        }
    };
    if data.len() as u64 > disk.size {
        return Err(format!(
            "the image is {} bytes, larger than {}",
            data.len(),
            describe(disk)
        ));
    }

    println!("image: {} ({} bytes)", image.display(), data.len());
    println!("stick: {}", describe(disk));
    if !opts.yes
        && !confirm(&format!(
            "Erase everything on {} and write the image? [y/N] ",
            disk.path
        ))?
    {
        return Err("canceled; nothing was written".into());
    }

    unmount(&disk.path)?;
    // Ask for the password now, not in the middle of dd's progress output.
    exec(Command::new("sudo").arg("-v"))?;
    exec(Command::new("sudo").args([
        "dd",
        &format!("if={}", image.display()),
        &format!("of={}", disk.path),
        "bs=4M",
        "conv=fsync",
        "status=progress",
    ]))?;
    // This has to come before sgdisk -e, which rewrites the primary GPT header.
    verify(&disk.path, &data)?;
    exec(Command::new("sudo").args(["sgdisk", "-e", &disk.path]))?;
    let check = output(Command::new("sudo").args(["sgdisk", "-v", &disk.path]))?;
    if !check.contains("No problems found") {
        return Err(format!(
            "sgdisk -v found problems on {}:\n{check}",
            disk.path
        ));
    }
    // The desktop may have mounted the new partition after dd.
    unmount(&disk.path)?;
    exec(Command::new("udisksctl").args(["power-off", "-b", &disk.path]))?;
    println!("{} written, verified and powered off", disk.path);
    Ok(())
}

/// Whole disks on USB that are removable or hotplug: the only disks `flash`
/// writes.
fn usb_disks() -> Result<Vec<Disk>, String> {
    let list = lsblk(&[
        "-d",
        "-b",
        "-e7",
        "-o",
        "PATH,TYPE,TRAN,RM,HOTPLUG,SIZE,MODEL,SERIAL",
    ])?;
    let mut disks = Vec::new();
    for line in list.lines() {
        let fields = pairs(line);
        let field = |key| field(&fields, key);
        let removable = field("RM") == "1" || field("HOTPLUG") == "1";
        if field("TYPE") != "disk" || field("TRAN") != "usb" || !removable {
            continue;
        }
        disks.push(Disk {
            path: field("PATH").into(),
            size: field("SIZE")
                .parse()
                .map_err(|e| format!("lsblk size {:?}: {e}", field("SIZE")))?,
            model: field("MODEL").into(),
            serial: field("SERIAL").into(),
        });
    }
    Ok(disks)
}

fn describe(disk: &Disk) -> String {
    let gib = disk.size as f64 / f64::from(1u32 << 30);
    format!(
        "{}: {} {}, {gib:.1} GiB",
        disk.path, disk.model, disk.serial
    )
}

/// Unmounts every mounted partition on `device`.
fn unmount(device: &str) -> Result<(), String> {
    for line in lsblk(&["-n", "-o", "PATH,MOUNTPOINT", device])?.lines() {
        let fields = pairs(line);
        if !field(&fields, "MOUNTPOINT").is_empty() {
            exec(Command::new("udisksctl").args(["unmount", "-b", field(&fields, "PATH")]))?;
        }
    }
    Ok(())
}

/// Reads the image's length back from `device` and compares it with `image`.
fn verify(device: &str, image: &[u8]) -> Result<(), String> {
    // Dropping the device's cached pages makes the read come from the stick
    // and not from what dd left in memory. O_DIRECT would do the same, but
    // uutils dd, which Ubuntu ships as dd, fails with EINVAL reading a block
    // device that way.
    exec(Command::new("sudo").args(["blockdev", "--flushbufs", device]))?;
    let out = Command::new("sudo")
        .args([
            "dd",
            &format!("if={device}"),
            "bs=4M",
            "iflag=count_bytes",
            &format!("count={}", image.len()),
            "status=none",
        ])
        .stdin(Stdio::inherit())
        .output()
        .map_err(|e| format!("running sudo dd: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "reading {device} back failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    if let Some(offset) = out.stdout.iter().zip(image).position(|(a, b)| a != b) {
        return Err(format!("{device} differs from the image at byte {offset}"));
    }
    if out.stdout.len() < image.len() {
        return Err(format!(
            "read back {} of the image's {} bytes from {device}",
            out.stdout.len(),
            image.len()
        ));
    }
    println!("verified: {device} holds the image");
    Ok(())
}

fn lsblk(args: &[&str]) -> Result<String, String> {
    output(Command::new("lsblk").arg("-P").args(args))
}

/// Splits a line of `lsblk -P` output, `KEY="value" ...`, into pairs.
/// lsblk escapes quotes inside values as `\x22`, so the next quote always
/// ends a value.
fn pairs(line: &str) -> Vec<(&str, &str)> {
    let mut pairs = Vec::new();
    let mut rest = line;
    while let Some((key, after)) = rest.split_once("=\"") {
        let Some((value, after)) = after.split_once('"') else {
            break;
        };
        pairs.push((key.trim(), value));
        rest = after;
    }
    pairs
}

fn field<'a>(pairs: &[(&str, &'a str)], key: &str) -> &'a str {
    pairs.iter().find(|(k, _)| *k == key).map_or("", |(_, v)| v)
}

fn confirm(prompt: &str) -> Result<bool, String> {
    print!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|e| format!("writing the prompt: {e}"))?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|e| format!("reading the answer: {e}"))?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}

/// Runs `cmd` with the terminal attached, and fails if it does.
fn exec(cmd: &mut Command) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|e| format!("running {}: {e}", show(cmd)))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} failed: {status}", show(cmd)))
    }
}

/// Runs `cmd` and returns its standard output, failing if it fails.
fn output(cmd: &mut Command) -> Result<String, String> {
    let out = cmd
        .stdin(Stdio::inherit())
        .output()
        .map_err(|e| format!("running {}: {e}", show(cmd)))?;
    if !out.status.success() {
        return Err(format!(
            "{} failed: {}",
            show(cmd),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("output of {}: {e}", show(cmd)))
}

fn show(cmd: &Command) -> String {
    let mut words = vec![cmd.get_program().to_string_lossy().into_owned()];
    words.extend(cmd.get_args().map(|arg| arg.to_string_lossy().into_owned()));
    words.join(" ")
}

fn kvm_usable() -> bool {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/kvm")
        .is_ok()
}
