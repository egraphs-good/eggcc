use bril_rs::{ConstOps, Literal, Type, ValueOps};
use egglog::{EGraph, TermDag};

use crate::{
    cfg::program_to_cfg,
    rvsdg::{cfg_to_rvsdg, BasicExpr, Id, Operand, RvsdgBody},
    util::parse_from_string,
};

use super::{egglog_optimizer::rvsdg_egglog_code, RvsdgFunction, RvsdgType};

pub fn new_rvsdg_egraph() -> EGraph {
    let mut egraph = EGraph::default();
    egraph
        .parse_and_run_program(rvsdg_egglog_code().as_str())
        .unwrap();
    egraph
}

/// Utility struct for building an RVSDG.
#[derive(Default)]
struct RvsdgTest {
    nodes: Vec<RvsdgBody>,
}

impl RvsdgTest {
    /// "pure" functions are ones whose state edges 'pass through'.
    fn into_pure_function(
        self,
        name: String,
        args: Vec<Type>,
        output_ty: Type,
        output: Operand,
    ) -> RvsdgFunction {
        self.into_function(
            name,
            args.clone(),
            Some((output_ty, output)),
            Some(Operand::Arg(args.len())),
        )
    }

    fn into_function(
        self,
        name: String,
        args: Vec<Type>,
        result: Option<(Type, Operand)>,
        state: Option<Operand>,
    ) -> RvsdgFunction {
        let mut wrapped_args: Vec<_> = args.clone().into_iter().map(RvsdgType::Bril).collect();
        wrapped_args.push(RvsdgType::PrintState);
        let results = result
            .map(|(t, s)| (RvsdgType::Bril(t), s))
            .into_iter()
            .chain(state.map(|s| (RvsdgType::PrintState, s)).into_iter())
            .collect();

        RvsdgFunction {
            name,
            args: wrapped_args,
            nodes: self.nodes,
            results,
        }
    }

    fn lit_int(&mut self, i: i64) -> Operand {
        self.make_node(RvsdgBody::BasicOp(BasicExpr::Const(
            ConstOps::Const,
            Literal::Int(i),
            Type::Int,
        )))
    }

    fn lit_bool(&mut self, b: bool) -> Operand {
        self.make_node(RvsdgBody::BasicOp(BasicExpr::Const(
            ConstOps::Const,
            Literal::Bool(b),
            Type::Bool,
        )))
    }

    fn void_function(&mut self, func: impl Into<String>, args: &[Operand]) -> Operand {
        self.make_node(RvsdgBody::BasicOp(BasicExpr::Call(
            func.into(),
            args.to_vec(),
            1,
            None,
        )))
    }

    fn lt(&mut self, l: Operand, r: Operand) -> Operand {
        self.make_node(RvsdgBody::BasicOp(BasicExpr::Op(
            ValueOps::Lt,
            vec![l, r],
            Type::Bool,
        )))
    }

    fn add(&mut self, l: Operand, r: Operand, ty: Type) -> Operand {
        self.make_node(RvsdgBody::BasicOp(BasicExpr::Op(
            ValueOps::Add,
            vec![l, r],
            ty,
        )))
    }

    fn mul(&mut self, l: Operand, r: Operand, ty: Type) -> Operand {
        self.make_node(RvsdgBody::BasicOp(BasicExpr::Op(
            ValueOps::Mul,
            vec![l, r],
            ty,
        )))
    }

    fn print(&mut self, x: Operand, state: Operand) -> Operand {
        self.make_node(RvsdgBody::BasicOp(BasicExpr::Print(vec![x, state])))
    }

    fn gamma(&mut self, pred: Operand, inputs: &[Operand], outputs: &[&[Operand]]) -> Id {
        let res = self.nodes.len();
        self.nodes.push(RvsdgBody::Gamma {
            pred,
            inputs: inputs.to_vec(),
            outputs: outputs.iter().map(|outs| outs.to_vec()).collect(),
        });
        res
    }

    fn theta(&mut self, pred: Operand, inputs: &[Operand], outputs: &[Operand]) -> Id {
        let res = self.nodes.len();
        self.nodes.push(RvsdgBody::Theta {
            pred,
            inputs: inputs.to_vec(),
            outputs: outputs.to_vec(),
        });
        res
    }

    fn make_node(&mut self, body: RvsdgBody) -> Operand {
        let res = Operand::Project(0, self.nodes.len());
        self.nodes.push(body);
        res
    }
}

#[test]
fn rvsdg_expr() {
    const PROGRAM: &str = r#"
    @sub() : int {
        v0: int = const 1;
        v1: int = const 2;
        v2: int = add v0 v1;
        ret v2;
    }
    "#;
    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    let mut expected = RvsdgTest::default();
    let one = expected.lit_int(1);
    let two = expected.lit_int(2);
    let res = expected.add(one, two, Type::Int);
    assert!(deep_equal(
        &expected.into_pure_function("sub".to_owned(), vec![], Type::Int, res),
        &rvsdg.functions[0]
    ));
}

#[test]
fn rvsdg_print() {
    const PROGRAM: &str = r#"
    @sub() {
        v0: int = const 1;
        v1: int = const 2;
        v2: int = add v0 v1;
        print v2;
        print v1;
    }
    "#;
    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    let mut expected = RvsdgTest::default();
    let v0 = expected.lit_int(1);
    let v1 = expected.lit_int(2);
    let v2 = expected.add(v0, v1, Type::Int);
    let res1 = expected.print(v2, Operand::Arg(0));
    let res2 = expected.print(v1, res1);
    assert!(deep_equal(
        &expected.into_function("sub".to_owned(), vec![], None, Some(res2)),
        &rvsdg.functions[0]
    ));
}

#[test]
fn rvsdg_state_gamma() {
    const PROGRAM: &str = r#"
    @sub() {
        x: int = const 1;
        c: bool = const true;
        br c .B .C;
    .B:
        call @some_func;
        jmp .End;
    .C:
        call @other_func;
        jmp .End;
    .End:
    }

    @other_func() {
    }

    @some_func() {
    }
    "#;
    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    let mut expected = RvsdgTest::default();
    let c = expected.lit_bool(true);
    let some_func = expected.void_function("some_func", &[Operand::Arg(0)]);
    let other_func = expected.void_function("other_func", &[Operand::Arg(0)]);
    let gamma = expected.gamma(c, &[Operand::Arg(0)], &[&[other_func], &[some_func]]);
    let res = Operand::Project(0, gamma);
    let expected = expected.into_function("sub".to_owned(), vec![], None, Some(res));
    dbg!(&expected);
    dbg!(&rvsdg.functions[0]);
    assert!(deep_equal(&expected, &rvsdg.functions[0]));
}

#[test]
fn rvsdg_unstructured() {
    const PROGRAM: &str = r#"@main(): int {
        x: int = const 4;
        a_cond: bool = lt x x;
        br a_cond .B .C;
      .B:
        a: int = const 1;
        b_cond: bool = lt x a;
        x: int = add x a;
        br b_cond .C .D;
      .C:
        jmp .B;
      .D:
        ret x;
      }"#;
    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = &cfg_to_rvsdg(&cfg).unwrap().functions[0];

    // It's hard to write a useful test that's more than just a "change
    // detector" here. In this case, the function is not computing anything
    // meaningful, but we know it should have the following properties:
    //
    // 1. A theta node: .B and .C form a cycle.
    // 2. A gamma node, as there is a join point in .B for the value of `x`
    // (whether the predecessor is .B or the entry block).
    assert!(rvsdg.results.len() == 2); // return value + state edge
    assert!(search_for(rvsdg, |body| matches!(
        body,
        RvsdgBody::Theta { .. }
    )));
    assert!(search_for(rvsdg, |body| matches!(
        body,
        RvsdgBody::Gamma { .. }
    )))
}

#[test]
fn rvsdg_basic_odd_branch() {
    // Bril program summing the numbers from 1 to n, multiplying by 2 if that
    // value is larger than 5. This gives us a theta node and a gamma
    // node, with the gamma requiring branch restructuring.
    const PROGRAM: &str = r#"
 @main(n: int): int {
    res: int = const 0;
    i: int = const 0;
 .loop:
    one: int = const 1;
    res: int = add res i;
    i: int = add i one;
    loop_cond: bool = lt i n;
    br loop_cond .loop .tail;
 .tail:
   five: int = const 5;
   rescale_cond: bool = lt res five;
   br rescale_cond .rescale .exit;
 .rescale:
   two: int = const 2;
   res: int = mul res two;
 .exit:
  ret res;
}"#;

    // construct expected program
    let mut expected = RvsdgTest::default();
    let state = Operand::Arg(1);
    let zero = expected.lit_int(0);
    let one = expected.lit_int(1);
    let two = expected.lit_int(2);
    let five = expected.lit_int(5);

    // loop variables
    let res = Operand::Arg(1);
    let i = Operand::Arg(2);
    let n = Operand::Arg(3);

    let ip1 = expected.add(i, one, Type::Int);
    let rpi = expected.add(res, i, Type::Int);
    let pred = expected.lt(ip1, n);
    let theta = expected.theta(
        pred,
        &[state, zero, zero, Operand::Arg(0)],
        &[
            Operand::Arg(0), // state = state
            rpi,             // res = res + i
            ip1,             // i = i + 1
            n,               // n = n
        ],
    );
    let state = Operand::Project(0, theta);
    let res = Operand::Project(1, theta);
    let pred = expected.lt(res, five);
    let mul2 = expected.mul(Operand::Arg(1), two, Type::Int);
    let gamma = expected.gamma(
        pred,
        &[state, res],
        &[
            &[Operand::Arg(0), Operand::Arg(1)],
            &[Operand::Arg(0), mul2],
        ],
    );
    let expected = expected.into_function(
        "main".to_owned(),
        vec![Type::Int],
        Some((Type::Int, Operand::Project(1, gamma))),
        Some(Operand::Project(0, gamma)),
    );

    // test correctness of RVSDGs converted from CFG
    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let actual = &cfg_to_rvsdg(&cfg).unwrap().functions[0];

    assert!(deep_equal(&expected, actual));
}

#[test]
fn rvsdg_odd_branch_egg_roundtrip() {
    // Bril program summing the numbers from 1 to n, multiplying by 2 if that
    // value is larger than 5. This gives us a theta node and a gamma
    // node, with the gamma requiring branch restructuring.
    const PROGRAM: &str = r#"
 @main(n: int): int {
    res: int = const 0;
    i: int = const 0;
 .loop:
    one: int = const 1;
    res: int = add res i;
    i: int = add i one;
    loop_cond: bool = lt i n;
    br loop_cond .loop .tail;
 .tail:
   five: int = const 5;
   rescale_cond: bool = lt res five;
   br rescale_cond .rescale .exit;
 .rescale:
   two: int = const 2;
   res: int = mul res two;
 .exit:
  ret res;
}"#;

    // construct expected program
    let mut expected = RvsdgTest::default();
    let state = Operand::Arg(1);
    let zero = expected.lit_int(0);
    let one = expected.lit_int(1);
    let two = expected.lit_int(2);
    let five = expected.lit_int(5);

    // loop variables
    let res = Operand::Arg(1);
    let i = Operand::Arg(2);
    let n = Operand::Arg(3);

    let ip1 = expected.add(i, one, Type::Int);
    let rpi = expected.add(res, i, Type::Int);
    let pred = expected.lt(ip1, n);
    let theta = expected.theta(
        pred,
        &[state, zero, zero, Operand::Arg(0)],
        &[
            Operand::Arg(0), // state = state
            rpi,             // res = res + i
            ip1,             // i = i + 1
            n,               // n = n
        ],
    );
    let state = Operand::Project(0, theta);
    let res = Operand::Project(1, theta);
    let pred = expected.lt(res, five);
    let mul2 = expected.mul(Operand::Arg(1), two, Type::Int);
    let gamma = expected.gamma(
        pred,
        &[state, res],
        &[
            &[Operand::Arg(0), Operand::Arg(1)],
            &[Operand::Arg(0), mul2],
        ],
    );
    let expected = expected.into_function(
        "main".to_owned(),
        vec![Type::Int],
        Some((Type::Int, Operand::Project(1, gamma))),
        Some(Operand::Project(0, gamma)),
    );

    // test correctness of RVSDGs converted from CFG
    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let actual = &cfg_to_rvsdg(&cfg).unwrap().functions[0];
    assert!(deep_equal(&expected, actual));

    // test equalties of egglog programs generated by RVSDG
    let actual = actual.to_egglog_expr();

    let actual_command =
        egglog::ast::Command::Action(egglog::ast::Action::Let("actual".into(), actual.clone()));
    const EGGLOG_PROGRAM: &str = r#"
    (let loop
        (Theta
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (IntT) (Arg 2)
                                                   (Node (PureOp (Const (IntT)
                                                                        (const)
                                                                        (Num 1)))))))
                                (Arg 3))))
              (VO (vec-of (Arg 1)
                      (Node (PureOp (Const (IntT) (const) (Num 0))))
                      (Node (PureOp (Const (IntT) (const) (Num 0))))
                      (Arg 0)))
              (VO (vec-of (Arg 0)
                      (Node (PureOp (badd (IntT) (Arg 1) (Arg 2))))
                      (Node (PureOp (badd (IntT) (Arg 2)
                                         (Node (PureOp (Const (IntT) (const) (Num 1)))))))
                      (Arg 3)))))
    (let rescaled 
        (Gamma
         (Node
          (PureOp
           (blt (BoolT) (Project 1 loop)
               (Node (PureOp (Const (IntT) (const) (Num 5)))))))
         (VO (vec-of
          (Project 0 loop)
          (Project 1 loop)))
         (VVO (vec-of (VO (vec-of (Arg 0) (Arg 1)))
                 (VO (vec-of (Arg 0)
                             (Node (PureOp (bmul (IntT) (Arg 1)
                                                (Node (PureOp (Const (IntT)
                                                                     (const)
                                                                     (Num 2)))))))))))))
    (let expected-result (Project 1 rescaled))
    (let expected-state (Project 0 rescaled))
    (let expected (Func "main" (vec-of (Bril (IntT)) (PrintState)) (vec-of (Bril (IntT)) (PrintState)) (VO (vec-of expected-result expected-state))))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_PROGRAM).unwrap();
    egraph.run_program(vec![actual_command]).unwrap();
    egraph
        .parse_and_run_program("(check (= expected actual))")
        .unwrap();

    // test correctness of RVSDG from egglog

    // TODO types don't work out now that we convert terms
    let mut termdag = TermDag::default();
    let actual_term = termdag.expr_to_term(&actual);
    let actual_rvsdg = RvsdgFunction::egglog_term_to_function(actual_term, &termdag);
    assert!(deep_equal(&expected, &actual_rvsdg));
}

fn search_for(f: &RvsdgFunction, mut pred: impl FnMut(&RvsdgBody) -> bool) -> bool {
    fn search_op(
        f: &RvsdgFunction,
        op: &Operand,
        pred: &mut impl FnMut(&RvsdgBody) -> bool,
    ) -> bool {
        match op {
            Operand::Arg(_) => false,
            Operand::Id(x) | Operand::Project(_, x) => search_node(f, &f.nodes[*x], pred),
        }
    }
    fn search_node(
        f: &RvsdgFunction,
        node: &RvsdgBody,
        pred: &mut impl FnMut(&RvsdgBody) -> bool,
    ) -> bool {
        if pred(node) {
            return true;
        }
        match node {
            RvsdgBody::BasicOp(x) => match x {
                BasicExpr::Op(_, args, _)
                | BasicExpr::Call(_, args, _, _)
                | BasicExpr::Print(args) => args.iter().any(|arg| search_op(f, arg, pred)),
                BasicExpr::Const(_, _, _) => false,
            },
            RvsdgBody::Gamma {
                pred: p,
                inputs,
                outputs,
            } => {
                search_op(f, p, pred)
                    || inputs.iter().any(|arg| search_op(f, arg, pred))
                    || outputs
                        .iter()
                        .any(|outs| outs.iter().any(|arg| search_op(f, arg, pred)))
            }
            RvsdgBody::Theta {
                pred: p,
                inputs,
                outputs,
            } => {
                search_op(f, p, pred)
                    || inputs.iter().any(|arg| search_op(f, arg, pred))
                    || outputs.iter().any(|arg| search_op(f, arg, pred))
            }
        }
    }
    f.results
        .iter()
        .any(|(_ty, res)| search_op(f, res, &mut pred))
}

/// We don't want to commit to the order in which nodes are laid out, so we do a
/// DFS to check if two functions are equal for the purposes of testing.
fn deep_equal(f1: &RvsdgFunction, f2: &RvsdgFunction) -> bool {
    if f1.args != f2.args {
        return false;
    }

    fn ops_equal(o1: &Operand, o2: &Operand, f1: &RvsdgFunction, f2: &RvsdgFunction) -> bool {
        match (o1, o2) {
            (Operand::Arg(x), Operand::Arg(y)) => x == y,
            (Operand::Project(p1, l), Operand::Project(p2, r)) => {
                p1 == p2 && ids_equal(*l, *r, f1, f2)
            }
            (Operand::Id(l), Operand::Id(r))
            | (Operand::Project(0, l), Operand::Id(r))
            | (Operand::Id(l), Operand::Project(0, r)) => ids_equal(*l, *r, f1, f2),
            (Operand::Arg(_), Operand::Id(_))
            | (Operand::Arg(_), Operand::Project(_, _))
            | (Operand::Id(_), Operand::Arg(_))
            | (Operand::Project(_, _), Operand::Arg(_))
            | (Operand::Project(_, _), Operand::Id(_))
            | (Operand::Id(_), Operand::Project(_, _)) => false,
        }
    }

    fn all_equal(
        ops1: &[Operand],
        ops2: &[Operand],
        f1: &RvsdgFunction,
        f2: &RvsdgFunction,
    ) -> bool {
        ops1.len() == ops2.len()
            && ops1
                .iter()
                .zip(ops2.iter())
                .all(|(l, r)| ops_equal(l, r, f1, f2))
    }

    fn ids_equal(i1: Id, i2: Id, f1: &RvsdgFunction, f2: &RvsdgFunction) -> bool {
        match (&f1.nodes[i1], &f2.nodes[i2]) {
            (RvsdgBody::BasicOp(l), RvsdgBody::BasicOp(r)) => match (l, r) {
                (BasicExpr::Op(vo1, as1, ty1), BasicExpr::Op(vo2, as2, ty2)) => {
                    vo1 == vo2 && all_equal(as1, as2, f1, f2) && ty1 == ty2
                }
                (BasicExpr::Call(func1, as1, n1, ty1), BasicExpr::Call(func2, as2, n2, ty2)) => {
                    func1 == func2 && n1 == n2 && all_equal(as1, as2, f1, f2) && ty1 == ty2
                }
                (BasicExpr::Const(c1, ty1, lit1), BasicExpr::Const(c2, ty2, lit2)) => {
                    c1 == c2 && ty1 == ty2 && lit1 == lit2
                }
                (BasicExpr::Print(as1), BasicExpr::Print(as2)) => all_equal(as1, as2, f1, f2),
                (BasicExpr::Call(..), BasicExpr::Op(..))
                | (BasicExpr::Call(..), BasicExpr::Const(..))
                | (BasicExpr::Call(..), BasicExpr::Print(..))
                | (BasicExpr::Const(..), BasicExpr::Call(..))
                | (BasicExpr::Const(..), BasicExpr::Op(..))
                | (BasicExpr::Const(..), BasicExpr::Print(..))
                | (BasicExpr::Op(..), BasicExpr::Call(..))
                | (BasicExpr::Op(..), BasicExpr::Const(..))
                | (BasicExpr::Op(..), BasicExpr::Print(..))
                | (BasicExpr::Print(..), BasicExpr::Call(..))
                | (BasicExpr::Print(..), BasicExpr::Const(..))
                | (BasicExpr::Print(..), BasicExpr::Op(..)) => false,
            },
            (
                RvsdgBody::Theta {
                    pred: p1,
                    inputs: is1,
                    outputs: os1,
                },
                RvsdgBody::Theta {
                    pred: p2,
                    inputs: is2,
                    outputs: os2,
                },
            ) => {
                ops_equal(p1, p2, f1, f2)
                    && all_equal(is1, is2, f1, f2)
                    && all_equal(os1, os2, f1, f2)
            }
            (
                RvsdgBody::Gamma {
                    pred: p1,
                    inputs: is1,
                    outputs: os1,
                },
                RvsdgBody::Gamma {
                    pred: p2,
                    inputs: is2,
                    outputs: os2,
                },
            ) => {
                if !ops_equal(p1, p2, f1, f2) || !all_equal(is1, is2, f1, f2) {
                    return false;
                }
                os1.len() == os2.len()
                    && os1
                        .iter()
                        .zip(os2.iter())
                        .all(|(l, r)| all_equal(l, r, f1, f2))
            }
            (RvsdgBody::BasicOp(_), RvsdgBody::Gamma { .. })
            | (RvsdgBody::BasicOp(_), RvsdgBody::Theta { .. })
            | (RvsdgBody::Gamma { .. }, RvsdgBody::Theta { .. })
            | (RvsdgBody::Gamma { .. }, RvsdgBody::BasicOp(_))
            | (RvsdgBody::Theta { .. }, RvsdgBody::BasicOp(_))
            | (RvsdgBody::Theta { .. }, RvsdgBody::Gamma { .. }) => false,
        }
    }

    return f1.results.len() == f2.results.len()
        && f1
            .results
            .iter()
            .zip(f2.results.iter())
            .all(|((t1, o1), (t2, o2))| t1 == t2 && ops_equal(o1, o2, f1, f2));
}

#[test]
fn rvsdg_subst() {
    const EGGLOG_PROGRAM: &str = r#"
    (let unsubsted
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (IntT) (Arg 2)
                                                   (Node (PureOp (Const (IntT)
                                                                        (const)
                                                                        (Num 1)))))))
                                (Arg 3)))))
    (let substed (SubstOperand unsubsted 3 (Arg 7)))
    (run-schedule (saturate (saturate fast-analyses) subst))
    (let expected
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (IntT) (Arg 2)
                                                   (Node (PureOp (Const (IntT)
                                                                        (const)
                                                                        (Num 1)))))))
                                (Arg 7)))))
    (check (= substed expected))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_PROGRAM).unwrap();

    const EGGLOG_THETA_PROGRAM: &str = r#"
    (let unsubsted
        (Theta
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (IntT) (Arg 2)
                                                   (Node (PureOp (Const (IntT)
                                                                        (const)
                                                                        (Num 1)))))))
                                (Arg 3))))
              (VO (vec-of (Arg 0)
                      (Node (PureOp (Const (IntT) (const) (Num 0))))
                      (Node (PureOp (Const (IntT) (const) (Num 0))))
                      (Arg 1)))
              (VO (vec-of (Arg 0)
                      (Node (PureOp (badd (IntT) (Arg 1) (Arg 2))))
                      (Node (PureOp (badd (IntT) (Arg 2)
                                         (Node (PureOp (Const (IntT) (const) (Num 1)))))))
                      (Arg 3)))))
    (let substed (SubstBody unsubsted 1 (Arg 7)))
    (run-schedule (saturate (saturate fast-analyses) subst))
    (let expected
        (Theta
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (IntT) (Arg 2)
                                                   (Node (PureOp (Const (IntT)
                                                                        (const)
                                                                        (Num 1)))))))
                                (Arg 3))))
              (VO (vec-of (Arg 0)
                      (Node (PureOp (Const (IntT) (const) (Num 0))))
                      (Node (PureOp (Const (IntT) (const) (Num 0))))
                      (Arg 7)))
              (VO (vec-of (Arg 0)
                      (Node (PureOp (badd (IntT) (Arg 1) (Arg 2))))
                      (Node (PureOp (badd (IntT) (Arg 2)
                                         (Node (PureOp (Const (IntT) (const) (Num 1)))))))
                      (Arg 3)))))
    (check (= substed expected))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_THETA_PROGRAM).unwrap();

    const EGGLOG_GAMMA_PROGRAM: &str = r#"
    (let unsubsted 
        (Gamma
         (Node
          (PureOp
           (blt (BoolT) (Arg 0) (Arg 0))))
         (VO (vec-of
          (Arg 1)
          (Arg 0)))
         (VVO (vec-of (VO (vec-of (Arg 0) (Arg 1)))
                 (VO (vec-of (Arg 0)
                             (Node (PureOp (bmul (IntT) (Arg 1)
                                                (Node (PureOp (Const (IntT)
                                                                     (const)
                                                                     (Num 2)))))))))))))
    (let substed (SubstBody unsubsted 0 (Arg 7)))
    (run-schedule (saturate (saturate fast-analyses) subst))
    (let expected
        (Gamma
         (Node
          (PureOp
           (blt (BoolT) (Arg 7) (Arg 7))))
         (VO (vec-of
          (Arg 1)
          (Arg 7)))
         (VVO (vec-of (VO (vec-of (Arg 0) (Arg 1)))
                 (VO (vec-of (Arg 0)
                             (Node (PureOp (bmul (IntT) (Arg 1)
                                                (Node (PureOp (Const (IntT)
                                                                     (const)
                                                                     (Num 2)))))))))))))
    (check (= substed expected))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_GAMMA_PROGRAM).unwrap();
}

#[test]
fn rvsdg_subst_beneath_theta() {
    const EGGLOG_THETA_PROGRAM: &str = r#"
    (let inputs
        (VO (vec-of
            (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 3))))
            (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
        )))

    (let unsubsted
        (Theta
              (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
              inputs
              (VO (vec-of
                (Node (PureOp (blt (BoolT)
                    (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                    (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))))))
                (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
              ))
            ))

    (can-subst-Operand-beneath
        (ThetaCtx inputs)
        (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
        (Arg 0))
    (run-schedule (saturate (saturate fast-analyses) subst-beneath))

    (let expected
        (Theta
              (Arg 0)
              (VO (vec-of
                (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 3))))
                (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
              ))
              (VO (vec-of
                (Node (PureOp (blt (BoolT)
                    (Arg 0)
                    (Arg 0))))
                (Node (PureOp (blt (BoolT) (Arg 0) (Arg 4))))
              ))
            ))
    (check (= unsubsted expected))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_THETA_PROGRAM).unwrap();
}

#[test]
fn rvsdg_subst_beneath_gamma() {
    const EGGLOG_GAMMA_PROGRAM: &str = r#"
    (let inputs
        (VO (vec-of
            (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 3))))
            (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
        )))

    (let unsubsted
        (Gamma
            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
            inputs
            (VVO (vec-of
                (VO (vec-of
                    (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                    (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                ))
                (VO (vec-of
                    (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                    (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                ))
            ))
          ))

    (can-subst-Operand-beneath
        (GammaCtx inputs)
        (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
        (Arg 0))
    (run-schedule (saturate (saturate fast-analyses) subst-beneath))

    (let expected
        (Gamma
            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
            (VO (vec-of
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 3))))
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
            ))
            (VVO (vec-of
                (VO (vec-of
                    (Arg 0)
                    (Node (PureOp (blt (BoolT) (Arg 0) (Arg 4))))
                ))
                (VO (vec-of
                    (Arg 0)
                    (Node (PureOp (blt (BoolT) (Arg 0) (Arg 4))))
                ))
            ))
          ))
    (check (= unsubsted expected))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_GAMMA_PROGRAM).unwrap();
}

#[test]
fn rvsdg_subst_beneath_inner_gamma_theta() {
    // This tests what happens when Gamma/Theta appears *within* above
    const EGGLOG_OPERAND_GROUP_PROGRAM: &str = r#"
    (let unsubsted
        (Theta
            (Arg 0)
            (VO (vec-of (Arg 0) (Arg 1) (Arg 2)(Arg 3) (Arg 4) (Arg 5)))
            (VO (vec-of
                (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                (Node (Theta
                      (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                      (VO (vec-of
                        (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 3))))
                        (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                      ))
                      (VO (vec-of
                        (Node (PureOp (blt (BoolT)
                            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))))))
                        (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                      ))
                    ))
                (Node (Gamma
                    (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                    (VO (vec-of
                      (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 3))))
                      (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                    ))
                    (VVO (vec-of
                        (VO (vec-of
                            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                            (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                        ))
                        (VO (vec-of
                            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                            (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                        ))
                    ))
                  ))
                ))))

    (can-subst-Operand-beneath
        (ThetaCtx
            (VO (vec-of (Arg 0) (Arg 1) (Arg 2)(Arg 3) (Arg 4) (Arg 5))))
        (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
        (Arg 0))
    (run-schedule (saturate (saturate fast-analyses) subst-beneath))

    (let expected
        (Theta
            (Arg 0)
            (VO (vec-of (Arg 0) (Arg 1) (Arg 2)(Arg 3) (Arg 4) (Arg 5)))
            (VO (vec-of
                (Arg 0)
                (Node (Theta
                      (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                      (VO (vec-of
                        (Node (PureOp (blt (BoolT) (Arg 0) (Arg 3))))
                        (Node (PureOp (blt (BoolT) (Arg 0) (Arg 4))))
                      ))
                      (VO (vec-of
                        (Node (PureOp (blt (BoolT)
                            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))))))
                        (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                      ))
                    ))
                (Node (Gamma
                    (Arg 0)
                    (VO (vec-of
                      (Node (PureOp (blt (BoolT) (Arg 0) (Arg 3))))
                      (Node (PureOp (blt (BoolT) (Arg 0) (Arg 4))))
                    ))
                    (VVO (vec-of
                        (VO (vec-of
                            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                            (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                        ))
                        (VO (vec-of
                            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                            (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                        ))
                    ))
                  ))
                ))))
    (check (= unsubsted expected))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph
        .parse_and_run_program(EGGLOG_OPERAND_GROUP_PROGRAM)
        .unwrap();
}

#[test]
fn rvsdg_shift() {
    const EGGLOG_PROGRAM: &str = r#"
    (let unshifted
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (IntT) (Arg 2)
                                                   (Node (PureOp (Const (IntT)
                                                                        (const)
                                                                        (Num 1)))))))
                                (Arg 3)))))
    (let shifted (ShiftOperand unshifted 2 4))
    (run-schedule (saturate shift))
    (let expected
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (IntT) (Arg 2)
                                                   (Node (PureOp (Const (IntT)
                                                                        (const)
                                                                        (Num 1)))))))
                                (Arg 7)))))
    (check (= shifted expected))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_PROGRAM).unwrap();

    const EGGLOG_THETA_PROGRAM: &str = r#"
    (let unshifted
        (Theta
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (IntT) (Arg 2)
                                                   (Node (PureOp (Const (IntT)
                                                                        (const)
                                                                        (Num 1)))))))
                                (Arg 3))))
              (VO (vec-of (Arg 0)
                      (Node (PureOp (Const (IntT) (const) (Num 0))))
                      (Node (PureOp (Const (IntT) (const) (Num 0))))
                      (Arg 1)))
              (VO (vec-of (Arg 0)
                      (Node (PureOp (badd (IntT) (Arg 1) (Arg 2))))
                      (Node (PureOp (badd (IntT) (Arg 2)
                                         (Node (PureOp (Const (IntT) (const) (Num 1)))))))
                      (Arg 3)))))
    (let shifted (ShiftBody unshifted 0 10))
    (run-schedule (saturate shift))
    (let expected
        (Theta
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (IntT) (Arg 2)
                                                   (Node (PureOp (Const (IntT)
                                                                        (const)
                                                                        (Num 1)))))))
                                (Arg 3))))
              (VO (vec-of (Arg 0)
                      (Node (PureOp (Const (IntT) (const) (Num 0))))
                      (Node (PureOp (Const (IntT) (const) (Num 0))))
                      (Arg 11)))
              (VO (vec-of (Arg 0)
                      (Node (PureOp (badd (IntT) (Arg 1) (Arg 2))))
                      (Node (PureOp (badd (IntT) (Arg 2)
                                         (Node (PureOp (Const (IntT) (const) (Num 1)))))))
                      (Arg 3)))))
    (check (= shifted expected))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_THETA_PROGRAM).unwrap();

    const EGGLOG_GAMMA_PROGRAM: &str = r#"
    (let unshifted 
        (Gamma
         (Node
          (PureOp
           (blt (BoolT) (Arg 0) (Arg 1))))
         (VO (vec-of
          (Arg 3)
          (Arg 0)))
         (VVO (vec-of (VO (vec-of (Arg 0) (Arg 1)))
                 (VO (vec-of (Arg 0)
                             (Node (PureOp (bmul (IntT) (Arg 1)
                                                (Node (PureOp (Const (IntT)
                                                                     (const)
                                                                     (Num 2)))))))))))))
    (let shifted (ShiftBody unshifted 0 10))
    (run-schedule (saturate shift))
    (let expected
        (Gamma
         (Node
          (PureOp
           (blt (BoolT) (Arg 0) (Arg 11))))
         (VO (vec-of
          (Arg 13)
          (Arg 0)))
         (VVO (vec-of (VO (vec-of (Arg 0) (Arg 1)))
                 (VO (vec-of (Arg 0)
                             (Node (PureOp (bmul (IntT) (Arg 1)
                                                (Node (PureOp (Const (IntT)
                                                                     (const)
                                                                     (Num 2)))))))))))))
    (check (= shifted expected))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_GAMMA_PROGRAM).unwrap();
}

#[test]
fn is_pure() {
    const EGGLOG_PROGRAM: &str = r#"
    (let pure-gamma
        (Gamma
            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
            (VO (vec-of
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 3))))
              (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
            ))
            (VVO (vec-of
                (VO (vec-of
                    (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                    (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                ))
                (VO (vec-of
                    (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
                    (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 4))))
                ))
            ))
          ))
    (run-schedule (saturate fast-analyses))
    (check (Operand-is-pure (Arg 1)))
    (check (Expr-is-pure (badd (BoolT) (Arg 1) (Arg 2))))
    (check (Body-is-pure (PureOp (badd (BoolT) (Arg 1) (Arg 2)))))
    (check (Operand-is-pure (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))))
    (check (Body-is-pure pure-gamma))

    (let output1
        (VO (vec-of
            (Node (PureOp (badd (BoolT) (Arg 0) (Arg 1))))
            (Node (PureOp (PRINT
                (Node (PureOp (badd (BoolT) (Arg 0) (Arg 1))))
                (Arg 2)
            )))
        ))
    )
    (let impure-gamma
        (Gamma
            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2))))
            (VO (vec-of
              (Arg 6)
              (Arg 7)
              (Arg 8) ; print edge
            ))
            (VVO (vec-of
                (VO (vec-of
                    (Node (PureOp (badd (BoolT) (Arg 0) (Arg 1))))
                    (Arg 2)
                ))
                output1
            ))
          ))
    (run-schedule (saturate fast-analyses))
    (fail (check (Expr-is-pure
        (PRINT
            (Node (PureOp (badd (BoolT) (Arg 0) (Arg 1))))
            (Arg 2)
        )
    )))
    (fail (check (Body-is-pure
        (PureOp (PRINT
            (Node (PureOp (badd (BoolT) (Arg 0) (Arg 1))))
            (Arg 2)
        ))
    )))
    (fail (check (Operand-is-pure
        (Node (PureOp (PRINT
            (Node (PureOp (badd (BoolT) (Arg 0) (Arg 1))))
            (Arg 2)
        )))
    )))
    (fail (check (Body-is-pure impure-gamma)))
    (check (= 1 (VecOperand-pure-prefix output1)))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_PROGRAM).unwrap();
}

#[test]
fn rvsdg_body_contains_theta() {
    const EGGLOG_THETA_PROGRAM: &str = r#"
    (let theta
        (Theta
              (Node (PureOp (badd (BoolT) (Arg 7) (Arg 8))))
              (VO (vec-of
                (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 3))))
                (Arg 7)
              ))
              (VO (vec-of
                (Node (PureOp (blt (BoolT)
                    (Arg 4)
                    (Node (PureOp (badd (BoolT) (Arg 5) (Arg 6)))))))
                (Arg 4)
              ))
            ))

    (run-schedule (saturate fast-analyses))

    (check (Body-contains-Expr theta -1 (badd (BoolT) (Arg 7) (Arg 8))))
    (fail (check (Body-contains-Expr theta 0 (badd (BoolT) (Arg 7) (Arg 8)))))
    (fail (check (Body-contains-Expr theta any (badd (BoolT) (Arg 1) (Arg 2)))))
    (check (Body-contains-Operand theta 1 (Arg 4)))
    (fail (check (Body-contains-Body theta any theta)))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_THETA_PROGRAM).unwrap();
}

#[test]
fn rvsdg_body_contains_gamma() {
    const EGGLOG_GAMMA_PROGRAM: &str = r#"
    (let gamma
        (Gamma
            (Arg 10)
            (VO (vec-of
              (Arg 11)
              (Arg 12)
            ))
            (VVO (vec-of
                (VO (vec-of
                    (Node (PureOp (badd (BoolT) (Arg 1) (Arg 0))))
                ))
                (VO (vec-of
                    (Arg 0)
                ))
            ))
          ))

    (run-schedule (saturate fast-analyses))

    (check (Body-contains-Expr gamma 0 (badd (BoolT) (Arg 1) (Arg 0))))
    (fail (check (Body-contains-Expr gamma 1 (badd (BoolT) (Arg 1) (Arg 0)))))
    (fail (check (Body-contains-Operand gamma any (Arg 10))))
    (fail (check (Body-contains-Operand gamma any (Arg 11))))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_GAMMA_PROGRAM).unwrap();
}

#[test]
fn rvsdg_body_contains_operand_group() {
    // This also tests what happens when Gamma/Theta appears *within*
    const EGGLOG_OPERAND_GROUP_PROGRAM: &str = r#"
    (let theta-pred
        (Node (PureOp (badd (BoolT) (Arg 7) (Arg 8)))))
    (let theta-input
        (Node (PureOp (blt (BoolT) (Node (PureOp (badd (BoolT) (Arg 1) (Arg 2)))) (Arg 3)))))
    (let theta-output
        (Node (PureOp (blt (BoolT)
            (Arg 4)
            (Node (PureOp (badd (BoolT) (Arg 5) (Arg 6))))))))
    (let theta
        (Theta
            theta-pred
              (VO (vec-of
                theta-input
                (Arg 7)
              ))
              (VO (vec-of
                theta-output
                (Arg 4)
              ))
            ))

    (let gamma-pred (Arg 10))
    (let gamma-input (Arg 11))
    (let gamma-output 
        (Node (PureOp (badd (BoolT) (Arg 1) (Arg 0)))))
    (let gamma
        (Gamma
            gamma-pred
            (VO (vec-of gamma-input (Arg 12)))
            (VVO (vec-of
                (VO (vec-of gamma-output))
                (VO (vec-of (Arg 0)))
            ))
          ))

    (let og
        (OperandGroup (VO (vec-of
            (Node (PureOp (badd (BoolT) (Arg 21) (Arg 20))))
            (Node gamma)
            (Node theta)
            ))))

    (run-schedule (saturate fast-analyses))

    (check (Body-contains-Body og 2 theta))
    (check (Body-contains-Body og 1 gamma))
    (fail (check (Body-contains-Body og any og)))
    (check (Body-contains-Expr og 0 (badd (BoolT) (Arg 21) (Arg 20))))
    ; Should contain Gamma pred and inputs, but not outputs
    (check (Body-contains-Operand og 1 gamma-pred))
    (check (Body-contains-Operand og 1 gamma-input))
    (fail (check (Body-contains-Operand og any gamma-output)))
    ; Should contain Theta inputs, but not pred or outputs 
    (fail (check (Body-contains-Operand og any theta-pred)))
    (check (Body-contains-Operand og 2 theta-input))
    (fail (check (Body-contains-Operand og any theta-output)))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph
        .parse_and_run_program(EGGLOG_OPERAND_GROUP_PROGRAM)
        .unwrap();
}

#[test]
fn test_conditional_invariant_code_motion() {
    const EGGLOG_GAMMA_PROGRAM: &str = r#"
    (let add
        (Node (PureOp (badd (BoolT) (Arg 1) (Arg 0)))))
    (let gamma-inputs
        (VO (vec-of (Arg 7) (Arg 8))))
    (let gamma
        (Gamma
            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 1))))
            gamma-inputs
            (VVO (vec-of
                (VO (vec-of add (Arg 0)))
                (VO (vec-of (Arg 0) add))
            ))
          ))
    
    (run-schedule
        (saturate fast-analyses)
        (run)
        (saturate fast-analyses subst subst-beneath))

    (let new-gamma
        (Gamma
            (Node (PureOp (badd (BoolT) (Arg 1) (Arg 1))))
            (VO (vec-of
                (Arg 7)
                (Arg 8)
                (Node (PureOp (badd (BoolT) (Arg 8) (Arg 7))))
            ))
            (VVO (vec-of
                (VO (vec-of (Arg 2) (Arg 0)))
                (VO (vec-of (Arg 0) (Arg 2)))
            ))
          ))
    
    (check (= gamma new-gamma))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_GAMMA_PROGRAM).unwrap();
}

#[test]
fn test_conditional_invariant_code_motion_2() {
    const EGGLOG_GAMMA_PROGRAM: &str = r#"
    (let add
        (Node (PureOp (badd (BoolT) (Arg 2) (Arg 3)))))
    (let gamma-inputs
        (VO (vec-of (Arg 6) (Arg 7) (Arg 8) (Arg 9))))
    (let gamma
        (Gamma
            (Arg 9)
            gamma-inputs
            (VVO (vec-of
                (VO (vec-of
                    (Arg 0)
                    (Node (PureOp (bmul (BoolT) add add)))))
                (VO (vec-of
                    (Arg 0)
                    (Node (PureOp (bmul (BoolT) add (Arg 1))))))
            ))
          ))
    
    (run-schedule
        (saturate fast-analyses)
        (run)
        (saturate subst)
        (repeat 3 (repeat 5 subst-beneath) (saturate fast-analyses))
    )
    
    (let new-gamma
        (Gamma
            (Arg 9)
            (VO (vec-of
                    (Arg 6) (Arg 7) (Arg 8) (Arg 9)
                    (Node (PureOp (badd (BoolT) (Arg 8) (Arg 9))))))
            (VVO (vec-of
                (VO (vec-of
                    (Arg 0)
                    (Node (PureOp (bmul (BoolT) (Arg 4) (Arg 4))))))
                (VO (vec-of
                    (Arg 0)
                    (Node (PureOp (bmul (BoolT) (Arg 4) (Arg 1))))))
            ))
          ))
    (extract gamma)
    (check (= gamma new-gamma))
    "#;
    let mut egraph = new_rvsdg_egraph();
    println!(
        "{:?}",
        egraph.parse_and_run_program(EGGLOG_GAMMA_PROGRAM).unwrap()
    );
}

#[test]
fn rvsdg_loop_inv_detect_simple() {
    const EGGLOG_THETA_PROGRAM1: &str = r#"
    (let t1 
    (Theta 
        (Node (PureOp 
            (beq (BoolT) 
                (Node (PureOp 
                    (bdiv (IntT) (Arg 2) 
                                (Node (PureOp 
                                    (bmul (IntT) 
                                        (Node (PureOp 
                                            (bsub (IntT) (Arg 1) 
                                                        (Node (PureOp 
                                                            (badd (IntT) (Arg 4) 
                                                                         (Arg 5)))))))
                                                        (Arg 3))))))) 
                                        (Arg 1)))) 
        (VO (vec-of (Arg 0)
                    (Node (PureOp (Const (IntT) (const) (Num 2)))) 
                    (Node (PureOp (Const (IntT) (const) (Num 6)))) 
                    (Node (PureOp (Const (IntT) (const) (Num 3)))) 
                    (Node (PureOp (Const (IntT) (const) (Num 0)))) 
                    (Node (PureOp (Const (IntT) (const) (Num 1)))))) 
        (VO (vec-of 
                (Node (PureOp 
                    (PRINT (Node (PureOp 
                                (bdiv (IntT) (Arg 2) 
                                            (Node (PureOp 
                                                (bmul (IntT) 
                                                    (Node (PureOp 
                                                        (bsub (IntT) (Arg 1)
                                                                    (Node (PureOp 
                                                                        (badd (IntT) (Arg 4) 
                                                                                        (Arg 5))))))) 
                                                                    (Arg 3)))))))
                                            (Arg 0))))
                (Arg 1)
                (Arg 2)
                (Arg 3)
                (Arg 4)
                (Arg 5)))))

    (run-schedule (saturate fast-analyses))

    (fail (check (arg_inv t1 0)))
    (check (arg_inv t1 1))
    (check (arg_inv t1 2))
    (check (arg_inv t1 3))
    (check (arg_inv t1 4))
    (check (arg_inv t1 5))

    (check (= true (is_inv_operand t1 (Arg 1))))
    (check (= true (is_inv_operand t1 (Arg 2))))
    (check (= true (is_inv_operand t1 (Arg 3))))
    (check (= true (is_inv_operand t1 (Arg 4))))
    (check (= true (is_inv_operand t1 (Arg 5))))

    (check (= true (is_inv_body t1 (PureOp (badd (IntT) (Arg 4) (Arg 5))))))

    (check (= true 
            (is_inv_expr 
                t1 
                (bmul (IntT) 
                    (Node (PureOp 
                        (bsub (IntT) (Arg 1)
                                    (Node (PureOp 
                                        (badd (IntT) (Arg 4) 
                                                     (Arg 5)))))))
                    (Arg 3)))))

    (let inv_operand 
        (Node (PureOp 
            (bdiv (IntT) 
                    (Arg 2) 
                    (Node (PureOp 
                        (bmul (IntT) 
                                (Node (PureOp 
                                    (bsub (IntT) (Arg 1)
                                                (Node (PureOp 
                                                    (badd (IntT) (Arg 4) 
                                                                (Arg 5))))))) 
                                (Arg 3))))))))
    (check (= true (is_inv_operand t1 inv_operand)))


    ; the operand at pred of theta is invariant
    (check (= true (is_inv_operand t1 (Node (PureOp (beq (BoolT) inv_operand (Arg 1)))))))

    ; print is not invariant
    (check (= false (is_inv_operand t1 (Node (PureOp (PRINT (Node (PureOp (bdiv (IntT) (Arg 2) 
    (Node (PureOp (bmul (IntT) (Node (PureOp (bsub (IntT) (Arg 1)
                                                        (Node (PureOp (badd (IntT) (Arg 4) 
                                                                                    (Arg 5))))))) 
                                (Arg 3))))))) 
    (Arg 0)))))))

    (check (= false (is_inv_expr 
                        t1 
                        (PRINT 
                            (Node (PureOp 
                                (bdiv (IntT) (Arg 2) 
                                            (Node (PureOp 
                                                (bmul (IntT) 
                                                    (Node (PureOp 
                                                        (bsub (IntT) 
                                                            (Arg 1)
                                                            (Node (PureOp 
                                                                (badd (IntT) 
                                                                    (Arg 4) 
                                                                    (Arg 5))))))) 
                                                    (Arg 3)))))))
                                            (Arg 0)))))


    (check (= false 
            (is_inv_operand 
                t1 
                (Node (PureOp 
                    (PRINT 
                        (Node (PureOp 
                            (bdiv (IntT) 
                                (Arg 2) 
                                (Node (PureOp 
                                    (bmul (IntT) 
                                        (Node (PureOp 
                                            (bsub (IntT) 
                                                (Arg 1)
                                                (Node (PureOp
                                                    (badd (IntT) 
                                                        (Arg 4) 
                                                        (Arg 5))))))) 
                                        (Arg 3))))))) 
                            (Arg 0)))))))

    ;; an expr that does not exist in original program should fail check
    (fail (check (is_inv_expr t1 (badd (IntT) (Arg 1) (Arg 2)))))
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_THETA_PROGRAM1).unwrap();

    const EGGLOG_THETA_PROGRAM2: &str = r#"
    (let t1 (Theta 
        (Node (PureOp 
            (blt (BoolT) 
                (Node (PureOp 
                    (badd (IntT) 
                        (Node (PureOp 
                            (Const (IntT) (const) (Num 1)))) 
                            (Arg 1)))) 
                        (Node (PureOp 
                            (Const (IntT) (const) (Num 5))))))) 
        (VO (vec-of (Arg 0) 
                    (Node (PureOp (Const (IntT) (const) (Num 0)))) 
                    (Node (PureOp (Const (IntT) (const) (Num 10)))) 
                    (Node (PureOp (Const (IntT) (const) (Num 10)))) 
                    (Node (PureOp (Const (IntT) (const) (Num 10))))))
        (VO (vec-of 
            (Node (PureOp 
                (PRINT 
                    (Node (PureOp 
                        (bmul (IntT) 
                            (Arg 2) 
                            (Node (PureOp 
                                (Const (IntT) (const) (Num 2))))))) 
                    (Node (PureOp 
                        (PRINT 
                            (Project 0 
                                (PureOp 
                                    (Call 
                                        (SomeType (IntT)) 
                                        "mean3" 
                                        (VO 
                                            (vec-of 
                                                (Node (PureOp 
                                                    (badd (IntT) 
                                                        (Arg 4) 
                                                        (Node (PureOp 
                                                            (Const (IntT) (const) (Num 5)))))))

                                                (Node (PureOp 
                                                    (bsub (IntT) 
                                                        (Arg 3) 
                                                        (Node (PureOp 
                                                            (Const (IntT) (const) (Num 3))))))) 
                                                (Node (PureOp 
                                                    (bsub (IntT) 
                                                        (Node (PureOp 
                                                            (bsub (IntT) (Arg 3) 
                                                                        (Node (PureOp 
                                                                            (Const (IntT) (const) (Num 3))))))) 
                                                                        (Node (PureOp 
                                                                            (Const (IntT) (const) (Num 2)))))))
                                                (Arg 0)))
                                            2))) 
            (Project 1 
                (PureOp 
                    (Call 
                        (SomeType (IntT)) 
                        "mean3" 
                        (VO 
                            (vec-of 
                                (Node (PureOp 
                                    (badd (IntT) 
                                        (Arg 4) 
                                        (Node (PureOp 
                                            (Const (IntT) (const) (Num 5)))))))

                                (Node (PureOp 
                                    (bsub (IntT) 
                                        (Arg 3) 
                                        (Node (PureOp 
                                            (Const (IntT) (const) (Num 3))))))) 
                                (Node (PureOp 
                                    (bsub (IntT) 
                                        (Node (PureOp 
                                            (bsub (IntT) (Arg 3) 
                                                        (Node (PureOp 
                                                            (Const (IntT) (const) (Num 3))))))) 
                                                        (Node (PureOp 
                                                            (Const (IntT) (const) (Num 2)))))))
                                (Arg 0)))
                            2)))))))))
                (Node (PureOp (badd (IntT) (Node (PureOp (Const (IntT) (const) (Num 1)))) (Arg 1))))
                (Node (PureOp (bmul (IntT) (Arg 2) (Node (PureOp (Const (IntT) (const) (Num 2))))))) 
                (Arg 3) 
                (Arg 4)))))

    (run-schedule
        (repeat 5 (run) (saturate fast-analyses)))

    (check (= true (is_inv_operand t1 (Arg 3))))
    (check (= true (is_inv_operand t1 (Arg 4))))
    (check (= false (is_inv_operand t1 (Arg 0))))
    (check (= false (is_inv_operand t1 (Arg 1))))
    (check (= false (is_inv_operand t1 (Arg 2))))
    (check (= true (is_inv_operand t1 (Node (PureOp (Const (IntT) (const) (Num 5)))))))
    (check (= true 
            (is_inv_expr 
                t1 
                (badd (IntT) 
                    (Arg 4) 
                    (Node (PureOp (Const (IntT) (const) (Num 5))))))))
    (check (= true 
            (is_inv_operand 
                t1 
                (Node (PureOp 
                    (bsub (IntT) 
                        (Node (PureOp (
                            bsub (IntT) (Arg 3) 
                                        (Node (PureOp (Const (IntT) (const) (Num 3))))))) 
                        (Node (PureOp (Const (IntT) (const) (Num 2))))))))))
                        (check (= false 
                            (is_inv_body
                                t1 
                                (PureOp 
                                    (Call 
                                        (SomeType (IntT)) 
                                        "mean3" 
                                        (VO (vec-of 
                                            (Node (PureOp 
                                                (badd (IntT) 
                                                    (Arg 4) 
                                                    (Node (PureOp 
                                                        (Const (IntT) (const) (Num 5))))))) 
                                            (Node (PureOp 
                                                (bsub (IntT) 
                                                    (Arg 3) 
                                                    (Node (PureOp 
                                                        (Const (IntT) (const) (Num 3))))))) 
                                            (Node (PureOp 
                                                (bsub (IntT) 
                                                    (Node (PureOp 
                                                        (bsub (IntT) 
                                                            (Arg 3) 
                                                            (Node (PureOp 
                                                                (Const (IntT) (const) (Num 3))))))) 
                                            (Node (PureOp (Const (IntT) (const) (Num 2))))))) 
                                            (Arg 0))) 
                                        2)))))
                                                                
    "#;
    let mut egraph = new_rvsdg_egraph();
    egraph.parse_and_run_program(EGGLOG_THETA_PROGRAM2).unwrap();
}
