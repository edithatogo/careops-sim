#![forbid(unsafe_code)]

use std::env;
use std::process::ExitCode;

use careops_ed_cli::execute;

fn main() -> ExitCode {
    // Single-threaded process entry, before any worker or Polars pool exists.
    // Library callers must set this environment variable before initializing Polars.
    match env::var("POLARS_MAX_THREADS") {
        Ok(value) if value.parse::<usize>().is_ok_and(|n| n > 0) => {}
        Ok(_) => {
            eprintln!("POLARS_MAX_THREADS must be a positive integer");
            return ExitCode::from(2);
        }
        Err(env::VarError::NotPresent) => env::set_var("POLARS_MAX_THREADS", "1"),
        Err(_) => {
            eprintln!("POLARS_MAX_THREADS must be valid UTF-8");
            return ExitCode::from(2);
        }
    }
    let args: Vec<String> = env::args().skip(1).collect();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    match execute(&args, &mut stdout, None) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(error.code as u8)
        }
    }
}
