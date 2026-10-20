use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");
    let compiler = env::var("RUSTC").expect("Cargo must identify the build compiler");
    assert!(
        !compiler.contains('\n') && !compiler.contains('\r'),
        "compiler command must be a single line"
    );
    let output = Command::new(&compiler)
        .arg("--version")
        .output()
        .expect("observe the actual build compiler");
    assert!(output.status.success(), "compiler identity command failed");
    let text = String::from_utf8(output.stdout).expect("compiler identity must be UTF-8");
    let identity = text.trim();
    assert!(
        identity.starts_with("rustc ") && !identity.contains('\n') && !identity.contains('\r'),
        "compiler identity must be one rustc version line"
    );
    println!("cargo:rustc-env=CAREOPS_ED_BUILD_RUSTC={identity}");
    println!("cargo:rustc-env=CAREOPS_ED_BUILD_COMPILER={compiler}");
}
