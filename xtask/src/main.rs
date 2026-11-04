use std::process::ExitCode;

const USAGE: &str = "usage: cargo xtask <command>

No commands yet.";

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        None | Some("help" | "-h" | "--help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(cmd) => {
            eprintln!("unknown command: {cmd}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}
