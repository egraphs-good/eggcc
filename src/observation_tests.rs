//! Actual native host consumers, registered after the durable prospective CLI detection.
use std::{fs, path::PathBuf};

use crate::util::{InterpMode, Run, RunMode, TestProgram};
use dag_in_context::ObservationMode;

#[test]
fn observation_host_preserves_add_sub_boundary_results_through_native_extraction() {
    for file in ["add-max.bril", "sub-min.bril"] {
        for mode in [ObservationMode::Manual, ObservationMode::Injective] {
            let evidence = tempfile::tempdir().unwrap();
            let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/encoding_payoff/arithmetic")
                .join(file);
            let mut run = Run::new(
                TestProgram::BrilFile(input).read_program(),
                RunMode::Optimize,
            );
            run.interp = InterpMode::Interp;
            run.configure_observations(mode, Some(evidence.path().to_owned()))
                .unwrap();
            let result = run.run().unwrap();
            assert_eq!(result.original_interpreted.as_deref(), Some("true\n"));
            assert_eq!(result.result_interpreted.as_deref(), Some("true\n"));
            let children: Vec<_> = fs::read_dir(evidence.path())
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect();
            assert_eq!(children.len(), 1);
            let child = &children[0];
            let index: serde_json::Value =
                serde_json::from_slice(&fs::read(child.join("index.json")).unwrap()).unwrap();
            assert_eq!(index["status"], "complete");
            assert_eq!(index["passes"].as_object().unwrap().len(), 3);
            assert!(index["passes"]
                .as_object()
                .unwrap()
                .values()
                .all(|pass| pass["stage"] == "native-round-trip-complete"));
            let audit: serde_json::Value =
                serde_json::from_slice(&fs::read(child.join("pass-00/audit.json")).unwrap())
                    .unwrap();
            assert_eq!(audit["accepted"], true);
            assert!(child.join("pass-00/serialization-entry.json").exists());
            let committed: serde_json::Value = serde_json::from_slice(
                &fs::read(child.join("pass-00/committed-before-audit.json")).unwrap(),
            )
            .unwrap();
            assert!(committed["conflicts"].as_array().unwrap().is_empty());
        }
    }
}

#[test]
fn observation_host_keeps_passing_and_skipped_outputs_through_three_native_passes() {
    for (file, expected) in [
        ("add-one.bril", "false\n"),
        ("sub-negative-one.bril", "false\n"),
        ("add-zero-skipped.bril", ""),
        ("sub-zero-skipped.bril", ""),
    ] {
        for mode in [ObservationMode::Manual, ObservationMode::Injective] {
            let evidence = tempfile::tempdir().unwrap();
            let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/encoding_payoff/arithmetic")
                .join(file);
            let mut run = Run::new(
                TestProgram::BrilFile(input).read_program(),
                RunMode::Optimize,
            );
            run.interp = InterpMode::Interp;
            run.configure_observations(mode, Some(evidence.path().to_owned()))
                .unwrap();
            let result = run.run().unwrap();
            assert_eq!(result.original_interpreted.as_deref(), Some(expected));
            assert_eq!(result.result_interpreted.as_deref(), Some(expected));
            let locator = result.observation_evidence.unwrap();
            let index: serde_json::Value =
                serde_json::from_slice(&fs::read(locator.index).unwrap()).unwrap();
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
}

#[test]
fn observation_configuration_requires_its_supported_native_context() {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/encoding_payoff/arithmetic/add-one.bril");
    let mut run = Run::new(
        TestProgram::BrilFile(input).read_program(),
        RunMode::Optimize,
    );
    run.eggcc_config.use_context = false;
    assert!(run
        .configure_observations(ObservationMode::Manual, Some("unused".into()))
        .is_err());
    run.eggcc_config.use_context = true;
    run.eggcc_config.ablate = Some("interval-analysis".into());
    assert!(run
        .configure_observations(ObservationMode::Injective, Some("unused".into()))
        .is_err());
    run.eggcc_config.ablate = None;
    run.test_type = RunMode::Parse;
    assert!(run
        .configure_observations(ObservationMode::Manual, Some("unused".into()))
        .is_err());
    run.test_type = RunMode::Optimize;
    assert!(run
        .configure_observations(ObservationMode::Manual, None)
        .is_err());
    assert!(run
        .configure_observations(ObservationMode::Off, None)
        .is_ok());
    assert!(run.eggcc_config.observations.is_none());
}

#[test]
fn arithmetic_bounds_preserve_wrapping_add_output() {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/encoding_payoff/arithmetic/add-max.bril");
    let mut run = Run::new(
        TestProgram::BrilFile(input).read_program(),
        RunMode::Optimize,
    );
    run.interp = InterpMode::Interp;
    let result = run.run().unwrap();
    assert_eq!(result.original_interpreted.as_deref(), Some("true\n"));
    assert_eq!(result.result_interpreted.as_deref(), Some("true\n"));
}

#[test]
fn arithmetic_bounds_preserve_wrapping_sub_output() {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/encoding_payoff/arithmetic/sub-min.bril");
    let mut run = Run::new(
        TestProgram::BrilFile(input).read_program(),
        RunMode::Optimize,
    );
    run.interp = InterpMode::Interp;
    let result = run.run().unwrap();
    assert_eq!(result.original_interpreted.as_deref(), Some("true\n"));
    assert_eq!(result.result_interpreted.as_deref(), Some("true\n"));
}
