//! Running external commands and asking the user.

use std::io::{self, Write};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, ensure};

use crate::root;

/// A cargo command in the repository root, using the cargo that runs xtask.
pub fn cargo() -> Command {
    let mut cmd = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
    cmd.current_dir(root());
    cmd
}

/// Builds `cmd` for the host. The root's `.cargo/config.toml` makes
/// x86_64-unknown-none the default target, also for xtask's own workspace.
pub fn on_host(cmd: &mut Command) -> &mut Command {
    cmd.args(["--target", "host-tuple"])
}

/// Runs `cmd` with the terminal attached, and fails if it does.
pub fn exec(cmd: &mut Command) -> Result<()> {
    let status = cmd
        .status()
        .with_context(|| format!("running {}", show(cmd)))?;
    ensure!(status.success(), "{} failed: {status}", show(cmd));
    Ok(())
}

/// Runs `cmd` and returns its standard output as text, failing if it fails.
pub fn output(cmd: &mut Command) -> Result<String> {
    let bytes = output_bytes(cmd)?;
    String::from_utf8(bytes).with_context(|| format!("output of {}", show(cmd)))
}

/// Runs `cmd` and returns its standard output, failing if it fails. Standard
/// input stays on the terminal, so that sudo can ask for a password.
pub fn output_bytes(cmd: &mut Command) -> Result<Vec<u8>> {
    let out = cmd
        .stdin(Stdio::inherit())
        .output()
        .with_context(|| format!("running {}", show(cmd)))?;
    ensure!(
        out.status.success(),
        "{} failed: {}",
        show(cmd),
        String::from_utf8_lossy(&out.stderr).trim()
    );
    Ok(out.stdout)
}

fn show(cmd: &Command) -> String {
    let mut words = vec![cmd.get_program().to_string_lossy().into_owned()];
    words.extend(cmd.get_args().map(|arg| arg.to_string_lossy().into_owned()));
    words.join(" ")
}

/// Asks a yes/no question; anything but yes is no.
pub fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt}");
    io::stdout().flush().context("writing the prompt")?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .context("reading the answer")?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}
