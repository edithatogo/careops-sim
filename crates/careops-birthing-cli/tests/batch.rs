use careops_birthing_cli::{model, run_batch, validate_replication, BatchInput};

fn input() -> BatchInput {
    serde_json::from_str(include_str!("../fixtures/synthetic.json")).unwrap()
}
fn private_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    root
}

#[test]
fn actual_multiday_routes_callbacks_conservation_and_named_streams() {
    let input = input();
    let rows = model::simulate(&input.scenario, 41, 1, 0).unwrap();
    assert_eq!(rows.iter().filter(|r| r.kind == "birth").count(), 6);
    assert_eq!(rows.iter().filter(|r| r.kind == "conservation").count(), 6);
    assert!(rows.iter().any(|r| r.tick >= 2 * 86400));
    for kind in [
        "theatre_resource_unavailable",
        "target_breach",
        "destination_unavailable",
        "origin_reuse_blocked",
        "compensation_completed",
        "duplicate_ack_ignored",
        "progression",
    ] {
        assert!(
            rows.iter().any(|r| r.kind == kind),
            "missing actual event {kind}"
        );
    }
    for row in rows.iter().filter(|r| r.kind == "conservation") {
        assert_eq!(row.data["people"], row.data["open_episodes"]);
        assert_eq!(row.data["held_origin_claims"], 0);
    }
    assert_eq!(rows, model::simulate(&input.scenario, 41, 1, 0).unwrap());
    let unrelated = model::simulate(&input.scenario, 41, 1, 3).unwrap();
    let project = |rows: Vec<careops_birthing_cli::Row>| {
        rows.into_iter()
            .filter(|r| matches!(r.kind.as_str(), "birth" | "progression" | "service_draw"))
            .collect::<Vec<_>>()
    };
    assert_eq!(project(rows), project(unrelated));
}

#[test]
fn real_ipc_parquet_and_one_two_four_workers_retain_original_capture() {
    let input = input();
    let mut baseline = None;
    for workers in [1, 2, 4] {
        let root = private_root();
        let result = run_batch(root.path(), &input, workers, false, None).unwrap();
        let digests: Vec<_> = result
            .iter()
            .map(|r| (&r.index, &r.seed, &r.rows, &r.canonical_sha256))
            .map(|(a, b, c, d)| (*a, *b, *c, d.clone()))
            .collect();
        if let Some(ref base) = baseline {
            assert_eq!(base, &digests);
        } else {
            baseline = Some(digests);
        }
        for r in &result {
            let verified = validate_replication(
                &root.path().join(&input.batch_id),
                &r.directory,
                &input,
                r.index,
                r.seed,
            )
            .unwrap();
            assert_eq!(&verified.replication, r);
            let bytes = std::fs::read(
                root.path()
                    .join(&input.batch_id)
                    .join(&r.directory)
                    .join(&verified.ipc[0].file),
            )
            .unwrap();
            assert_eq!(&bytes[..4], &[255, 255, 255, 255]);
        }
        assert!(run_batch(root.path(), &input, workers, false, None).is_err());
    }
}

#[test]
fn interrupted_attempt_is_preserved_and_completed_replications_are_reused() {
    let mut input = input();
    input.replications = 3;
    let root = private_root();
    assert!(run_batch(root.path(), &input, 1, false, Some(1)).is_err());
    let batch = root.path().join(&input.batch_id);
    let interrupted = format!("{}-attempt1-rep1", input.batch_id);
    assert!(batch.join(&interrupted).exists());
    assert!(validate_replication(&batch, &interrupted, &input, 1, 0).is_err());
    assert!(!batch.join("batch_manifest.json").exists());
    let completed = format!("{}-attempt1-rep0", input.batch_id);
    let frozen = std::fs::read(batch.join(&completed).join("birthing_manifest.json")).unwrap();
    let results = run_batch(root.path(), &input, 2, true, None).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(
        results[1].directory,
        format!("{}-attempt2-rep1", input.batch_id)
    );
    assert_eq!(
        frozen,
        std::fs::read(batch.join(&completed).join("birthing_manifest.json")).unwrap()
    );
    let manifest = validate_replication(&batch, &completed, &input, 0, input.seed).unwrap();
    let file = batch.join(&completed).join(&manifest.ipc[0].file);
    std::fs::write(file, b"tampered").unwrap();
    assert!(validate_replication(&batch, &completed, &input, 0, input.seed).is_err());
    assert!(run_batch(root.path(), &input, 1, true, None).is_err());
}

#[test]
fn unknown_versions_units_and_unbounded_profiles_fail_before_output() {
    for mutate in [0, 1, 2, 3, 4, 5, 6] {
        let root = private_root();
        let mut input = input();
        match mutate {
            0 => input.scenario.schema_version = 2,
            1 => input.scenario.time_unit = "minute".into(),
            2 => input.replications = 33,
            3 => input.scenario.provenance = "clinical".into(),
            4 => {
                input.scenario.preparation_seconds = 0;
                input.scenario.incision_to_birth_seconds = 0;
            }
            5 => input.scenario.cleaning_seconds = input.scenario.travel_min_seconds + 1,
            _ => {
                input.scenario.preparation_seconds = u64::MAX;
                input.scenario.incision_to_birth_seconds = u64::MAX;
            }
        };
        assert!(run_batch(root.path(), &input, 1, false, None).is_err());
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
}
