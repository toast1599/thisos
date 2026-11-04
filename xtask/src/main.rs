use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

const USAGE: &str = "usage: cargo xtask <command> [options]

commands:
  build [--release]          build the kernel and write target/thisos.img
  run [--release] [--gdb]    build, then boot the image in QEMU
                             (--gdb: gdb stub on tcp 1234, start paused)
  font                       rasterize the console font into lib/console/src/font";

const OVMF: Source = Source::EDK2_STABLE202605_R1;

/// The console's glyph sizes in pixels per em, smallest first.
const FONT_SIZES: [u32; 4] = [12, 16, 24, 32];

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

fn kvm_usable() -> bool {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/kvm")
        .is_ok()
}
