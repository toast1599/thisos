//! Running the tests.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

use crate::build::{self, Arch, Kernel};
use crate::cmd::{cargo, exec, on_host};
use crate::root;

/// Runs the host tests of every `lib/*` crate and `thisos-rt`, then the
/// kernel's tests for `arch` in QEMU.
pub fn test(arch: Arch) -> Result<()> {
    for manifest in libs()? {
        exec(on_host(
            cargo().arg("test").arg("--manifest-path").arg(&manifest),
        ))?;
    }

    // Set here rather than in .cargo/config.toml: cargo starts the runner in
    // kernel/, where the xtask alias's relative manifest path doesn't resolve.
    let xtask = std::env::current_exe().context("finding the xtask executable")?;
    let xtask = xtask.to_str().context("the xtask path isn't UTF-8")?;
    ensure!(
        !xtask.contains(char::is_whitespace),
        "cargo splits its runner on whitespace, and the xtask path {xtask:?} has some"
    );
    let runner = format!(
        "CARGO_TARGET_{}_RUNNER",
        arch.target().to_uppercase().replace('-', "_")
    );
    let mut command = format!("{xtask} qemu-test --arch {}", arch.name());
    // Inside the repository, as the xtask executable is, so the check above
    // covers it too.
    if let Some(ramdisk) = arch.ramdisk(false)? {
        let ramdisk = ramdisk.to_str().context("the ramdisk path isn't UTF-8")?;
        command = format!("{command} --ramdisk {ramdisk}");
    }
    exec(
        cargo()
            .env(runner, command)
            // The same kind as build --tests, so a QEMU run logs `tests` like a
            // laptop test image. It doesn't spare a rebuild when switching
            // between the two: the ID has the time to the minute, and the
            // kernel's option_env! recompiles whenever it changes.
            .env("THISOS_BUILD", build::build_env(false, Kernel::Tests)?)
            .args(["test", "-p", "thisos-kernel", "--target", arch.target()]),
    )
}

/// The manifests of the crates that build and test on the host: the
/// `lib/*` crates, and `thisos-rt`, whose tests leave out its entry point,
/// panic handler and syscalls.
pub fn libs() -> Result<Vec<PathBuf>> {
    let dir = root().join("lib");
    let mut manifests = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let manifest = entry
            .with_context(|| format!("reading {}", dir.display()))?
            .path()
            .join("Cargo.toml");
        if manifest.is_file() {
            manifests.push(manifest);
        }
    }
    manifests.sort();
    manifests.push(root().join("user/rt/Cargo.toml"));
    Ok(manifests)
}
