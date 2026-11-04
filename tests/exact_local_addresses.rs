use dag_in_context::ExactLocalAddressMode;
use eggcc::util::{InterpMode, Run, RunMode, TestProgram};
use std::path::PathBuf;

#[test]
fn exact_local_addresses_full_native_round_trips() {
    for (case, expected) in [
        ("fresh-allocations", "17\n"),
        ("same-pointer", "99\n"),
        ("zero-offset", "99\n"),
    ] {
        let input =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/encoding/{case}.bril"));
        let mut off_bril = None;
        for mode in [
            ExactLocalAddressMode::Off,
            ExactLocalAddressMode::Keys,
            ExactLocalAddressMode::Query,
            ExactLocalAddressMode::Manual,
        ] {
            let mut run = Run::new(
                TestProgram::BrilFile(input.clone()).read_program(),
                RunMode::Optimize,
            );
            run.interp = InterpMode::Interp;
            run.eggcc_config.non_weakly_linear = true;
            run.eggcc_config.exact_local_addresses = mode;
            let result = run.run().unwrap();
            assert_eq!(
                result.original_interpreted.as_deref(),
                Some(expected),
                "{case}: {mode:?}"
            );
            assert_eq!(
                result.result_interpreted.as_deref(),
                Some(expected),
                "{case}: {mode:?}"
            );
            let bril = &result.visualizations[0].result;
            if mode == ExactLocalAddressMode::Off {
                off_bril = Some(bril.clone());
            } else if mode == ExactLocalAddressMode::Keys {
                assert_eq!(
                    Some(bril),
                    off_bril.as_ref(),
                    "keys changed output for {case}"
                );
            }
            if case == "fresh-allocations" {
                assert_eq!(
                    bril.contains(" = load "),
                    matches!(
                        mode,
                        ExactLocalAddressMode::Off | ExactLocalAddressMode::Keys
                    ),
                    "{mode:?}: {bril}"
                );
            }
        }
    }
}

#[test]
fn exact_local_addresses_cli_rejects_ablation() {
    for mode in ["keys", "query", "manual"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_eggcc"))
            .args([
                "--with-context",
                "--exact-local-addresses",
                mode,
                "--ablate",
                "mem-simple",
                "input.bril",
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8(output.stderr)
            .unwrap()
            .contains("do not support --ablate"));
        assert!(output.stdout.is_empty());
    }
}
