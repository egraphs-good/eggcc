//! Full host pipeline checks, separate from the synthetic collector invariants.
use std::{fs, path::PathBuf};

use crate::util::{InterpMode, Run, RunMode, TestProgram};
use dag_in_context::ObservationMode;

#[test]
fn observation_host_rejects_actual_branch_merge_before_native_extraction() {
    for mode in [ObservationMode::Manual, ObservationMode::Injective] {
        let evidence = tempfile::tempdir().unwrap();
        let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/encoding_impact/arithmetic/mul-minimal-max.bril");
        let mut run = Run::new(
            TestProgram::BrilFile(input).read_program(),
            RunMode::Optimize,
        );
        run.interp = InterpMode::Interp;
        // Retain the production three-pass/non-weakly-linear/default greedy config.
        run.configure_observations(mode, Some(evidence.path().to_owned()), false)
            .unwrap();
        let error = match run.run() {
            Ok(_) => panic!("the real invalid merge was accepted"),
            Err(error) => error,
        };
        let children: Vec<_> = fs::read_dir(evidence.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(children.len(), 1);
        let child = &children[0];
        let index: serde_json::Value =
            serde_json::from_slice(&fs::read(child.join("index.json")).unwrap()).unwrap();
        assert_eq!(index["status"], "rejected");
        assert_eq!(index["stage"], "observation-audit-rejected");
        let audit_path = child.join("pass-00/audit.json");
        let audit: serde_json::Value =
            serde_json::from_slice(&fs::read(&audit_path).unwrap()).unwrap();
        assert_eq!(audit["accepted"], false, "{error}");
        assert_eq!(audit["stage"], "pre-serialization-observation-audit");
        assert!(!child.join("pass-00/serialization-entry.json").exists());
        let committed: serde_json::Value = serde_json::from_slice(
            &fs::read(child.join("pass-00/committed-before-audit.json")).unwrap(),
        )
        .unwrap();
        assert!(!committed["conflicts"].as_array().unwrap().is_empty());
    }
}

#[test]
fn observation_host_valid_control_finishes_all_three_native_round_trips() {
    for mode in [ObservationMode::Manual, ObservationMode::Injective] {
        let evidence = tempfile::tempdir().unwrap();
        let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/encoding_impact/arithmetic/mul-minimal-one.bril");
        let mut run = Run::new(
            TestProgram::BrilFile(input).read_program(),
            RunMode::Optimize,
        );
        run.interp = InterpMode::Interp;
        run.configure_observations(mode, Some(evidence.path().to_owned()), false)
            .unwrap();
        let result = run.run().unwrap();
        assert_eq!(result.original_interpreted.as_deref(), Some("false\n"));
        assert_eq!(result.result_interpreted.as_deref(), Some("false\n"));
        let locator = result.observation_evidence.as_ref().unwrap();
        assert_eq!(
            locator.directory.parent().unwrap(),
            evidence.path().canonicalize().unwrap()
        );
        assert_eq!(locator.index, locator.directory.join("index.json"));
        let index: serde_json::Value =
            serde_json::from_slice(&fs::read(&locator.index).unwrap()).unwrap();
        assert_eq!(index["status"], "complete");
        assert_eq!(index["passes"].as_object().unwrap().len(), 3);
        for pass in 0..3 {
            let dir = locator.directory.join(format!("pass-{pass:02}"));
            let audit: serde_json::Value =
                serde_json::from_slice(&fs::read(dir.join("audit.json")).unwrap()).unwrap();
            assert_eq!(audit["accepted"], true);
            assert!(dir.join("serialization-entry.json").exists());
        }
    }
}

#[test]
fn observation_cli_configuration_enforces_context_mode_and_ablation_boundary() {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/encoding_impact/arithmetic/mul-minimal-one.bril");
    let mut run = Run::new(
        TestProgram::BrilFile(input).read_program(),
        RunMode::Optimize,
    );
    run.eggcc_config.use_context = false;
    assert!(run
        .configure_observations(ObservationMode::Manual, Some("unused".into()), false)
        .is_err());
    run.eggcc_config.use_context = true;
    run.eggcc_config.ablate = Some("interval-analysis".into());
    assert!(run
        .configure_observations(ObservationMode::Injective, Some("unused".into()), false)
        .is_err());
    run.eggcc_config.ablate = None;
    assert!(run
        .configure_observations(ObservationMode::Manual, Some("unused".into()), true)
        .is_err());
    assert!(run
        .configure_observations(ObservationMode::Manual, None, false)
        .is_err());
    assert!(run
        .configure_observations(ObservationMode::Off, None, false)
        .is_ok());
    assert!(run.eggcc_config.observations.is_none());
}

#[test]
fn observation_reusing_one_api_configuration_allocates_once_per_call_and_preserves_it() {
    let root = tempfile::tempdir().unwrap();
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/encoding_impact/arithmetic/mul-minimal-one.bril");
    let mut run = Run::new(
        TestProgram::BrilFile(input).read_program(),
        RunMode::Optimize,
    );
    run.interp = InterpMode::Interp;
    run.configure_observations(
        ObservationMode::Injective,
        Some(root.path().to_owned()),
        false,
    )
    .unwrap();
    let first = run.run().unwrap();
    let second = run.run().unwrap();
    let a = first.observation_evidence.unwrap();
    let b = second.observation_evidence.unwrap();
    assert_ne!(a.directory, b.directory);
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
    assert_eq!(
        run.eggcc_config.observations.as_ref().unwrap().evidence_dir,
        root.path()
    );
    for locator in [a, b] {
        let index: serde_json::Value =
            serde_json::from_slice(&fs::read(locator.index).unwrap()).unwrap();
        assert_eq!(index["status"], "complete");
        assert_eq!(index["passes"].as_object().unwrap().len(), 3);
        for pass in 0..3 {
            assert_eq!(
                index["passes"][format!("{pass:02}")]["stage"],
                "native-round-trip-complete"
            );
            assert!(locator
                .directory
                .join(format!("pass-{pass:02}/audit.json"))
                .is_file());
        }
    }
}

#[test]
fn observation_off_omits_locator_and_does_not_allocate_evidence() {
    use dag_in_context::{interpreter::Value, schema::Constant, ObservationConfig};
    let root = tempfile::tempdir().unwrap();
    let never_created = root.path().join("off-must-not-create");
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/encoding_impact/arithmetic/mul-minimal-one.bril");
    let mut run = Run::new(
        TestProgram::BrilFile(input).read_program(),
        RunMode::Optimize,
    );
    run.interp = InterpMode::Interp;
    run.eggcc_config.observations = Some(ObservationConfig {
        mode: ObservationMode::Off,
        root_inputs: vec![Value::Tuple(vec![
            Value::Const(Constant::Int(1)),
            Value::StateV,
        ])],
        evidence_dir: never_created.clone(),
        prepass_probe: false,
    });
    let result = run.run().unwrap();
    assert!(result.observation_evidence.is_none());
    assert!(!never_created.exists());
    assert!(serde_json::to_value(result)
        .unwrap()
        .get("observation_evidence")
        .is_none());
}
