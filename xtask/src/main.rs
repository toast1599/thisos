mod build;
mod ci;
mod cmd;
mod flash;
mod font;
mod lsblk;
mod qemu;
mod test;

use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// Build, run and test tasks for ThisOS.
#[derive(Parser)]
#[command(bin_name = "cargo xtask")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the kernel and write target/thisos.img, then print its path and
    /// build ID.
    Build {
        /// Use the release profile.
        #[arg(long)]
        release: bool,
    },
    /// Build, then boot the image in QEMU.
    Run {
        /// Use the release profile.
        #[arg(long)]
        release: bool,
        /// Start QEMU's gdb stub on TCP port 1234, with the CPU paused.
        #[arg(long)]
        gdb: bool,
    },
    /// Build, then write the image to a USB stick, verify it and power the
    /// stick off.
    Flash {
        /// Use the release profile.
        #[arg(long)]
        release: bool,
        /// The stick to write, when more than one USB disk is plugged in.
        #[arg(long, value_name = "PATH")]
        device: Option<String>,
        /// Don't ask before writing.
        #[arg(long)]
        yes: bool,
    },
    /// Run the host tests.
    Test,
    /// Run locally everything CI runs.
    Ci,
    /// Rasterize the console font into lib/console/src/font.
    Font,
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Command::Build { release } => build::build(release).map(|image| {
            println!("{}\n{}", image.path.display(), image.id);
        }),
        Command::Run { release, gdb } => qemu::run(release, gdb),
        Command::Flash {
            release,
            device,
            yes,
        } => flash::flash(release, device.as_deref(), yes),
        Command::Test => test::test(),
        Command::Ci => ci::ci(),
        Command::Font => font::font(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repository root")
}
