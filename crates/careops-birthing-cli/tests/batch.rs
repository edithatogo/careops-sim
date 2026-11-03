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
fn resume_rejects_a_different_model_build_before_reusing_or_running_any_replication() {
    let mut input = input();
    input.replications = 3;
    let root = private_root();
    assert!(run_batch(root.path(), &input, 1, false, Some(1)).is_err());
    let batch = root.path().join(&input.batch_id);
    let path = batch.join("checkpoint.json");
    let mut checkpoint: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    checkpoint["model_contract_sha256"] = "0".repeat(64).into();
    let changed = serde_json::to_vec(&checkpoint).unwrap();
    std::fs::write(&path, &changed).unwrap();
    let before = std::fs::read_dir(&batch).unwrap().count();
    assert!(run_batch(root.path(), &input, 1, true, None).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), changed);
    assert_eq!(std::fs::read_dir(&batch).unwrap().count(), before);
}

#[test]
fn operative_capture_retains_all_metrics_and_their_policy_foreign_keys() {
    let input = input();
    for target in [0, 3, 30] {
        let mut scenario = input.scenario.clone();
        scenario.target_birth_seconds = target;
        let rows = model::simulate(&scenario, 41, 1, 0).unwrap();
        let cases: Vec<_> = rows.iter().filter(|r| r.kind == "theatre_case").collect();
        assert_eq!(cases.len(), scenario.days as usize);
        for case in cases {
            for (name, timing) in [
                ("transfer", "departed"),
                ("incision", "incision"),
                ("birth", "birth"),
            ] {
                let metric = &case.data["metrics"][name];
                assert_eq!(
                    metric["elapsed"].as_u64().unwrap(),
                    case.data["timings"][timing].as_u64().unwrap()
                        - case.data["decision_tick"].as_u64().unwrap()
                );
                let assessment = &case.data["assessments"][0];
                assert_eq!(metric["assessment_tick"], assessment["observed_at_tick"]);
                assert_eq!(
                    metric["scheme_version"],
                    assessment["policy"]["scheme_version"]
                );
                assert_eq!(metric["policy_revision"], assessment["policy"]["revision"]);
                assert_eq!(assessment["urgency"]["effective_date"], "2026-11-03");
            }
            let birth = &case.data["metrics"]["birth"];
            assert_eq!(
                birth["breached"].as_bool().unwrap(),
                birth["elapsed"].as_u64().unwrap() > target
            );
            for callback in [
                "Accepted",
                "TransferDeparted",
                "TransferArrived",
                "ServiceStarted",
                "ProcedureStarted",
                "Birth",
                "ServiceCompleted",
                "RoomReady",
            ] {
                assert!(case.data["callbacks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c["callback"] == callback));
            }
        }
    }
}

#[test]
fn completed_formats_cannot_change_capture_status_even_with_updated_file_hashes() {
    use kairo_ecs_arrow::{polars_interop as io, EventStatus};
    let mut input = input();
    input.replications = 1;
    let root = private_root();
    let result = run_batch(root.path(), &input, 1, false, None).unwrap();
    let batch = root.path().join(&input.batch_id);
    let directory = batch.join(&result[0].directory);
    let domain_path = directory.join("birthing_manifest.json");
    let native_path = directory.join("run_manifest.json");
    let mut domain: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&domain_path).unwrap()).unwrap();
    let mut native: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&native_path).unwrap()).unwrap();
    let ipc_path = directory.join(domain["ipc"][0]["file"].as_str().unwrap());
    let parquet_path = directory.join(native["parts"][0]["file_name"].as_str().unwrap());
    let frame =
        io::read_ipc_stream(std::io::Cursor::new(std::fs::read(&ipc_path).unwrap())).unwrap();
    let mut rows = io::dataframe_to_records(&frame).unwrap();
    rows[0].status = EventStatus::Error;
    let mut changed = io::records_to_dataframe(&rows).unwrap();
    let mut ipc = Vec::new();
    io::write_ipc_stream(&mut changed, &mut ipc).unwrap();
    let mut parquet = Vec::new();
    io::write_parquet_zstd(&mut changed, &mut parquet).unwrap();
    domain["ipc"][0]["sha256"] = careops_birthing_cli::hash(&ipc).into();
    native["parts"][0]["file_hash"] = format!("{:016x}", io::calculate_checksum(&parquet)).into();
    std::fs::write(ipc_path, ipc).unwrap();
    std::fs::write(parquet_path, parquet).unwrap();
    std::fs::write(domain_path, serde_json::to_vec(&domain).unwrap()).unwrap();
    std::fs::write(native_path, serde_json::to_vec(&native).unwrap()).unwrap();
    assert!(validate_replication(&batch, &result[0].directory, &input, 0, input.seed).is_err());
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

#[test]
fn uncertainty_is_explicit_and_cannot_be_silently_changed_or_omitted() {
    let mut input = input();
    input.replications = 1;
    input.scenario.days = 1;
    let root = private_root();
    let results = run_batch(root.path(), &input, 1, false, None).unwrap();
    let batch = root.path().join(&input.batch_id);
    let result = &results[0];
    let path = batch.join(&result.directory).join("birthing_manifest.json");
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(original["schema_version"], 2);
    assert_eq!(
        original["uncertainty"],
        serde_json::json!({
            "schema": "careops.birthing.observation-uncertainty.v1",
            "method": "not_estimated",
            "reason": "case_observations_no_declared_sampling_estimand",
            "observation_unit": "synthetic_case",
            "experiment_unit": "independent_replication",
            "metrics": ["delay_seconds", "origin_occupied_seconds", "target_breach"]
        })
    );
    for key in [
        "schema",
        "method",
        "reason",
        "observation_unit",
        "experiment_unit",
        "metrics",
    ] {
        let mut changed = original.clone();
        changed["uncertainty"].as_object_mut().unwrap().remove(key);
        std::fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(
            validate_replication(&batch, &result.directory, &input, 0, result.seed).is_err(),
            "missing {key}"
        );
        let mut changed = original.clone();
        changed["uncertainty"][key] = if key == "metrics" {
            serde_json::json!(["delay_seconds"])
        } else {
            "invented".into()
        };
        std::fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(
            validate_replication(&batch, &result.directory, &input, 0, result.seed).is_err(),
            "altered {key}"
        );
    }
    for mode in 0..3 {
        let mut changed = original.clone();
        match mode {
            0 => {
                changed.as_object_mut().unwrap().remove("uncertainty");
            }
            1 => {
                changed["uncertainty"]["population_ci"] = serde_json::json!([0, 0]);
            }
            _ => {
                changed["schema_version"] = 1.into();
            }
        }
        std::fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(validate_replication(&batch, &result.directory, &input, 0, result.seed).is_err());
    }
    std::fs::write(&path, serde_json::to_vec(&original).unwrap()).unwrap();
    validate_replication(&batch, &result.directory, &input, 0, result.seed).unwrap();
}
