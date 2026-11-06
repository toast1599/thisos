//! Building the kernel and its disk image.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::cmd::{cargo, exec, output};
use crate::root;

/// The file in the image's FAT partition that holds the build ID.
pub const BUILD_FILE: &str = "thisos-build.txt";

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
pub fn build(release: bool, kernel: Kernel) -> Result<Image> {
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
            cargo_build
                .env("THISOS_BUILD", &build)
                .args(["build", "-p", "thisos-kernel"]);
            if release {
                cargo_build.arg("--release");
            }
            exec(&mut cargo_build)?;
            (
                root()
                    .join("target/x86_64-unknown-none")
                    .join(profile)
                    .join("thisos-kernel"),
                root().join("target/thisos.img"),
            )
        }
        Kernel::Tests => (
            test_elf(release, &build)?,
            root().join("target/thisos-tests.img"),
        ),
    };

    bootloader::DiskImageBuilder::new(elf)
        .set_file_contents(BUILD_FILE.into(), format!("{id}\n").into_bytes())
        .create_uefi_image(&path)
        .with_context(|| format!("writing {}", path.display()))?;
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
fn test_elf(release: bool, build: &str) -> Result<PathBuf> {
    let mut cargo_test = cargo();
    cargo_test
        .env("THISOS_BUILD", build)
        .args(["test", "-p", "thisos-kernel", "--no-run", "--lib"])
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
