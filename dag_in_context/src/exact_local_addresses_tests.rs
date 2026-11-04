use super::*;
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    time::Instant,
};

const PREFIX: &str = r#"
(let ty (TupleT (TCons (StateT) (TNil))))
(let ctx (InFunc "main"))
(FunctionHasType "main" ty ty)
(let initial (Get (Arg ty ctx) 0))
(let size (Const (Int 1) ty ctx))
(let a0 (Alloc 0 size initial (PointerT (IntT))))
(let p (Get a0 0))
(let a1 (Alloc 0 size (Get a0 1) (PointerT (IntT))))
(let q (Get a1 0))
"#;

#[test]
fn exact_local_addresses_reject_seeded_conflicts() {
    for mode in [
        ExactLocalAddressMode::Keys,
        ExactLocalAddressMode::Query,
        ExactLocalAddressMode::Manual,
    ] {
        let config = EggccConfig {
            exact_local_addresses: mode,
            ..EggccConfig::default()
        };
        for (poison, diagnostic) in [
            ("(union p q)", "pointer has incompatible keys"),
            (
                "(FreshState initial \"main\" 7)",
                "state has unequal ordinal counts",
            ),
            (
                "(union initial (Get a0 1))",
                "state has unequal ordinal counts",
            ),
            (
                "(union (LocalCell \"main\" 0 0) (LocalCell \"main\" 1 0))",
                "injectivity violation",
            ),
        ] {
            let program = format!(
                "{}\n{PREFIX}\n{poison}\n{}\n(panic \"reached serialization sentinel\")",
                prologue_for_config(&config),
                mode.egglog_schedule()
            );
            let start = Instant::now();
            let result = catch_unwind(AssertUnwindSafe(|| {
                new_single_threaded_egraph().parse_and_run_program(None, &program)
            }));
            let message = match result {
                Ok(Err(error)) => error.to_string(),
                Err(payload) => payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                    .expect("string panic diagnostic"),
                Ok(Ok(_)) => panic!("seeded conflict was accepted: {mode:?}: {poison}"),
            };
            assert!(
                message.contains(diagnostic),
                "{mode:?}: {poison}: {message}"
            );
            assert!(!message.contains("serialization sentinel"), "{message}");
            assert!(
                start.elapsed().as_secs_f64() < 2.0,
                "rejection was not prompt: {mode:?}: {poison}"
            );
        }
    }
}

#[test]
fn exact_local_addresses_require_supported_configuration() {
    use crate::ast::*;
    let program = function(
        "main",
        tuplet_vec(vec![statet()]),
        tuplet_vec(vec![statet()]),
        single(getat(0)),
    )
    .func_to_program();
    for mode in [
        ExactLocalAddressMode::Keys,
        ExactLocalAddressMode::Query,
        ExactLocalAddressMode::Manual,
    ] {
        let config = EggccConfig {
            exact_local_addresses: mode,
            use_context: false,
            ..EggccConfig::default()
        };
        let result = catch_unwind(AssertUnwindSafe(|| {
            build_program(&program, None, &program.fns(), "", &config)
        }));
        assert!(result.is_err(), "{mode:?} accepted erased contexts");
        let zero_pass = EggccConfig {
            stop_after_n_passes: 0,
            ..config.clone()
        };
        assert!(catch_unwind(AssertUnwindSafe(|| optimize(&program, &zero_pass))).is_err());
        let config = EggccConfig {
            exact_local_addresses: mode,
            ablate: Some("mem-simple".to_string()),
            ..EggccConfig::default()
        };
        let result = catch_unwind(AssertUnwindSafe(|| {
            build_program(&program, None, &program.fns(), "", &config)
        }));
        assert!(result.is_err(), "{mode:?} accepted ablation");
        let zero_pass = EggccConfig {
            stop_after_n_passes: 0,
            ..config.clone()
        };
        assert!(catch_unwind(AssertUnwindSafe(|| optimize(&program, &zero_pass))).is_err());
    }
}
