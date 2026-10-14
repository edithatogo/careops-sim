//! Headless runner example for CLI integration and walkthrough tests.

use std::env;
use std::process::ExitCode;

use careops_ed::disposition::api_runner::execute_cli_args;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let out = execute_cli_args(&args);
    if !out.stdout.is_empty() {
        print!("{}", out.stdout);
        if !out.stdout.ends_with('\n') {
            println!();
        }
    }
    if !out.stderr.is_empty() {
        eprint!("{}", out.stderr);
        if !out.stderr.ends_with('\n') {
            eprintln!();
        }
    }
    ExitCode::from(out.exit_code as u8)
}
