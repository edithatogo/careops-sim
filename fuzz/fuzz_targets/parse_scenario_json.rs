#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../scenario_oracle.rs"]
mod scenario_oracle;

fuzz_target!(|data: &[u8]| scenario_oracle::check(data));
