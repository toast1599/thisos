//! Running the tests.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cmd::{cargo, exec, on_host};
use crate::root;

/// Runs the host tests of every `lib/*` crate.
pub fn test() -> Result<()> {
    for manifest in libs()? {
        exec(on_host(
            cargo().arg("test").arg("--manifest-path").arg(&manifest),
        ))?;
    }
    Ok(())
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
