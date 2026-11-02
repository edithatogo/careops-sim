fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn run() -> careops_birthing_cli::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 3 || args.len() > 4 {
        return Err(
            "usage: careops-birthing INPUT.json PRIVATE_OUTPUT_ROOT WORKERS [--resume]".into(),
        );
    }
    let resume = args.get(3).map(|v| v.as_str());
    if resume.is_some_and(|v| v != "--resume") {
        return Err("unknown option".into());
    }
    let input = serde_json::from_slice(&careops_birthing_cli::read_bounded(
        std::path::Path::new(&args[0]),
        65536,
    )?)?;
    let results = careops_birthing_cli::run_batch(
        std::path::Path::new(&args[1]),
        &input,
        args[2].parse()?,
        resume.is_some(),
        None,
    )?;
    println!("{}", serde_json::to_string(&results)?);
    Ok(())
}
