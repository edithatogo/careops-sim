use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn files(path: &Path, found: &mut Vec<PathBuf>) {
    let metadata = fs::symlink_metadata(path).expect("model source metadata");
    assert!(
        !metadata.file_type().is_symlink(),
        "model source symlink is unsupported"
    );
    if metadata.is_dir() {
        for entry in fs::read_dir(path).expect("model source directory") {
            files(&entry.expect("model source entry").path(), found);
        }
    } else if path.extension().is_some_and(|e| e == "rs") {
        found.push(path.into());
    }
}
fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.parent().unwrap().parent().unwrap();
    let mut paths = Vec::new();
    for crate_name in ["careops-core", "careops-ed", "careops-ed-cli"] {
        let source = root.join("crates").join(crate_name).join("src");
        println!("cargo:rerun-if-changed={}", source.display());
        files(&source, &mut paths);
        paths.push(root.join("crates").join(crate_name).join("Cargo.toml"));
    }
    // Bind the actual consumed path runtime, also in source-only packages without Git.
    for name in ["types", "rng", "core", "des", "state", "abm"] {
        let runtime = root
            .join("libs/kairos/crates")
            .join(format!("kairo-ecs-{name}"));
        println!("cargo:rerun-if-changed={}", runtime.display());
        files(&runtime.join("src"), &mut paths);
        paths.push(runtime.join("Cargo.toml"));
    }
    paths.extend([
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        manifest.join("build.rs"),
    ]);
    paths.sort();
    let mut digest = Sha256::new();
    digest.update(b"careops.ed.model-build.v1\0");
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let name = path
            .strip_prefix(root)
            .unwrap()
            .to_str()
            .unwrap()
            .as_bytes();
        let content = fs::read(&path).expect("model input bytes");
        digest.update((name.len() as u64).to_le_bytes());
        digest.update(name);
        digest.update((content.len() as u64).to_le_bytes());
        digest.update(content);
    }
    let rustc = env::var_os("RUSTC").expect("actual Rust compiler");
    let version = Command::new(rustc)
        .arg("-vV")
        .output()
        .expect("Rust compiler readback");
    assert!(version.status.success(), "Rust compiler readback failed");
    digest.update((version.stdout.len() as u64).to_le_bytes());
    digest.update(version.stdout);
    for name in [
        "TARGET",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_CFG_TARGET_FEATURE",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        let value = env::var(name).unwrap_or_default();
        digest.update((name.len() as u64).to_le_bytes());
        digest.update(name.as_bytes());
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }
    let mut features: Vec<_> = env::vars()
        .filter(|(key, _)| key.starts_with("CARGO_FEATURE_"))
        .collect();
    features.sort();
    for (key, value) in features {
        digest.update((key.len() as u64).to_le_bytes());
        digest.update(key.as_bytes());
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }
    let value: String = digest
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    println!("cargo:rustc-env=CAREOPS_ED_MODEL_CONTRACT_SHA256={value}");
    let compiler = env::var("RUSTC").expect("actual compiler path");
    assert!(
        !compiler.contains(['\n', '\r']),
        "single-line compiler path"
    );
    let output = Command::new(&compiler)
        .arg("--version")
        .output()
        .expect("compiler identity");
    assert!(output.status.success(), "compiler identity command failed");
    let text = String::from_utf8(output.stdout).expect("compiler identity UTF8");
    let identity = text.trim();
    assert!(
        identity.starts_with("rustc ") && !identity.contains(['\n', '\r']),
        "single-line compiler identity"
    );
    println!("cargo:rustc-env=CAREOPS_ED_BUILD_RUSTC={identity}");
    println!("cargo:rustc-env=CAREOPS_ED_BUILD_COMPILER={compiler}");
}
