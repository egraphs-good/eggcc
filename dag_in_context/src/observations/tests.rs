use super::*;
use std::collections::HashSet;

fn fixture(value: i64) -> (TreeProgram, Vec<Value>) {
    let arg_ty = Type::TupleT(vec![BaseType::IntT, BaseType::StateT]);
    let ctx = Assumption::InFunc("main".into());
    let arg = Rc::new(Expr::Arg(arg_ty.clone(), ctx.clone()));
    let x = Rc::new(Expr::Get(arg.clone(), 0));
    let zero = Rc::new(Expr::Const(Constant::Int(0), arg_ty.clone(), ctx.clone()));
    let pred = Rc::new(Expr::Bop(BinaryOp::LessThan, zero, x));
    let branch_ctx = Assumption::InIf(true, pred.clone(), arg.clone());
    let inner_arg = Rc::new(Expr::Arg(arg_ty.clone(), branch_ctx.clone()));
    let product = Rc::new(Expr::Bop(
        BinaryOp::Mul,
        Rc::new(Expr::Get(inner_arg.clone(), 0)),
        Rc::new(Expr::Const(
            Constant::Int(2),
            arg_ty.clone(),
            branch_ctx.clone(),
        )),
    ));
    let test = Rc::new(Expr::Bop(
        BinaryOp::LessThan,
        product,
        Rc::new(Expr::Const(Constant::Int(0), arg_ty.clone(), branch_ctx)),
    ));
    let then = Rc::new(Expr::Single(Rc::new(Expr::Bop(
        BinaryOp::Print,
        test,
        Rc::new(Expr::Get(inner_arg, 1)),
    ))));
    let otherwise = Rc::new(Expr::Single(Rc::new(Expr::Get(
        Rc::new(Expr::Arg(
            arg_ty.clone(),
            Assumption::InIf(false, pred.clone(), arg.clone()),
        )),
        1,
    ))));
    let body = Rc::new(Expr::If(pred, arg, then, otherwise));
    (
        TreeProgram {
            entry: Rc::new(Expr::Function(
                "main".into(),
                arg_ty,
                Type::TupleT(vec![BaseType::StateT]),
                body,
            )),
            functions: vec![],
        },
        vec![Value::Tuple(vec![
            Value::Const(Constant::Int(value)),
            Value::StateV,
        ])],
    )
}

#[test]
fn feasible_branch_wraps_and_structural_print_is_not_evaluated() {
    let (program, inputs) = fixture(i64::MAX);
    let collected = collect(&program, &["main".into()], &inputs, 0).unwrap();
    assert_eq!(collected.points.len(), 2);
    assert!(collected
        .observations
        .iter()
        .any(|o| o.point == 1 && o.value));
    assert_eq!(collected.skipped_regions.len(), 1);
    assert_eq!(collected.skipped_regions[0].observations, 0);
}

#[test]
fn unreachable_region_gets_no_observations() {
    for value in [0, -1] {
        let (program, inputs) = fixture(value);
        let collected = collect(&program, &["main".into()], &inputs, 0).unwrap();
        assert_eq!(collected.points.len(), 2);
        assert!(collected.observations.iter().all(|o| o.point == 0));
        assert_eq!(collected.skipped_regions.len(), 1);
        assert_eq!(collected.skipped_regions[0].observations, 0);
    }
}

#[test]
fn actual_inputs_regions_and_passes_have_distinct_tokens() {
    let (program, mut inputs) = fixture(i64::MAX);
    inputs.extend(fixture(1).1);
    let a = collect(&program, &["main".into()], &inputs, 0).unwrap();
    let b = collect(&program, &["main".into()], &inputs, 1).unwrap();
    assert_eq!(a.points.len(), 4);
    let tokens: HashSet<_> = a.points.iter().chain(&b.points).map(|p| p.token).collect();
    assert_eq!(tokens.len(), 8);
    assert!(a.observations.iter().any(|o| o.point == 1 && o.value));
    assert!(a.observations.iter().any(|o| o.point == 3 && !o.value));
}

#[test]
fn wrong_root_type_and_unsupported_reachable_grammar_reject() {
    let (mut program, _) = fixture(1);
    let wrong = [Value::Tuple(vec![
        Value::Const(Constant::Bool(true)),
        Value::StateV,
    ])];
    assert!(collect(&program, &["main".into()], &wrong, 0)
        .unwrap_err()
        .contains("type"));
    let Expr::Function(name, input, output, _) = program.entry.as_ref() else {
        unreachable!()
    };
    program.entry = Rc::new(Expr::Function(
        name.clone(),
        input.clone(),
        output.clone(),
        Rc::new(Expr::Call(
            "unavailable".into(),
            Rc::new(Expr::Arg(input.clone(), Assumption::InFunc(name.clone()))),
        )),
    ));
    assert!(collect(&program, &["main".into()], &fixture(1).1, 0)
        .unwrap_err()
        .contains("unsupported"));
}

#[test]
fn wrong_context_and_region_arg_type_reject_before_seeding() {
    let (program, inputs) = fixture(1);
    let Expr::Function(name, input, output, _) = program.entry.as_ref() else {
        unreachable!()
    };
    for (ty, ctx) in [
        (input.clone(), Assumption::InFunc("other".into())),
        (
            Type::TupleT(vec![BaseType::BoolT, BaseType::StateT]),
            Assumption::InFunc(name.clone()),
        ),
    ] {
        let bad = TreeProgram {
            entry: Rc::new(Expr::Function(
                name.clone(),
                input.clone(),
                output.clone(),
                Rc::new(Expr::Arg(ty, ctx)),
            )),
            functions: vec![],
        };
        let error = collect(&bad, &["main".into()], &inputs, 0).unwrap_err();
        assert!(error.contains("context") || error.contains("type"));
    }
}

#[test]
fn manual_and_injective_reject_only_same_point_conflict() {
    for mode in [
        ObservationMode::CommonSeed,
        ObservationMode::Manual,
        ObservationMode::Injective,
    ] {
        let mut graph = crate::new_single_threaded_egraph();
        let program = format!(
            "(datatype Expr (A) (B))\n{}\n\
            (ObservationSeed 0 (A) 0 true) (ObservationSeed 1 (B) 0 false)\n\
            (union (EvalBoolAt (A) 0) (ObservedBool true))\n\
            (union (EvalBoolAt (B) 0) (ObservedBool false))\n\
            (union (A) (B))",
            declarations(mode)
        );
        graph.parse_and_run_program(None, &program).unwrap();
        graph
            .parse_and_run_program(None, "(run observation-witness 1)")
            .unwrap();
        assert_eq!(conflicts(&graph).unwrap().len(), 1);
        let rejected = graph
            .parse_and_run_program(None, audit_schedule(mode))
            .is_err();
        assert_eq!(rejected, mode != ObservationMode::CommonSeed);
    }
    for mode in [ObservationMode::Manual, ObservationMode::Injective] {
        let mut graph = crate::new_single_threaded_egraph();
        graph
            .parse_and_run_program(
                None,
                &format!(
                    "(datatype Expr (A) (B))\n{}\n\
            (ObservationSeed 0 (A) 0 true) (ObservationSeed 1 (A) 1 false)\n\
            (union (EvalBoolAt (A) 0) (ObservedBool true))\n\
            (union (EvalBoolAt (A) 1) (ObservedBool false))\n\
            (union (EvalBoolAt (B) 0) (ObservedBool true))",
                    declarations(mode)
                ),
            )
            .unwrap();
        graph
            .parse_and_run_program(None, "(run observation-witness 1)")
            .unwrap();
        assert!(conflicts(&graph).unwrap().is_empty());
        graph
            .parse_and_run_program(None, audit_schedule(mode))
            .unwrap();
        graph
            .parse_and_run_program(None, "(fail (check (= (A) (B))))")
            .unwrap();
    }
}

#[test]
fn equal_type_transport_and_nested_selected_else_use_actual_region_inputs() {
    // Independent oracle: 7 -> 14 and 3 -> 6 -> 12, so the designated
    // branch-local y<10 is false in both cases. Root input reuse is wrong.
    for (root, nested) in [(7, false), (3, true)] {
        let ty = Type::TupleT(vec![BaseType::IntT, BaseType::StateT]);
        let root_ctx = Assumption::InFunc("main".into());
        let root_arg = Rc::new(Expr::Arg(ty.clone(), root_ctx.clone()));
        let pred = if nested {
            Rc::new(Expr::Bop(
                BinaryOp::LessThan,
                Rc::new(Expr::Const(Constant::Int(0), ty.clone(), root_ctx.clone())),
                Rc::new(Expr::Get(root_arg.clone(), 0)),
            ))
        } else {
            Rc::new(Expr::Const(
                Constant::Bool(true),
                ty.clone(),
                root_ctx.clone(),
            ))
        };
        let make_input = |arg: RcExpr, ctx: Assumption| {
            Rc::new(Expr::Concat(
                Rc::new(Expr::Single(Rc::new(Expr::Bop(
                    BinaryOp::Mul,
                    Rc::new(Expr::Get(arg.clone(), 0)),
                    Rc::new(Expr::Const(Constant::Int(2), ty.clone(), ctx)),
                )))),
                Rc::new(Expr::Single(Rc::new(Expr::Get(arg, 1)))),
            ))
        };
        let inputs = make_input(root_arg.clone(), root_ctx);
        let outer_ctx = Assumption::InIf(true, pred.clone(), inputs.clone());
        let outer_arg = Rc::new(Expr::Arg(ty.clone(), outer_ctx.clone()));
        let inner_pred = Rc::new(Expr::Bop(
            BinaryOp::LessThan,
            Rc::new(Expr::Get(outer_arg.clone(), 0)),
            Rc::new(Expr::Const(Constant::Int(5), ty.clone(), outer_ctx.clone())),
        ));
        let inner_inputs = make_input(outer_arg.clone(), outer_ctx.clone());
        let target_ctx = if nested {
            Assumption::InIf(false, inner_pred.clone(), inner_inputs.clone())
        } else {
            outer_ctx.clone()
        };
        let target_arg = if nested {
            Rc::new(Expr::Arg(ty.clone(), target_ctx.clone()))
        } else {
            outer_arg.clone()
        };
        let target = Rc::new(Expr::Bop(
            BinaryOp::LessThan,
            Rc::new(Expr::Get(target_arg.clone(), 0)),
            Rc::new(Expr::Const(Constant::Int(10), ty.clone(), target_ctx)),
        ));
        let printed = Rc::new(Expr::Single(Rc::new(Expr::Bop(
            BinaryOp::Print,
            target.clone(),
            Rc::new(Expr::Get(target_arg, 1)),
        ))));
        let dead_ctx = Assumption::InIf(true, inner_pred.clone(), inner_inputs.clone());
        let dead_arg = Rc::new(Expr::Arg(ty.clone(), dead_ctx.clone()));
        let dead = Rc::new(Expr::Bop(
            BinaryOp::LessThan,
            Rc::new(Expr::Const(Constant::Int(0), ty.clone(), dead_ctx)),
            Rc::new(Expr::Get(dead_arg.clone(), 0)),
        ));
        let body = if nested {
            Rc::new(Expr::If(
                inner_pred,
                inner_inputs,
                Rc::new(Expr::Single(Rc::new(Expr::Bop(
                    BinaryOp::Print,
                    dead.clone(),
                    Rc::new(Expr::Get(dead_arg, 1)),
                )))),
                printed,
            ))
        } else {
            printed
        };
        let dead_outer_ctx = Assumption::InIf(false, pred.clone(), inputs.clone());
        let dead_outer_arg = Rc::new(Expr::Arg(ty.clone(), dead_outer_ctx.clone()));
        let dead_outer = Rc::new(Expr::Bop(
            BinaryOp::LessThan,
            Rc::new(Expr::Get(dead_outer_arg.clone(), 0)),
            Rc::new(Expr::Const(Constant::Int(100), ty.clone(), dead_outer_ctx)),
        ));
        let outer_else = Rc::new(Expr::Single(Rc::new(Expr::Bop(
            BinaryOp::Print,
            dead_outer.clone(),
            Rc::new(Expr::Get(dead_outer_arg, 1)),
        ))));
        let program = TreeProgram {
            entry: Rc::new(Expr::Function(
                "main".into(),
                ty,
                Type::TupleT(vec![BaseType::StateT]),
                Rc::new(Expr::If(pred, inputs, body, outer_else)),
            )),
            functions: vec![],
        };
        let observations = collect(
            &program,
            &["main".into()],
            &[Value::Tuple(vec![
                Value::Const(Constant::Int(root)),
                Value::StateV,
            ])],
            0,
        )
        .unwrap();
        let target_observations: Vec<_> = observations
            .observations
            .iter()
            .filter(|o| Rc::ptr_eq(&o.expr, &target))
            .collect();
        assert_eq!(target_observations.len(), 1);
        assert!(!target_observations[0].value);
        assert!(observations
            .observations
            .iter()
            .all(|o| !Rc::ptr_eq(&o.expr, &dead)));
        assert_eq!(observations.points.len(), if nested { 3 } else { 2 });
        assert_eq!(
            observations.points[1].input,
            Value::Tuple(vec![Value::Const(Constant::Int(root * 2)), Value::StateV])
        );
        if nested {
            assert_eq!(
                observations.points[2].input,
                Value::Tuple(vec![Value::Const(Constant::Int(12)), Value::StateV])
            );
        }
        assert!(observations
            .observations
            .iter()
            .all(|o| !Rc::ptr_eq(&o.expr, &dead_outer)));
    }
}

#[test]
fn same_contextual_rc_under_three_and_twenty_has_two_payloads_and_audits_accept() {
    let ty = Type::TupleT(vec![BaseType::IntT, BaseType::StateT]);
    let ctx = Assumption::InFunc("main".into());
    let arg = Rc::new(Expr::Arg(ty.clone(), ctx.clone()));
    let shared = Rc::new(Expr::Bop(
        BinaryOp::LessThan,
        Rc::new(Expr::Get(arg.clone(), 0)),
        Rc::new(Expr::Const(Constant::Int(10), ty.clone(), ctx)),
    ));
    let program = TreeProgram {
        entry: Rc::new(Expr::Function(
            "main".into(),
            ty,
            Type::TupleT(vec![BaseType::StateT]),
            Rc::new(Expr::Single(Rc::new(Expr::Bop(
                BinaryOp::Print,
                shared.clone(),
                Rc::new(Expr::Get(arg, 1)),
            )))),
        )),
        functions: vec![],
    };
    let inputs: Vec<_> = [3, 20]
        .into_iter()
        .map(|x| Value::Tuple(vec![Value::Const(Constant::Int(x)), Value::StateV]))
        .collect();
    let collected = collect(&program, &["main".into()], &inputs, 0).unwrap();
    let records: Vec<_> = collected
        .observations
        .iter()
        .filter(|o| Rc::ptr_eq(&o.expr, &shared))
        .collect();
    assert_eq!(
        records.iter().map(|o| o.value).collect::<Vec<_>>(),
        vec![true, false]
    );
    assert_ne!(records[0].point, records[1].point);
    assert_eq!(collected.points.len(), 2);
    for mode in [ObservationMode::Manual, ObservationMode::Injective] {
        let dir = tempfile::tempdir().unwrap();
        let config = crate::EggccConfig {
            observations: Some(ObservationConfig {
                mode,
                root_inputs: inputs.clone(),
                evidence_dir: dir.path().to_owned(),
                prepass_probe: false,
            }),
            ..crate::EggccConfig::default()
        };
        let built = crate::build_program(&program, None, &["main".into()], "", &config, 0, None);
        let mut graph = crate::new_single_threaded_egraph();
        let observer = built.observations.as_ref().unwrap();
        observer
            .initialize(&mut graph, &built.initialization, &built.schedule)
            .unwrap();
        observer.audit(&mut graph).unwrap();
        assert!(conflicts(&graph).unwrap().is_empty());
    }
}

#[test]
fn state_carrier_print_is_structural_and_detached_evaluation_is_only_pure_bool() {
    let ty = Type::TupleT(vec![BaseType::IntT, BaseType::StateT]);
    let ctx = Assumption::InFunc("main".into());
    let arg = Rc::new(Expr::Arg(ty.clone(), ctx.clone()));
    let pred = Rc::new(Expr::Const(Constant::Bool(true), ty.clone(), ctx.clone()));
    let print = Rc::new(Expr::Bop(
        BinaryOp::Print,
        Rc::new(Expr::Get(arg.clone(), 0)),
        Rc::new(Expr::Get(arg.clone(), 1)),
    ));
    let inputs = Rc::new(Expr::Concat(
        Rc::new(Expr::Single(Rc::new(Expr::Bop(
            BinaryOp::Mul,
            Rc::new(Expr::Get(arg, 0)),
            Rc::new(Expr::Const(Constant::Int(2), ty.clone(), ctx)),
        )))),
        Rc::new(Expr::Single(print.clone())),
    ));
    let branch_ctx = Assumption::InIf(true, pred.clone(), inputs.clone());
    let branch_arg = Rc::new(Expr::Arg(ty.clone(), branch_ctx.clone()));
    let target = Rc::new(Expr::Bop(
        BinaryOp::LessThan,
        Rc::new(Expr::Get(branch_arg.clone(), 0)),
        Rc::new(Expr::Const(Constant::Int(10), ty.clone(), branch_ctx)),
    ));
    let then = Rc::new(Expr::Concat(
        Rc::new(Expr::Single(target.clone())),
        Rc::new(Expr::Single(Rc::new(Expr::Get(branch_arg, 1)))),
    ));
    let else_ctx = Assumption::InIf(false, pred.clone(), inputs.clone());
    let otherwise = Rc::new(Expr::Concat(
        Rc::new(Expr::Single(Rc::new(Expr::Const(
            Constant::Bool(false),
            ty.clone(),
            else_ctx.clone(),
        )))),
        Rc::new(Expr::Single(Rc::new(Expr::Get(
            Rc::new(Expr::Arg(ty.clone(), else_ctx)),
            1,
        )))),
    ));
    let program = TreeProgram {
        entry: Rc::new(Expr::Function(
            "main".into(),
            ty,
            Type::TupleT(vec![BaseType::BoolT, BaseType::StateT]),
            Rc::new(Expr::If(pred.clone(), inputs, then, otherwise)),
        )),
        functions: vec![],
    };
    let input = Value::Tuple(vec![Value::Const(Constant::Int(7)), Value::StateV]);
    let collected = collect(&program, &["main".into()], std::slice::from_ref(&input), 0).unwrap();
    assert_eq!(collected.points.len(), 2);
    assert_eq!(
        collected.points[1].input,
        Value::Tuple(vec![Value::Const(Constant::Int(14)), Value::StateV])
    );
    assert_eq!(collected.structural_print_visits, 1);
    // The collector invokes the detached helper exactly at its pure Bool
    // observation records; neither the If/input tuple nor Print is among them.
    assert_eq!(collected.observations.len(), 2);
    assert!(collected
        .observations
        .iter()
        .all(|o| Rc::ptr_eq(&o.expr, &pred) || Rc::ptr_eq(&o.expr, &target)));
    assert!(collected
        .observations
        .iter()
        .any(|o| Rc::ptr_eq(&o.expr, &target) && !o.value));
    let (value, log) = crate::interpreter::interpret_dag_prog(&program, &input);
    assert_eq!(
        value,
        Value::Tuple(vec![Value::Const(Constant::Bool(false)), Value::StateV])
    );
    assert_eq!(log, vec!["7"]);
}

#[test]
fn prepass_probe_uses_initialized_closed_terms_and_distinguishes_max_from_one() {
    for (input, expected) in [(i64::MAX, true), (1, false)] {
        let (program, inputs) = fixture(input);
        let dir = tempfile::tempdir().unwrap();
        let config = crate::EggccConfig {
            observations: Some(ObservationConfig {
                mode: ObservationMode::Injective,
                root_inputs: inputs,
                evidence_dir: dir.path().to_owned(),
                prepass_probe: true,
            }),
            ..crate::EggccConfig::default()
        };
        let built = crate::build_program(&program, None, &["main".into()], "", &config, 0, None);
        let mut graph = crate::new_single_threaded_egraph();
        built
            .observations
            .as_ref()
            .unwrap()
            .initialize(&mut graph, &built.initialization, &built.schedule)
            .unwrap();
        let receipt: serde_json::Value = serde_json::from_slice(
            &std::fs::read(dir.path().join("pass-00/prepass-probe.json")).unwrap(),
        )
        .unwrap();
        let outcomes = receipt["outcomes"].as_array().unwrap();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0]["known_disequal"], expected);
        assert!(!outcomes[0]["query"].as_str().unwrap().contains("__tmp"));
    }
}
