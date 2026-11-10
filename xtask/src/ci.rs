//! Running locally everything CI runs.

use std::process::{Command, Stdio};

use anyhow::{Result, ensure};

use crate::build::{self, Arch, Kernel};
use crate::cmd::{cargo, exec, on_host, output};
use crate::{banned, test};

/// Runs the checks in CI's order and stops at the first that fails.
pub fn ci() -> Result<()> {
    // Checked first, so a missing tool fails before minutes of other checks.
    ensure!(
        cargo()
            .args(["deny", "--version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success()),
        "cargo deny isn't installed; install it with `cargo install cargo-deny --locked`"
    );

    banned::check()?;
    exec(cargo().args(["fmt", "--all", "--check"]))?;
    exec(cargo().args([
        "fmt",
        "--manifest-path",
        "xtask/Cargo.toml",
        "--all",
        "--check",
    ]))?;

    // The aarch64 arch crate's assembly doesn't build for x86_64; it gets
    // its own passes below.
    exec(cargo().args([
        "clippy",
        "--workspace",
        "--exclude",
        "thisos-arch-aarch64",
        "--",
        "-D",
        "warnings",
    ]))?;
    // Not --workspace --all-targets: that builds thisos-console's tests,
    // which need std, for x86_64-unknown-none.
    exec(cargo().args([
        "clippy",
        "-p",
        "thisos-kernel",
        "--tests",
        "--",
        "-D",
        "warnings",
    ]))?;
    let aarch64 = Arch::Aarch64.target();
    exec(cargo().args([
        "clippy",
        "-p",
        "thisos-kernel",
        "-p",
        "thisos-arch-aarch64",
        "--target",
        aarch64,
        "--",
        "-D",
        "warnings",
    ]))?;
    exec(cargo().args([
        "clippy",
        "-p",
        "thisos-kernel",
        "--tests",
        "--target",
        aarch64,
        "--",
        "-D",
        "warnings",
    ]))?;
    let libs = test::libs()?;
    for manifest in &libs {
        exec(
            on_host(cargo().arg("clippy").arg("--manifest-path").arg(manifest))
                .args(["--tests", "--", "-D", "warnings"]),
        )?;
    }
    exec(
        on_host(cargo().args(["clippy", "--manifest-path", "xtask/Cargo.toml"])).args([
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ]),
    )?;
    exec(on_host(cargo().args([
        "test",
        "--manifest-path",
        "xtask/Cargo.toml",
    ])))?;

    test::test(Arch::X86_64)?;
    // cargo miri takes host-tuple as a target name and fails to build a
    // sysroot for it, so it gets the host's actual tuple.
    let host = output(Command::new("rustc").args(["--print", "host-tuple"]))?;
    for manifest in &libs {
        exec(
            cargo()
                .args(["miri", "test", "--target", host.trim(), "--manifest-path"])
                .arg(manifest),
        )?;
    }

    for manifest in ["Cargo.toml", "xtask/Cargo.toml"] {
        exec(cargo().args([
            "deny",
            "--manifest-path",
            manifest,
            "--config",
            "deny.toml",
            "check",
        ]))?;
    }

    build::build(Arch::X86_64, false, Kernel::Main)?;
    // Clippy stops before codegen, so only this assembles and links the
    // aarch64 kernel.
    build::build(Arch::Aarch64, false, Kernel::Main)?;
    Ok(())
}
