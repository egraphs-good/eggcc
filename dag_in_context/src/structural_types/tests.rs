use super::*;
use egglog::{new_experimental_egraph_with_options, DisequalityEncoding};

const ACTUAL: &str = "(TupleT (TCons (IntT) (TCons (PointerT (IntT)) (TCons (StateT) (TNil)))))";
const EXPECTED: &str = "(TupleT (TCons (IntT) (TCons (PointerT (BoolT)) (TCons (StateT) (TNil)))))";

fn prepare(
    mode: StructuralTypeMode,
    encoding: DisequalityEncoding,
    expected: &str,
) -> egglog::EGraph {
    let mut graph = new_experimental_egraph_with_options(false, encoding);
    let config = crate::EggccConfig {
        structural_types: mode,
        ..crate::EggccConfig::default()
    };
    graph
        .parse_and_run_program(None, &crate::prologue_for_config(&config))
        .unwrap();
    graph
        .parse_and_run_program(
            None,
            &format!(
                "(let actual {ACTUAL})
                 (let expected {expected})
                 (let body (Arg actual (InFunc \"structural_type_fixture\")))
                 (Function \"structural_type_fixture\" actual expected body)
                 (run-schedule (saturate (saturate type-helpers) type-analysis))"
            ),
        )
        .unwrap();
    graph
}

#[test]
fn structural_type_nested_actual_consumer_and_feature_controls() {
    for encoding in [DisequalityEncoding::Nee, DisequalityEncoding::Ee] {
        for mode in [
            StructuralTypeMode::Off,
            StructuralTypeMode::Manual,
            StructuralTypeMode::Combined,
            StructuralTypeMode::AnnotationsOff,
            StructuralTypeMode::QueryOff,
        ] {
            let mut graph = prepare(mode, encoding, EXPECTED);
            graph
                .parse_and_run_program(
                    None,
                    "(check (ExpectType body expected \"Function body had wrong type\"))
                     (check (HasType body actual))",
                )
                .unwrap();
            if mode != StructuralTypeMode::Off {
                graph
                    .parse_and_run_program(
                        None,
                        "(run-schedule (saturate structural-type-consume))",
                    )
                    .unwrap();
            }
            for (stage, left, right) in [
                ("leaf", "(IntT)", "(BoolT)"),
                ("pointer", "(PointerT (IntT))", "(PointerT (BoolT))"),
                (
                    "inner-list",
                    "(TCons (PointerT (IntT)) (TCons (StateT) (TNil)))",
                    "(TCons (PointerT (BoolT)) (TCons (StateT) (TNil)))",
                ),
                (
                    "outer-list",
                    "(TCons (IntT) (TCons (PointerT (IntT)) (TCons (StateT) (TNil))))",
                    "(TCons (IntT) (TCons (PointerT (BoolT)) (TCons (StateT) (TNil))))",
                ),
                ("tuple", ACTUAL, EXPECTED),
            ] {
                let known = match graph.parse_and_run_program(
                    None,
                    &format!("(check (known-disequal? {left} {right}))"),
                ) {
                    Ok(_) => true,
                    Err(egglog::Error::CheckError(..)) => false,
                    Err(error) => panic!("{mode:?}/{encoding:?}/{stage}: {error}"),
                };
                let expected_known = match mode {
                    StructuralTypeMode::Off => false,
                    StructuralTypeMode::AnnotationsOff => stage == "leaf",
                    _ => true,
                };
                assert_eq!(known, expected_known, "{mode:?}/{encoding:?}/{stage}");
                eprintln!(
                    "structural-type-witness {}",
                    serde_json::json!({"case":"nested-pointer-second-slot","encoding":format!("{encoding:?}"),"mode":format!("{mode:?}"),"stage":stage,"left":left,"right":right,"known_disequal":known})
                );
            }
            if matches!(
                mode,
                StructuralTypeMode::AnnotationsOff | StructuralTypeMode::QueryOff
            ) {
                graph
                    .parse_and_run_program(
                        None,
                        "(fail (check (StructuralTypeMismatch expected actual)))",
                    )
                    .unwrap();
                let error = validate(&graph).unwrap_err().to_string();
                assert!(
                    error.contains("coverage-unknown"),
                    "{mode:?}/{encoding:?}: {error}"
                );
                graph
                    .parse_and_run_program(None, "(run error-checking 1)")
                    .unwrap();
            } else {
                let error = graph
                    .parse_and_run_program(None, "(run error-checking 1)")
                    .unwrap_err()
                    .to_string();
                assert!(
                    error.contains("type mismatch- check RUST_LOG"),
                    "{mode:?}/{encoding:?}: {error}"
                );
            }
            eprintln!(
                "structural-type-outcome {}",
                serde_json::json!({"case":"nested-pointer-second-slot","encoding":format!("{encoding:?}"),"mode":format!("{mode:?}"),"consumer":if matches!(mode, StructuralTypeMode::AnnotationsOff | StructuralTypeMode::QueryOff) {"coverage-unknown"} else {"type-mismatch"},"actual_producers_checked":true})
            );
        }
    }
}

#[test]
fn structural_type_equal_helper_normal_forms_validate() {
    for encoding in [DisequalityEncoding::Nee, DisequalityEncoding::Ee] {
        for mode in [
            StructuralTypeMode::Manual,
            StructuralTypeMode::Combined,
            StructuralTypeMode::AnnotationsOff,
            StructuralTypeMode::QueryOff,
        ] {
            // Append, remove and indexing all normalize to the unchanged actual type.
            let expected = "(TupleT (TLConcat (TCons (TypeList-ith (TCons (IntT) (TNil)) 0) (TNil))
                (TypeListRemoveAt (TCons (BoolT) (TCons (PointerT (IntT)) (TCons (StateT) (TNil)))) 0)))";
            let mut graph = prepare(mode, encoding, expected);
            graph
                .parse_and_run_program(None, "(run-schedule (saturate structural-type-consume)) (run error-checking 1) (check (= actual expected))")
                .unwrap();
            assert_eq!(validate(&graph).unwrap(), 1);
        }
    }
}

#[test]
fn structural_type_query_off_retains_direct_tag_consumer() {
    for encoding in [DisequalityEncoding::Nee, DisequalityEncoding::Ee] {
        for mode in [
            StructuralTypeMode::Manual,
            StructuralTypeMode::Combined,
            StructuralTypeMode::AnnotationsOff,
            StructuralTypeMode::QueryOff,
        ] {
            let mut graph = prepare(mode, encoding, "(Base (IntT))");
            graph.parse_and_run_program(None, "(run-schedule (saturate structural-type-consume)) (check (StructuralTypeMismatch expected actual))").unwrap();
            let error = graph
                .parse_and_run_program(None, "(run error-checking 1)")
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("type mismatch- check RUST_LOG"),
                "{mode:?}/{encoding:?}: {error}"
            );
        }
    }
}

#[test]
fn structural_type_mismatch_shapes_match_manual_and_existing() {
    for expected in [
        "(TupleT (TCons (BoolT) (TCons (PointerT (IntT)) (TCons (StateT) (TNil)))))",
        "(TupleT (TCons (IntT) (TCons (PointerT (IntT)) (TCons (FloatT) (TNil)))))",
        "(TupleT (TCons (IntT) (TCons (PointerT (IntT)) (TNil))))",
        "(TupleT (TCons (IntT) (TCons (BoolT) (TCons (StateT) (TNil)))))",
    ] {
        for mode in [
            StructuralTypeMode::Off,
            StructuralTypeMode::Manual,
            StructuralTypeMode::Combined,
        ] {
            let mut graph = prepare(mode, DisequalityEncoding::Nee, expected);
            if mode != StructuralTypeMode::Off {
                graph
                    .parse_and_run_program(
                        None,
                        "(run-schedule (saturate structural-type-consume))",
                    )
                    .unwrap();
            }
            let error = graph
                .parse_and_run_program(None, "(run error-checking 1)")
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("type mismatch- check RUST_LOG"),
                "{mode:?}/{expected}: {error}"
            );
        }
    }
}

#[test]
fn structural_type_unsupported_and_missing_producer_fail_closed() {
    for expected in ["(TmpType)", "(Base (TypeList-ith (TNil) -1))"] {
        let mut graph = prepare(
            StructuralTypeMode::Combined,
            DisequalityEncoding::Nee,
            expected,
        );
        graph
            .parse_and_run_program(None, "(run-schedule (saturate structural-type-consume))")
            .unwrap();
        assert!(validate(&graph)
            .unwrap_err()
            .to_string()
            .contains("unsupported"));
    }
    let mut graph = prepare(
        StructuralTypeMode::Combined,
        DisequalityEncoding::Nee,
        ACTUAL,
    );
    graph.parse_and_run_program(None, "(constructor Opaque () Expr) (ExpectType (Opaque) actual \"missing producer\") (run-schedule (saturate type-helpers))").unwrap();
    assert!(validate(&graph)
        .unwrap_err()
        .to_string()
        .contains("missing-actual"));
}

#[test]
fn structural_type_late_normalization_and_multiple_actuals_are_audited() {
    let mut graph = prepare(
        StructuralTypeMode::Combined,
        DisequalityEncoding::Nee,
        ACTUAL,
    );
    graph
        .parse_and_run_program(None, "(run-schedule (saturate structural-type-consume))")
        .unwrap();
    assert_eq!(validate(&graph).unwrap(), 1);
    graph.parse_and_run_program(None, "(constructor Later () Type) (HasType body (Later)) (run-schedule (saturate type-helpers))").unwrap();
    assert!(validate(&graph)
        .unwrap_err()
        .to_string()
        .contains("unsupported"));
    graph.parse_and_run_program(None, "(union (Later) actual) (run-schedule (saturate type-helpers) (saturate structural-type-consume))").unwrap();
    assert_eq!(validate(&graph).unwrap(), 1);
}

#[test]
fn structural_type_missing_tag_law_stays_unknown() {
    let mut graph = new_experimental_egraph_with_options(false, DisequalityEncoding::Nee);
    let config = crate::EggccConfig {
        structural_types: StructuralTypeMode::Combined,
        ..crate::EggccConfig::default()
    };
    // This control removes the one root tag law, rather than seeding a composite result.
    let prologue =
        crate::prologue_for_config(&config).replace("(StructuralBaseTagDQ (IntT) (BoolT))", "");
    graph.parse_and_run_program(None, &prologue).unwrap();
    graph.parse_and_run_program(None, &format!("(let actual {ACTUAL}) (let expected {EXPECTED}) (let body (Arg actual (InFunc \"f\"))) (Function \"f\" actual expected body) (run-schedule (saturate (saturate type-helpers) type-analysis) (saturate structural-type-consume))")).unwrap();
    graph
        .parse_and_run_program(None, "(fail (check (known-disequal? expected actual)))")
        .unwrap();
    assert!(validate(&graph)
        .unwrap_err()
        .to_string()
        .contains("coverage-unknown"));
}

#[test]
fn structural_type_corrupt_tag_union_rejects_and_equal_sentinel_is_unsupported() {
    for encoding in [DisequalityEncoding::Nee, DisequalityEncoding::Ee] {
        for mode in [
            StructuralTypeMode::Manual,
            StructuralTypeMode::Combined,
            StructuralTypeMode::AnnotationsOff,
            StructuralTypeMode::QueryOff,
        ] {
            let mut graph = prepare(mode, encoding, ACTUAL);
            graph
                .parse_and_run_program(
                    None,
                    "(union (IntT) (BoolT)) (run-schedule (saturate type-helpers))",
                )
                .unwrap();
            let error = graph
                .parse_and_run_program(None, "(run-schedule (saturate structural-type-consume))")
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("structural-type-inconsistent-tags"),
                "{mode:?}/{encoding:?}: {error}"
            );
        }

        let mut graph = prepare(StructuralTypeMode::Combined, encoding, ACTUAL);
        graph.parse_and_run_program(None, "(let temporary (Arg (TmpType) (InFunc \"temporary\"))) (Function \"temporary\" (TmpType) (TmpType) temporary) (run-schedule (saturate (saturate type-helpers) type-analysis) (saturate structural-type-consume))").unwrap();
        assert!(validate(&graph)
            .unwrap_err()
            .to_string()
            .contains("unsupported"));
    }
}

#[test]
fn structural_type_matches_independent_rust_type_equality_oracle() {
    use crate::schema::{BaseType::*, Type};

    // Rust's derived Type/Vec/BaseType equality is independent of both the
    // engine query and the hand-written Egglog structural mismatch rules.
    let actual = Type::TupleT(vec![IntT, PointerT(Box::new(IntT)), StateT]);
    assert_eq!(actual.to_string(), ACTUAL);
    let cases = [
        ("equal", actual.clone()),
        (
            "nested-pointer-second-slot",
            Type::TupleT(vec![IntT, PointerT(Box::new(BoolT)), StateT]),
        ),
        (
            "head",
            Type::TupleT(vec![BoolT, PointerT(Box::new(IntT)), StateT]),
        ),
        (
            "tail",
            Type::TupleT(vec![IntT, PointerT(Box::new(IntT)), FloatT]),
        ),
        ("length", Type::TupleT(vec![IntT, PointerT(Box::new(IntT))])),
        ("leaf-pointer-tag", Type::TupleT(vec![IntT, BoolT, StateT])),
        ("base-tuple-tag", Type::Base(IntT)),
        (
            "deeper-pointer",
            Type::TupleT(vec![
                IntT,
                PointerT(Box::new(PointerT(Box::new(IntT)))),
                StateT,
            ]),
        ),
        ("empty", Type::TupleT(vec![])),
    ];
    for encoding in [DisequalityEncoding::Nee, DisequalityEncoding::Ee] {
        for mode in [
            StructuralTypeMode::Off,
            StructuralTypeMode::Manual,
            StructuralTypeMode::Combined,
        ] {
            for (case, expected) in &cases {
                let oracle_mismatch = actual != *expected;
                let mut graph = prepare(mode, encoding, &expected.to_string());
                graph
                    .parse_and_run_program(
                        None,
                        "(check (ExpectType body expected \"Function body had wrong type\"))
                         (check (HasType body actual))",
                    )
                    .unwrap();
                if mode != StructuralTypeMode::Off {
                    graph
                        .parse_and_run_program(
                            None,
                            "(run-schedule (saturate structural-type-consume))",
                        )
                        .unwrap();
                }
                let observed = match graph.parse_and_run_program(None, "(run error-checking 1)") {
                    Ok(_) => {
                        if mode != StructuralTypeMode::Off {
                            assert_eq!(validate(&graph).unwrap(), 1);
                        }
                        "equal"
                    }
                    Err(error) => {
                        assert!(
                            error.to_string().contains("type mismatch- check RUST_LOG"),
                            "{mode:?}/{encoding:?}/{case}: {error}"
                        );
                        "type-mismatch"
                    }
                };
                assert_eq!(
                    observed == "type-mismatch",
                    oracle_mismatch,
                    "{mode:?}/{encoding:?}/{case}"
                );
                eprintln!(
                    "structural-type-oracle {}",
                    serde_json::json!({"oracle":"Rust schema::Type derived PartialEq","case":case,"encoding":format!("{encoding:?}"),"mode":format!("{mode:?}"),"actual_type":actual.to_string(),"expected_type":expected.to_string(),"expected_mismatch":oracle_mismatch,"observed_consumer":observed,"actual_producers_checked":true})
                );
            }
        }
    }
}
