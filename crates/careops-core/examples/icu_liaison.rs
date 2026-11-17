//! Run the ICU liaison illustrative economic scenario.
use std::env;
use std::fs;
use std::process::ExitCode;

use careops_core::icu_liaison::{simulate, Scenario};
use sha2::{Digest, Sha256};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: cargo run --locked -p careops-core --example icu_liaison -- <scenario.json>");
        return ExitCode::FAILURE;
    };
    if args.next().is_some() {
        eprintln!("exactly one scenario path is required");
        return ExitCode::FAILURE;
    }
    let source = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("cannot read scenario: {error}");
            return ExitCode::FAILURE;
        }
    };
    let scenario: Scenario = match serde_json::from_str(&source) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid scenario JSON: {error}");
            return ExitCode::FAILURE;
        }
    };
    let result = match simulate(&scenario) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid simulation parameters: {error}");
            return ExitCode::FAILURE;
        }
    };
    let input_sha256 = format!("{:x}", Sha256::digest(source.as_bytes()));
    let output = serde_json::json!({
        "input_sha256": input_sha256,
        "output": result
    });
    match serde_json::to_string_pretty(&output) {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("cannot serialise output: {error}");
            ExitCode::FAILURE
        }
    }
}
