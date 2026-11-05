//! Running the tests.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

use crate::build::{self, Kernel};
use crate::cmd::{cargo, exec, on_host};
use crate::root;

/// Runs the host tests of every `lib/*` crate, then the kernel's tests in
/// QEMU.
pub fn test() -> Result<()> {
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
    exec(
        cargo()
            .env(
                "CARGO_TARGET_X86_64_UNKNOWN_NONE_RUNNER",
                format!("{xtask} qemu-test"),
            )
            // The same as build --tests, so that switching between the two
            // doesn't rebuild the kernel.
            .env("THISOS_BUILD", build::build_env(false, Kernel::Tests)?)
            .args(["test", "-p", "thisos-kernel"]),
    )
}

/// The manifests of the `lib/*` crates, which build and test on the host.
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
    Ok(manifests)
}
