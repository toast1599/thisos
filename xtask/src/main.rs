mod banned;
mod build;
mod ci;
mod cmd;
mod flash;
mod font;
mod lsblk;
mod qemu;
mod test;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use crate::build::Kernel;

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
        #[command(flatten)]
        tests: Tests,
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
        #[command(flatten)]
        tests: Tests,
    },
    /// Run the host tests, then the kernel's tests in QEMU.
    Test,
    /// Run locally everything CI runs.
    Ci,
    /// Rasterize the console font into lib/console/src/font.
    Font,
    /// Boot a kernel test ELF in QEMU; cargo's runner for `xtask test`.
    #[command(hide = true)]
    QemuTest { elf: PathBuf },
}

/// The tests an image boots in place of the kernel.
#[derive(Args)]
struct Tests {
    /// Build the kernel library's tests, which don't fault, into
    /// target/thisos-tests.img instead.
    #[arg(long)]
    tests: bool,
    /// Build the faulting test in kernel/tests/<NAME>.rs into
    /// target/thisos-test-<NAME>.img instead.
    #[arg(long, value_name = "NAME", conflicts_with = "tests")]
    test: Option<String>,
}

impl Tests {
    fn kernel(&self) -> Kernel<'_> {
        match (&self.test, self.tests) {
            (Some(name), _) => Kernel::Test(name),
            (None, true) => Kernel::Tests,
            (None, false) => Kernel::Main,
        }
    }
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Command::Build { release, tests } => build::build(release, tests.kernel()).map(|image| {
            println!("{}\n{}", image.path.display(), image.id);
        }),
        Command::Run { release, gdb } => qemu::run(release, gdb),
        Command::Flash {
            release,
            device,
            tests,
        } => flash::flash(release, tests.kernel(), device.as_deref()),
        Command::Test => test::test(),
        Command::Ci => ci::ci(),
        Command::Font => font::font(),
        Command::QemuTest { elf } => qemu::test(&elf),
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
