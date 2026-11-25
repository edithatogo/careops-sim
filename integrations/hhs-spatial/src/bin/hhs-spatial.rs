use careops_hhs_spatial::{Assumptions, load};
use serde_json::json;
use std::{error::Error, path::Path};
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 && args.len() != 4 {
        return Err("usage: hhs-spatial PACKAGE EXPECTED_MANIFEST_SHA256 [ROUTE_JSON]".into());
    }
    let package = load(Path::new(&args[1]), &args[2])?;
    if args.len() == 4 {
        let route: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[3])?)?;
        let assumptions: Assumptions = serde_json::from_value(route["assumptions"].clone())?;
        let path: Vec<String> = serde_json::from_value(route["path"].clone())?;
        let seconds = package.travel_seconds(&path, &assumptions)?;
        println!(
            "{}",
            json!({"manifest_sha256":package.manifest_sha256,"provisional":true,
            "spatial_acceptance":false,"units_verified":false,"assumptions":assumptions,
            "path":path,"travel_seconds":seconds})
        );
    } else {
        println!("{}", serde_json::to_string(&package)?);
    }
    Ok(())
}
