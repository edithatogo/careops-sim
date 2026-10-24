//! Atomic file system operations for simulation outputs and checkpoints.

use std::fs;
use std::path::Path;

use super::error::{ExperimentError, ExperimentErrorKind};

/// Atomically writes content to path via temporary staging file and atomic rename.
pub fn atomic_write_file(path: &Path, content: &[u8]) -> Result<(), ExperimentError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                ExperimentError::new(
                    ExperimentErrorKind::OutputWriteFailed,
                    format!("failed to create directory '{}': {e}", parent.display()),
                )
            })?;
        }
    }

    let staging_path = path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&staging_path, content).map_err(|e| {
        ExperimentError::new(
            ExperimentErrorKind::OutputWriteFailed,
            format!(
                "failed to write staging file '{}': {e}",
                staging_path.display()
            ),
        )
    })?;

    fs::rename(&staging_path, path).map_err(|e| {
        let _ = fs::remove_file(&staging_path);
        ExperimentError::new(
            ExperimentErrorKind::OutputWriteFailed,
            format!("failed to atomically commit file '{}': {e}", path.display()),
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_write_file() {
        let temp_dir = std::env::temp_dir().join(format!("test_atomic_{}", std::process::id()));
        let file_path = temp_dir.join("test_file.txt");
        let content = b"hello world";

        atomic_write_file(&file_path, content).expect("atomic write should succeed");
        assert_eq!(fs::read(&file_path).unwrap(), content);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
