//! Building the kernel and its disk image.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::cmd::{cargo, exec, output};
use crate::root;

/// The file in the image's FAT partition that holds the build ID.
pub const BUILD_FILE: &str = "thisos-build.txt";

/// A disk image `build` wrote.
pub struct Image {
    pub path: PathBuf,
    /// `ThisOS <version> <commit> <profile> <time>`, as the kernel logs it.
    pub id: String,
}

/// Builds the kernel and writes the UEFI disk image.
pub fn build(release: bool) -> Result<Image> {
    let profile = if release { "release" } else { "debug" };
    let pkgid = output(cargo().args(["pkgid", "-p", "thisos-kernel"]))?;
    let version = pkgid
        .trim()
        .rsplit(['@', '#'])
        .next()
        .with_context(|| format!("no version in cargo pkgid output {pkgid:?}"))?;
    let commit = output(
        Command::new("git")
            .current_dir(root())
            .args(["describe", "--always", "--dirty"]),
    )?;
    let time = output(Command::new("date").args(["-u", "+%Y-%m-%dT%H:%MZ"]))?;
    // The kernel logs "ThisOS <version> " followed by this.
    let build = format!("{} {profile} {}", commit.trim(), time.trim());
    let id = format!("ThisOS {version} {build}");

    let mut cargo_build = cargo();
    cargo_build
        .env("THISOS_BUILD", &build)
        .args(["build", "-p", "thisos-kernel"]);
    if release {
        cargo_build.arg("--release");
    }
    exec(&mut cargo_build)?;

    let path = image_path();
    bootloader::DiskImageBuilder::new(kernel_path(profile))
        .set_file_contents(BUILD_FILE.into(), format!("{id}\n").into_bytes())
        .create_uefi_image(&path)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(Image { path, id })
}

fn kernel_path(profile: &str) -> PathBuf {
    root()
        .join("target/x86_64-unknown-none")
        .join(profile)
        .join("thisos-kernel")
}

fn image_path() -> PathBuf {
    root().join("target/thisos.img")
}
