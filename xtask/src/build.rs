//! Building the kernel and its disk image.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;

use crate::cmd::{cargo, exec, output};
use crate::{limine, root};

/// The file in the image's FAT partition that holds the build ID.
pub const BUILD_FILE: &str = "thisos-build.txt";

/// The architecture a kernel is built for.
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Arch {
    #[value(name = "x86_64")]
    X86_64,
    Aarch64,
}

impl Arch {
    /// Its name on xtask's command line.
    pub fn name(self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
        }
    }

    /// The Rust target the kernel builds for.
    pub fn target(self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64-unknown-none",
            Arch::Aarch64 => "aarch64-unknown-none-softfloat",
        }
    }

    /// Writes the disk image that boots the kernel ELF at `elf` to `path`,
    /// with the file at `ramdisk` as the loader's ramdisk, and `id` in
    /// [`BUILD_FILE`], when they are `Some`.
    pub fn write_image(
        self,
        elf: &Path,
        ramdisk: Option<&Path>,
        id: Option<&str>,
        path: &Path,
    ) -> Result<()> {
        match self {
            Arch::X86_64 => {
                let mut builder = bootloader::DiskImageBuilder::new(elf.to_path_buf());
                if let Some(ramdisk) = ramdisk {
                    builder.set_ramdisk(ramdisk.to_path_buf());
                }
                if let Some(id) = id {
                    builder.set_file_contents(BUILD_FILE.into(), format!("{id}\n").into_bytes());
                }
                builder
                    .create_uefi_image(path)
                    .with_context(|| format!("writing {}", path.display()))
            }
            Arch::Aarch64 => {
                ensure!(ramdisk.is_none(), "aarch64 images carry no ramdisk");
                limine::write_image(elf, id, path)
            }
        }
    }

    /// Builds what the kernel's images carry as the loader's ramdisk and
    /// returns its path: init's ELF file on x86_64, and nothing on aarch64,
    /// which has no user mode yet (ADR 0006).
    pub fn ramdisk(self, release: bool) -> Result<Option<PathBuf>> {
        if self == Arch::Aarch64 {
            return Ok(None);
        }
        let mut cargo_build = cargo();
        cargo_build.args(["build", "-p", "thisos-init", "--target", self.target()]);
        if release {
            cargo_build.arg("--release");
        }
        exec(&mut cargo_build)?;
        let profile = if release { "release" } else { "debug" };
        Ok(Some(
            root()
                .join("target")
                .join(self.target())
                .join(profile)
                .join("thisos-init"),
        ))
    }

    /// What follows `thisos` in the names of its files in `target/`:
    /// nothing for x86_64, which came first.
    pub fn suffix(self) -> &'static str {
        match self {
            Arch::X86_64 => "",
            Arch::Aarch64 => "-aarch64",
        }
    }
}

/// What an image boots.
#[derive(Clone, Copy)]
pub enum Kernel {
    /// The kernel itself.
    Main,
    /// The kernel library's test binary, which holds every kernel test.
    Tests,
}

/// A disk image `build` wrote.
pub struct Image {
    pub path: PathBuf,
    /// `ThisOS <version> <commit> <profile> <time>`, as the kernel logs it.
    pub id: String,
}

/// Builds `kernel` and writes its UEFI disk image.
pub fn build(arch: Arch, release: bool, kernel: Kernel) -> Result<Image> {
    let profile = if release { "release" } else { "debug" };
    let pkgid = output(cargo().args(["pkgid", "-p", "thisos-kernel"]))?;
    let version = pkgid
        .trim()
        .rsplit(['@', '#'])
        .next()
        .with_context(|| format!("no version in cargo pkgid output {pkgid:?}"))?;
    let build = build_env(release, kernel)?;
    let id = format!("ThisOS {version} {build}");

    let (elf, path) = match kernel {
        Kernel::Main => {
            let mut cargo_build = cargo();
            cargo_build.env("THISOS_BUILD", &build).args([
                "build",
                "-p",
                "thisos-kernel",
                "--target",
                arch.target(),
            ]);
            if release {
                cargo_build.arg("--release");
            }
            exec(&mut cargo_build)?;
            (
                root()
                    .join("target")
                    .join(arch.target())
                    .join(profile)
                    .join("thisos-kernel"),
                root().join(format!("target/thisos{}.img", arch.suffix())),
            )
        }
        Kernel::Tests => (
            test_elf(arch, release, &build)?,
            root().join(format!("target/thisos{}-tests.img", arch.suffix())),
        ),
    };

    let ramdisk = arch.ramdisk(release)?;
    arch.write_image(&elf, ramdisk.as_deref(), Some(&id), &path)?;
    Ok(Image { path, id })
}

/// `THISOS_BUILD` for a kernel build: `<commit> <profile>[ tests] <time>`.
/// The kernel logs "ThisOS <version> " followed by this.
pub fn build_env(release: bool, kernel: Kernel) -> Result<String> {
    let profile = if release { "release" } else { "debug" };
    let commit = output(
        Command::new("git")
            .current_dir(root())
            .args(["describe", "--always", "--dirty"]),
    )?;
    let time = output(Command::new("date").args(["-u", "+%Y-%m-%dT%H:%MZ"]))?;
    let kind = match kernel {
        Kernel::Main => "",
        Kernel::Tests => " tests",
    };
    Ok(format!("{} {profile}{kind} {}", commit.trim(), time.trim()))
}

/// Builds the kernel library's test binary and returns its path. Cargo
/// names it only in its JSON messages.
fn test_elf(arch: Arch, release: bool, build: &str) -> Result<PathBuf> {
    let mut cargo_test = cargo();
    cargo_test
        .env("THISOS_BUILD", build)
        .args(["test", "-p", "thisos-kernel", "--no-run", "--lib"])
        .args(["--target", arch.target()])
        .arg("--message-format=json-render-diagnostics")
        .stderr(Stdio::inherit());
    if release {
        cargo_test.arg("--release");
    }
    let messages = output(&mut cargo_test)?;
    let mut elfs = Vec::new();
    for line in messages.lines() {
        let message: Value =
            serde_json::from_str(line).with_context(|| format!("cargo message {line:?}"))?;
        if message["reason"] == "compiler-artifact"
            && message["profile"]["test"] == true
            && let Some(elf) = message["executable"].as_str()
        {
            elfs.push(PathBuf::from(elf));
        }
    }
    match <[PathBuf; 1]>::try_from(elfs) {
        Ok([elf]) => Ok(elf),
        Err(elfs) => bail!("expected one kernel test binary, cargo built {elfs:?}"),
    }
}
