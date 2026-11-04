//! Valid-source coverage through the production native optimization pipeline.
use crate::util::{InterpMode, Run, RunMode, TestProgram};
use dag_in_context::StructuralTypeMode;
use std::path::PathBuf;

#[test]
fn structural_type_host_preserves_valid_pointer_call_in_all_modes() {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/encoding_combined/structural-types.bril");
    for mode in [
        StructuralTypeMode::Off,
        StructuralTypeMode::Manual,
        StructuralTypeMode::Combined,
        StructuralTypeMode::AnnotationsOff,
        StructuralTypeMode::QueryOff,
    ] {
        let mut run = Run::new(
            TestProgram::BrilFile(input.clone()).read_program(),
            RunMode::Optimize,
        );
        run.interp = InterpMode::Interp;
        run.eggcc_config.structural_types = mode;
        assert!(run.eggcc_config.non_weakly_linear);
        assert_eq!(run.eggcc_config.get_schedule_list().len(), 3);
        assert_eq!(run.eggcc_config.stop_after_n_passes, i64::MAX);
        let result = run
            .run()
            .unwrap_or_else(|error| panic!("{mode:?}: {error}"));
        assert_eq!(result.original_interpreted.as_deref(), Some("7\n"));
        assert_eq!(result.result_interpreted.as_deref(), Some("7\n"));
    }
}
