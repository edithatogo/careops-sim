//! Native ED patient-result parts. Domain columns are encoded by Kairos/Polars;
//! publication uses same-filesystem no-clobber staging and manifest-last commit.
//! One exclusive owner per run directory is required. This is not object storage.
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use careops_core::experiment::{ExperimentError, ExperimentErrorKind};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

use crate::capture::{
    BoundedPartWriter, CaptureBatch, CaptureLimits, CapturedReplication, PartStore, Result,
};
use crate::{PatientResult, PatientStatus};

pub const PATIENT_SCHEMA: &str = "careops.ed.patient-result.v2";
pub const CODEC: &str = "careops.ed.patient-result.v2/parquet-zstd/polars-96439685";
pub const DEFAULT_LIMITS: CaptureLimits = CaptureLimits {
    max_rows: 256,
    max_bytes: 4 * 1024 * 1024,
    max_patient_id_bytes: 4096,
};
fn invalid(e: impl std::fmt::Display) -> ExperimentError {
    ExperimentError::new(ExperimentErrorKind::MalformedConfig, e.to_string())
}
fn io(e: impl std::fmt::Display) -> ExperimentError {
    ExperimentError::new(ExperimentErrorKind::OutputWriteFailed, e.to_string())
}

/// Temporary runs live as long as their returned document (or clones). Serialized
/// documents need a persistent output directory; temporary paths are not portable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CaptureSource {
    pub directory: PathBuf,
    pub limits: CaptureLimits,
    #[serde(skip)]
    pub(crate) _owner: Option<Arc<tempfile::TempDir>>,
}
impl PartialEq for CaptureSource {
    fn eq(&self, other: &Self) -> bool {
        self.directory == other.directory && self.limits == other.limits
    }
}
impl CaptureSource {
    pub fn persistent(directory: PathBuf, limits: CaptureLimits) -> Self {
        Self {
            directory,
            limits,
            _owner: None,
        }
    }
    pub(crate) fn temporary() -> Result<Self> {
        let owner = Arc::new(tempfile::tempdir().map_err(io)?);
        Ok(Self {
            directory: owner.path().to_path_buf(),
            limits: DEFAULT_LIMITS,
            _owner: Some(owner),
        })
    }
}

pub struct ParquetPartStore {
    root: PathBuf,
    limits: CaptureLimits,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    codec: String,
    status: String,
    replications: Vec<CapturedReplication>,
}
impl ParquetPartStore {
    pub fn open(root: &Path, limits: CaptureLimits) -> Result<Self> {
        let m = fs::symlink_metadata(root).map_err(io)?;
        if !m.is_dir() || m.file_type().is_symlink() {
            return Err(invalid("capture root must be a real directory"));
        }
        if limits.max_rows == 0 || limits.max_bytes == 0 || limits.max_patient_id_bytes == 0 {
            return Err(invalid("capture limits must be positive"));
        }
        Ok(Self {
            root: root.canonicalize().map_err(io)?,
            limits,
        })
    }
    fn path(&self, name: &str) -> Result<PathBuf> {
        if !matches!(
            Path::new(name).components().collect::<Vec<_>>().as_slice(),
            [Component::Normal(_)]
        ) {
            return Err(invalid("invalid capture filename"));
        }
        let path = self.root.join(name);
        if let Ok(m) = fs::symlink_metadata(&path) {
            if !m.is_file() || m.file_type().is_symlink() {
                return Err(invalid("capture path must be a regular file"));
            }
        }
        Ok(path)
    }
    pub(crate) fn publish(
        &self,
        name: &str,
        encode: impl FnOnce(&mut File) -> Result<()>,
    ) -> Result<()> {
        let path = self.path(name)?;
        if path.exists() {
            return Err(invalid("capture collision"));
        }
        let mut staging = tempfile::NamedTempFile::new_in(&self.root).map_err(io)?;
        encode(staging.as_file_mut())?;
        staging.as_file_mut().flush().map_err(io)?;
        staging.as_file().sync_all().map_err(io)?;
        staging.persist_noclobber(path).map_err(io)?;
        File::open(&self.root)
            .and_then(|f| f.sync_all())
            .map_err(io)?;
        Ok(())
    }
    pub fn completed_replications(&self) -> Result<Vec<CapturedReplication>> {
        let bytes = self.read_part("manifest.json", 16 * 1024 * 1024)?;
        let m: Manifest = serde_json::from_slice(&bytes).map_err(invalid)?;
        if m.schema != PATIENT_SCHEMA || m.codec != CODEC || m.status != "complete" {
            return Err(invalid("unsupported or incomplete capture manifest"));
        }
        Ok(m.replications)
    }
    fn dataframe(&self, b: &CaptureBatch) -> Result<DataFrame> {
        if b.rows.is_empty()
            || b.rows.len() > self.limits.max_rows
            || b.rows
                .iter()
                .any(|r| r.patient_id.len() > self.limits.max_patient_id_bytes)
        {
            return Err(invalid("capture row bounds"));
        }
        let n = b.rows.len();
        let mut cols: Vec<Column> = vec![
            Series::new("time_unit".into(), vec![b.time_unit.as_str(); n]).into(),
            Series::new("schema".into(), vec![PATIENT_SCHEMA; n]).into(),
            Series::new("capture_schema".into(), vec![b.schema.as_str(); n]).into(),
            Series::new("run_id".into(), vec![b.run_id.as_str(); n]).into(),
            Series::new("input_sha256".into(), vec![b.input_sha256.as_str(); n]).into(),
            Series::new(
                "replication_index".into(),
                vec![b.replication_index as u64; n],
            )
            .into(),
            Series::new("seed".into(), vec![b.seed; n]).into(),
            Series::new(
                "sequence".into(),
                (b.first_sequence
                    ..b.first_sequence
                        .checked_add(n)
                        .ok_or_else(|| invalid("sequence overflow"))?)
                    .map(|v| v as u64)
                    .collect::<Vec<_>>(),
            )
            .into(),
            Series::new(
                "patient_id".into(),
                b.rows
                    .iter()
                    .map(|r| r.patient_id.as_str())
                    .collect::<Vec<_>>(),
            )
            .into(),
            Series::new(
                "status".into(),
                b.rows
                    .iter()
                    .map(|r| match r.status {
                        PatientStatus::Completed => "completed",
                        PatientStatus::Unfinished => "unfinished",
                    })
                    .collect::<Vec<_>>(),
            )
            .into(),
        ];
        for (name, values) in [
            (
                "arrival_tick",
                b.rows
                    .iter()
                    .map(|r| Some(r.arrival_tick))
                    .collect::<Vec<_>>(),
            ),
            ("start_tick", b.rows.iter().map(|r| r.start_tick).collect()),
            (
                "completion_tick",
                b.rows.iter().map(|r| r.completion_tick).collect(),
            ),
            (
                "work_ticks",
                b.rows.iter().map(|r| Some(r.work_ticks)).collect(),
            ),
            ("wait_ticks", b.rows.iter().map(|r| r.wait_ticks).collect()),
            (
                "elapsed_ticks",
                b.rows.iter().map(|r| r.elapsed_ticks).collect(),
            ),
        ] {
            cols.push(
                Series::new(
                    format!("{name}_hi").into(),
                    values.iter().map(|v| v.map(|_| 0u64)).collect::<Vec<_>>(),
                )
                .into(),
            );
            cols.push(Series::new(format!("{name}_lo").into(), values).into());
        }
        DataFrame::new(n, cols).map_err(invalid)
    }
}
impl PartStore for ParquetPartStore {
    fn codec_id(&self) -> &str {
        CODEC
    }
    fn encode_part(&self, b: &CaptureBatch, w: &mut dyn Write) -> Result<()> {
        kairo_ecs_arrow::polars_interop::write_parquet_zstd(&mut self.dataframe(b)?, w).map_err(io)
    }
    fn write_part(&self, name: &str, b: &CaptureBatch, max: usize) -> Result<()> {
        if self.is_complete()? {
            return Err(invalid("completed capture cannot be overwritten"));
        }
        self.publish(name, |f| {
            self.encode_part(
                b,
                &mut BoundedPartWriter::new(f, max.min(self.limits.max_bytes)),
            )
        })
    }
    fn read_part(&self, name: &str, max: usize) -> Result<Vec<u8>> {
        let mut f = File::open(self.path(name)?).map_err(io)?;
        if f.metadata().map_err(io)?.len() > max as u64 {
            return Err(invalid("capture file exceeds read bound"));
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut f)
            .take(max as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes.len() > max {
            return Err(invalid("capture file grew beyond read bound"));
        }
        Ok(bytes)
    }
    fn decode_part(&self, bytes: &[u8]) -> Result<CaptureBatch> {
        if bytes.len() > self.limits.max_bytes {
            return Err(invalid("encoded part exceeds cap"));
        }
        // Inspect declared row and decompressed sizes before allocating columns.
        // Decoder scratch still requires platform/RSS qualification, not a zero-copy claim.
        let mut reader = ParquetReader::new(Cursor::new(bytes));
        let md = reader.get_metadata().map_err(invalid)?;
        let decoded = md
            .row_groups
            .iter()
            .try_fold(0usize, |n, r| n.checked_add(r.total_byte_size()))
            .ok_or_else(|| invalid("decoded size overflow"))?;
        let cap = self
            .limits
            .max_rows
            .checked_mul(self.limits.max_patient_id_bytes.saturating_add(4096))
            .ok_or_else(|| invalid("decode bound overflow"))?;
        if md.num_rows == 0 || md.num_rows > self.limits.max_rows || decoded > cap {
            return Err(invalid("decoded part exceeds bounds"));
        }
        let df =
            kairo_ecs_arrow::polars_interop::read_parquet(Cursor::new(bytes)).map_err(invalid)?;
        if df.width() != 22 {
            return Err(invalid("unexpected patient schema columns"));
        }
        let string = |name: &str, i: usize| -> Result<String> {
            Ok(df
                .column(name)
                .map_err(invalid)?
                .str()
                .map_err(invalid)?
                .get(i)
                .ok_or_else(|| invalid("missing string"))?
                .to_owned())
        };
        let number = |name: &str, i: usize| -> Result<u64> {
            df.column(name)
                .map_err(invalid)?
                .u64()
                .map_err(invalid)?
                .get(i)
                .ok_or_else(|| invalid("missing integer"))
        };
        let tick = |name: &str, i: usize| -> Result<Option<u64>> {
            let hi = df
                .column(&format!("{name}_hi"))
                .map_err(invalid)?
                .u64()
                .map_err(invalid)?
                .get(i);
            let lo = df
                .column(&format!("{name}_lo"))
                .map_err(invalid)?
                .u64()
                .map_err(invalid)?
                .get(i);
            match (hi, lo) {
                (None, None) => Ok(None),
                (Some(0), Some(v)) => Ok(Some(v)),
                _ => Err(invalid("invalid or unsupported ED tick limbs")),
            }
        };
        let mut b = CaptureBatch {
            time_unit: string("time_unit", 0)?,
            schema: string("capture_schema", 0)?,
            run_id: string("run_id", 0)?,
            input_sha256: string("input_sha256", 0)?,
            replication_index: usize::try_from(number("replication_index", 0)?).map_err(invalid)?,
            seed: number("seed", 0)?,
            first_sequence: usize::try_from(number("sequence", 0)?).map_err(invalid)?,
            rows: Vec::new(),
        };
        for i in 0..df.height() {
            if string("time_unit", i)? != b.time_unit
                || string("schema", i)? != PATIENT_SCHEMA
                || string("capture_schema", i)? != b.schema
                || string("run_id", i)? != b.run_id
                || string("input_sha256", i)? != b.input_sha256
                || number("replication_index", i)? != b.replication_index as u64
                || number("seed", i)? != b.seed
                || number("sequence", i)?
                    != b.first_sequence
                        .checked_add(i)
                        .ok_or_else(|| invalid("sequence overflow"))? as u64
            {
                return Err(invalid("inconsistent part identity"));
            }
            let patient_id = string("patient_id", i)?;
            if patient_id.len() > self.limits.max_patient_id_bytes {
                return Err(invalid("patient ID bound"));
            }
            b.rows.push(PatientResult {
                patient_id,
                arrival_tick: tick("arrival_tick", i)?.ok_or_else(|| invalid("arrival null"))?,
                start_tick: tick("start_tick", i)?,
                completion_tick: tick("completion_tick", i)?,
                work_ticks: tick("work_ticks", i)?.ok_or_else(|| invalid("work null"))?,
                wait_ticks: tick("wait_ticks", i)?,
                elapsed_ticks: tick("elapsed_ticks", i)?,
                status: match string("status", i)?.as_str() {
                    "completed" => PatientStatus::Completed,
                    "unfinished" => PatientStatus::Unfinished,
                    _ => return Err(invalid("unknown patient status")),
                },
            });
        }
        Ok(b)
    }
    fn part_names(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(io)? {
            let entry = entry.map_err(io)?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| invalid("non-UTF8 capture name"))?;
            if name.ends_with(".part") {
                self.path(&name)?;
                names.push(name);
            }
        }
        names.sort();
        Ok(names)
    }
    fn is_complete(&self) -> Result<bool> {
        Ok(self.path("manifest.json")?.exists())
    }
    fn complete(&self, reps: &[CapturedReplication]) -> Result<()> {
        let bytes = serde_json::to_vec(&Manifest {
            schema: PATIENT_SCHEMA.into(),
            codec: CODEC.into(),
            status: "complete".into(),
            replications: reps.to_vec(),
        })
        .map_err(invalid)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(invalid("manifest exceeds read bound"));
        }
        self.publish("manifest.json", |f| f.write_all(&bytes).map_err(io))
    }
}
