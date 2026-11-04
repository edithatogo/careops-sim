//! Caller-declared synthetic fixture snapshots, distinct from captured runs.
//! No input/model/runtime/movement provenance is inferred from an online summary.
use crate::disposition::api_runner::{ExitCodeDef, ValidationError};
use crate::{PatientStatus, RunSummary};
use polars::prelude::*;

pub fn fixture_patient_table(summary: &RunSummary) -> Result<DataFrame, ValidationError> {
    let invalid = |message: &str| ValidationError::new(ExitCodeDef::MalformedConfig, message);
    let count = summary.patients.len();
    if summary.schema_version != 1
        || count > u32::MAX as usize
        || summary.scenario_id.trim().is_empty()
        || summary.scenario_id.len() > 256
        || summary.time_unit.trim().is_empty()
        || summary.time_unit.len() > 64
    {
        return Err(invalid(
            "unsupported fixture version, identity, unit or native row capacity",
        ));
    }
    let mut keys = std::collections::BTreeSet::new();
    for row in &summary.patients {
        if row.patient_id.is_empty() || !keys.insert(row.patient_id.as_str()) {
            return Err(invalid("empty or duplicate fixture patient identity"));
        }
    }
    let completed = summary
        .patients
        .iter()
        .filter(|p| p.status == PatientStatus::Completed)
        .count();
    let started = summary
        .patients
        .iter()
        .filter(|p| p.start_tick.is_some())
        .count();
    if summary.arrivals != count
        || summary.completed != completed
        || summary.started != started
        || summary.unfinished != count - completed
    {
        return Err(invalid("fixture summary counts differ from patient facts"));
    }
    let mut columns: Vec<Column> = vec![
        Series::new(
            "schema".into(),
            vec!["careops.ed.synthetic-fixture-patients.v2"; count],
        )
        .into(),
        Series::new(
            "source_kind".into(),
            vec!["synthetic_fixture_snapshot"; count],
        )
        .into(),
        Series::new(
            "fixture_id".into(),
            vec![summary.scenario_id.as_str(); count],
        )
        .into(),
        Series::new("seed".into(), vec![summary.seed; count]).into(),
        Series::new("time_unit".into(), vec![summary.time_unit.as_str(); count]).into(),
        Series::new("horizon_ticks".into(), vec![summary.horizon_ticks; count]).into(),
        Series::new(
            "patient_id".into(),
            summary
                .patients
                .iter()
                .map(|p| p.patient_id.as_str())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "status".into(),
            summary
                .patients
                .iter()
                .map(|p| match p.status {
                    PatientStatus::Completed => "completed",
                    PatientStatus::Unfinished => "unfinished",
                })
                .collect::<Vec<_>>(),
        )
        .into(),
    ];
    // Preserve original row order and raw u64 values; do not recompute durations,
    // infer completion, or turn a missing endpoint into a zero tick.
    for (name, values) in [
        (
            "arrival_tick",
            summary
                .patients
                .iter()
                .map(|p| Some(p.arrival_tick))
                .collect::<Vec<_>>(),
        ),
        (
            "start_tick",
            summary.patients.iter().map(|p| p.start_tick).collect(),
        ),
        (
            "completion_tick",
            summary.patients.iter().map(|p| p.completion_tick).collect(),
        ),
        (
            "work_ticks",
            summary
                .patients
                .iter()
                .map(|p| Some(p.work_ticks))
                .collect(),
        ),
        (
            "wait_ticks",
            summary.patients.iter().map(|p| p.wait_ticks).collect(),
        ),
        (
            "elapsed_ticks",
            summary.patients.iter().map(|p| p.elapsed_ticks).collect(),
        ),
    ] {
        columns.push(
            Series::new(
                format!("{name}_hi").into(),
                values
                    .iter()
                    .map(|value| value.map(|_| 0u64))
                    .collect::<Vec<_>>(),
            )
            .into(),
        );
        columns.push(Series::new(format!("{name}_lo").into(), values).into());
    }
    DataFrame::new(count, columns).map_err(|error| invalid(&error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PatientResult;
    fn summary() -> RunSummary {
        RunSummary {
            schema_version: 1,
            scenario_id: "fixture,\"é\n".into(),
            seed: u64::MAX,
            time_unit: "tick".into(),
            horizon_ticks: u64::MAX,
            arrivals: 3,
            started: 2,
            completed: 1,
            unfinished: 2,
            patients: vec![
                PatientResult {
                    patient_id: "inflight".into(),
                    arrival_tick: 0,
                    start_tick: Some(0),
                    completion_tick: None,
                    work_ticks: 1,
                    wait_ticks: Some(0),
                    elapsed_ticks: None,
                    status: PatientStatus::Unfinished,
                },
                PatientResult {
                    patient_id: "queued".into(),
                    arrival_tick: 1,
                    start_tick: None,
                    completion_tick: None,
                    work_ticks: 0,
                    wait_ticks: None,
                    elapsed_ticks: None,
                    status: PatientStatus::Unfinished,
                },
                PatientResult {
                    patient_id: "completed".into(),
                    arrival_tick: u64::MAX - 2,
                    start_tick: Some(u64::MAX - 1),
                    completion_tick: Some(u64::MAX),
                    work_ticks: u64::MAX,
                    wait_ticks: Some(1),
                    elapsed_ticks: Some(2),
                    status: PatientStatus::Completed,
                },
            ],
        }
    }
    #[test]
    fn snapshot_preserves_all_states_source_order_width_and_null_pairs() {
        let source = summary();
        let frame = fixture_patient_table(&source).unwrap();
        assert_eq!(frame.height(), 3);
        assert_eq!(frame.width(), 20);
        assert_eq!(
            (0..frame.height())
                .map(|row| frame
                    .column("patient_id")
                    .unwrap()
                    .str()
                    .unwrap()
                    .get(row)
                    .unwrap())
                .collect::<Vec<_>>(),
            ["inflight", "queued", "completed"]
        );
        assert_eq!(
            frame.column("schema").unwrap().str().unwrap().get(0),
            Some("careops.ed.synthetic-fixture-patients.v2")
        );
        assert_eq!(
            frame.column("source_kind").unwrap().str().unwrap().get(0),
            Some("synthetic_fixture_snapshot")
        );
        for row in 0..3 {
            assert_eq!(
                frame.column("fixture_id").unwrap().str().unwrap().get(row),
                Some(source.scenario_id.as_str())
            );
            assert_eq!(
                frame.column("seed").unwrap().u64().unwrap().get(row),
                Some(u64::MAX)
            );
            assert_eq!(
                frame
                    .column("horizon_ticks")
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(row),
                Some(u64::MAX)
            );
            let patient = &source.patients[row];
            for (stem, value) in [
                ("arrival_tick", Some(patient.arrival_tick)),
                ("start_tick", patient.start_tick),
                ("completion_tick", patient.completion_tick),
                ("work_ticks", Some(patient.work_ticks)),
                ("wait_ticks", patient.wait_ticks),
                ("elapsed_ticks", patient.elapsed_ticks),
            ] {
                assert_eq!(
                    frame
                        .column(&format!("{stem}_hi"))
                        .unwrap()
                        .u64()
                        .unwrap()
                        .get(row),
                    value.map(|_| 0)
                );
                assert_eq!(
                    frame
                        .column(&format!("{stem}_lo"))
                        .unwrap()
                        .u64()
                        .unwrap()
                        .get(row),
                    value
                );
            }
        }
        assert!(frame.column("input_sha256").is_err());
        assert!(frame.column("model_contract_sha256").is_err());
        assert!(frame.column("replication_index").is_err());
    }
    #[test]
    fn empty_snapshot_keeps_typed_columns_without_inventing_patients() {
        let mut source = summary();
        source.patients.clear();
        source.arrivals = 0;
        source.started = 0;
        source.completed = 0;
        source.unfinished = 0;
        let frame = fixture_patient_table(&source).unwrap();
        assert_eq!(frame.height(), 0);
        assert_eq!(frame.width(), 20);
        assert_eq!(
            frame.column("completion_tick_lo").unwrap().dtype(),
            &DataType::UInt64
        );
        assert_eq!(
            frame.column("fixture_id").unwrap().dtype(),
            &DataType::String
        );
    }
    #[test]
    fn unknown_version_duplicate_identity_and_inconsistent_counts_refuse() {
        let source = summary();
        for kind in 0..5 {
            let mut bad = source.clone();
            match kind {
                0 => bad.schema_version = 99,
                1 => bad.patients[1].patient_id = bad.patients[0].patient_id.clone(),
                2 => bad.patients[1].patient_id.clear(),
                3 => bad.arrivals = 4,
                _ => bad.started = 3,
            }
            assert_eq!(
                fixture_patient_table(&bad).unwrap_err().code,
                ExitCodeDef::MalformedConfig
            );
        }
    }
}
