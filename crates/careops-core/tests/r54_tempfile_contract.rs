//! Apache-2.0. Observable publication contracts; no staging-name implementation oracle.
use careops_core::experiment::{atomic_write_file, ExperimentErrorKind};
use std::{fs, process::Command, sync::Arc, thread};

#[test]
fn overwrite_nested_and_error_paths() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("nested/result.bin");
    atomic_write_file(&path, b"first").unwrap();
    atomic_write_file(&path, b"second").unwrap();
    assert_eq!(fs::read(path).unwrap(), b"second");
    let directory = root.path().join("directory");
    fs::create_dir(&directory).unwrap();
    assert_eq!(
        atomic_write_file(&directory, b"x").unwrap_err().kind,
        ExperimentErrorKind::OutputWriteFailed
    );
    assert!(directory.is_dir());
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
    let file = root.path().join("parent-file");
    fs::write(&file, b"original").unwrap();
    assert_eq!(
        atomic_write_file(&file.join("child"), b"x")
            .unwrap_err()
            .kind,
        ExperimentErrorKind::OutputWriteFailed
    );
    assert_eq!(fs::read(file).unwrap(), b"original");
}

#[test]
fn unrelated_pid_staging_is_preserved() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("result");
    let unrelated = destination.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&unrelated, b"unrelated diagnostic").unwrap();
    atomic_write_file(&destination, b"published").unwrap();
    assert_eq!(fs::read(destination).unwrap(), b"published");
    assert_eq!(fs::read(unrelated).unwrap(), b"unrelated diagnostic");
}

#[test]
fn simultaneous_writers_publish_one_complete_payload() {
    let root = tempfile::tempdir().unwrap();
    let destination = Arc::new(root.path().join("shared"));
    let barrier = Arc::new(std::sync::Barrier::new(8));
    let handles: Vec<_> = (0..8u8)
        .map(|id| {
            let destination = destination.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                atomic_write_file(&destination, &vec![id; 65536]).unwrap();
            })
        })
        .collect();
    let results: Vec<_> = handles.into_iter().map(|handle| handle.join()).collect();
    for result in results {
        result.unwrap();
    }
    let bytes = fs::read(&*destination).unwrap();
    assert_eq!(bytes.len(), 65536);
    assert!(bytes[0] < 8 && bytes.iter().all(|v| *v == bytes[0]));
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
}

#[test]
fn directory_owners_isolate_cleanup_and_retention() {
    let root = tempfile::tempdir().unwrap();
    let owners: Vec<_> = (0..8)
        .map(|_| tempfile::tempdir_in(root.path()).unwrap())
        .collect();
    let paths: std::collections::HashSet<_> = owners.iter().map(|d| d.path().to_owned()).collect();
    assert_eq!(paths.len(), 8);
    drop(owners);
    assert!(paths.iter().all(|p| !p.exists()));
    let failed: Result<(), std::io::Error> = (|| {
        let d = tempfile::tempdir_in(root.path())?;
        fs::write(d.path().join("data"), b"x")?;
        Err(std::io::Error::other("intentional"))
    })();
    assert!(failed.is_err());
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    let retained = tempfile::tempdir_in(root.path()).unwrap().keep();
    assert!(retained.is_dir());
    fs::remove_dir(retained).unwrap();
}

#[cfg(unix)]
#[test]
fn actual_write_fault_preserves_destination_and_cleans_scratch() {
    if let Some(root) = std::env::var_os("CAREOPS_R54_FAULT_CHILD") {
        let root = std::path::PathBuf::from(root);
        let destination = root.join("output");
        let error = atomic_write_file(&destination, &vec![42; 1024 * 1024]).unwrap_err();
        assert_eq!(error.kind, ExperimentErrorKind::OutputWriteFailed);
        assert_eq!(fs::read(destination).unwrap(), b"original");
        let entries: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                (e.file_name(), e.metadata().unwrap().len())
            })
            .collect();
        assert_eq!(entries.len(), 1, "failed write left scratch: {entries:?}");
        return;
    }
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("output"), b"original").unwrap();
    let result = Command::new("/bin/sh")
        .args(["-c", "ulimit -f 1; trap '' XFSZ; exec \"$@\"", "r54"])
        .arg(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "actual_write_fault_preserves_destination_and_cleans_scratch",
            "--nocapture",
        ])
        .env("CAREOPS_R54_FAULT_CHILD", root.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "isolated partial-write contract failed: {} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[cfg(unix)]
#[test]
fn creation_permissions_follow_child_umask_and_basename_works() {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(mode) = std::env::var("CAREOPS_R54_MODE_CHILD") {
        atomic_write_file(std::path::Path::new("bare"), b"bytes").unwrap();
        assert_eq!(
            fs::metadata("bare").unwrap().permissions().mode() & 0o777,
            u32::from_str_radix(&mode, 8).unwrap()
        );
        assert_eq!(fs::read("bare").unwrap(), b"bytes");
        return;
    }
    for (mask, expected) in [("022", "644"), ("077", "600")] {
        let root = tempfile::tempdir().unwrap();
        let status = Command::new("/bin/sh")
            .args(["-c", &format!("umask {mask}; exec \"$@\""), "r54"])
            .arg(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "creation_permissions_follow_child_umask_and_basename_works",
                "--nocapture",
            ])
            .current_dir(root.path())
            .env("CAREOPS_R54_MODE_CHILD", expected)
            .status()
            .unwrap();
        assert!(status.success());
    }
}
