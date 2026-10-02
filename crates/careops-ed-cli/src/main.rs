use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use careops_ed::{parse_scenario_json, run_scenario, RunSummary};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
struct RunManifest {
    schema_version: u32,
    scenario_id: String,
    input_file: String,
    input_sha256: String,
    seed: u64,
    time_unit: String,
    horizon_ticks: u64,
}

#[derive(Serialize)]
struct RunDocument {
    manifest: RunManifest,
    summary: RunSummary,
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("careops-ed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(usage_error());
    };
    if command == "--help" || command == "-h" {
        println!("{}", usage());
        return Ok(());
    }
    if command != "run" {
        return Err(usage_error());
    }
    let input_file = args.next().ok_or_else(usage_error)?;
    if args.next().is_some() {
        return Err(usage_error());
    }

    let input_path = PathBuf::from(&input_file);
    let bytes = fs::read(&input_path)?;
    let source = std::str::from_utf8(&bytes)?;
    let scenario = parse_scenario_json(source)?;
    let summary = run_scenario(&scenario)?;
    let input_sha256 = format!("{:x}", Sha256::digest(&bytes));
    let manifest = RunManifest {
        schema_version: 1,
        scenario_id: summary.scenario_id.clone(),
        input_file,
        input_sha256,
        seed: summary.seed,
        time_unit: summary.time_unit.clone(),
        horizon_ticks: summary.horizon_ticks,
    };
    let document = RunDocument { manifest, summary };
    println!("{}", serde_json::to_string_pretty(&document)?);
    Ok(())
}

fn usage() -> &'static str {
    "Usage: careops-ed run <scenario.json>\n       careops-ed --help\n\nRuns a validated synthetic E0 scenario and writes a JSON result to stdout."
}

fn usage_error() -> Box<dyn Error> {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, usage()).into()
}
