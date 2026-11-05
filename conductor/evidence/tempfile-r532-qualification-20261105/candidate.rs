//! Apache-2.0. Private qualification candidate, not parent runtime adoption.
use std::{fs, io::Write, path::Path};
use super::error::{ExperimentError, ExperimentErrorKind};
pub fn atomic_write_file(path: &Path, content: &[u8]) -> Result<(), ExperimentError> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| ExperimentError::new(
        ExperimentErrorKind::OutputWriteFailed,
        format!("failed to create directory '{}': {e}", parent.display()),
    ))?;
    let mut builder = tempfile::Builder::new();
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(fs::Permissions::from_mode(0o666));
    }
    let mut staged = builder.tempfile_in(parent).map_err(|e| ExperimentError::new(
        ExperimentErrorKind::OutputWriteFailed,
        format!("failed to create staging file beside '{}': {e}", path.display()),
    ))?;
    staged.write_all(content).map_err(|e| ExperimentError::new(
        ExperimentErrorKind::OutputWriteFailed,
        format!("failed to write staging file '{}': {e}", staged.path().display()),
    ))?;
    staged.persist(path).map_err(|e| ExperimentError::new(
        ExperimentErrorKind::OutputWriteFailed,
        format!("failed to atomically commit file '{}': {e}", path.display()),
    ))?;
    Ok(())
}
