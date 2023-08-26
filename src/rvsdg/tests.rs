use bril_rs::{ConstOps, Literal, Type, ValueOps};

use crate::{
    cfg::to_cfg,
    rvsdg::{from_cfg::to_rvsdg, Expr, Id, Operand, RvsdgBody},
    util::{parse_from_string, DebugVisualizations},
};

use super::RvsdgFunction;

/// Utility struct for building an RVSDG.
#[derive(Default)]
struct RvsdgTest {
    nodes: Vec<RvsdgBody>,
}

impl RvsdgTest {
    fn into_function(self, n_args: usize, output: Operand) -> RvsdgFunction {
        RvsdgFunction {
            n_args,
            nodes: self.nodes,
            result: Some(output),
        }
    }

    fn lit_int(&mut self, i: i64) -> Operand {
        self.make_node(RvsdgBody::PureOp(Expr::Const(
            ConstOps::Const,
            Type::Int,
            Literal::Int(i),
        )))
    }

    fn lit_bool(&mut self, b: bool) -> Operand {
        self.make_node(RvsdgBody::PureOp(Expr::Const(
            ConstOps::Const,
            Type::Bool,
            Literal::Bool(b),
        )))
    }

    fn lt(&mut self, l: Operand, r: Operand) -> Operand {
        self.make_node(RvsdgBody::PureOp(Expr::Op(ValueOps::Lt, vec![l, r])))
    }

    fn add(&mut self, l: Operand, r: Operand) -> Operand {
        self.make_node(RvsdgBody::PureOp(Expr::Op(ValueOps::Add, vec![l, r])))
    }

    fn mul(&mut self, l: Operand, r: Operand) -> Operand {
        self.make_node(RvsdgBody::PureOp(Expr::Op(ValueOps::Mul, vec![l, r])))
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
    let mut cfg = to_cfg(&prog.functions[0]);
    let rvsdg = to_rvsdg(&mut cfg).unwrap();

    let mut expected = RvsdgTest::default();
    let one = expected.lit_int(1);
    let two = expected.lit_int(2);
    let res = expected.add(one, two);
    assert!(deep_equal(&expected.into_function(0, res), &rvsdg));
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
    DebugVisualizations::new(PROGRAM)
        .write_output("/tmp/unstructured_")
        .unwrap();
    let prog = parse_from_string(PROGRAM);
    let mut cfg = to_cfg(&prog.functions[0]);
    let rvsdg = to_rvsdg(&mut cfg).unwrap();
    // This example is a bit less natural, and while I believe this is a
    // faithful RVSDG, it'd be nicer to get further assurance that this is
    // correct (e.g. by roundtripping this to bril and ensuring the same values
    // came out).
    let mut expected = RvsdgTest::default();
    let four = expected.lit_int(4);
    let one = expected.lit_int(1);
    let zero = expected.lit_int(0);

    let pred = expected.lt(four, four);
    let gamma1 = expected.gamma(pred, &[], &[&[four, one], &[four, zero]]);

    // loop body:

    let pred2 = expected.lt(Operand::Arg(0), one);
    let in0 = expected.add(Operand::Arg(0), one);
    let gamma_inner0 = expected.gamma(
        pred2,
        &[in0, Operand::Arg(1)],
        &[
            &[Operand::Arg(0), zero, Operand::Arg(1)],
            &[Operand::Arg(0), one, one],
        ],
    );

    let gamma_outer = expected.gamma(
        zero,
        &[Operand::Arg(0), Operand::Arg(1)],
        &[
            &[
                Operand::Project(0, gamma_inner0),
                Operand::Project(1, gamma_inner0),
                Operand::Project(2, gamma_inner0),
            ],
            &[Operand::Arg(0), one, zero],
        ],
    );

    let res = expected.theta(
        Operand::Project(1, gamma_outer),
        &[Operand::Project(0, gamma1), Operand::Project(1, gamma1)],
        &[
            Operand::Project(0, gamma_outer),
            Operand::Project(2, gamma_outer),
        ],
    );

    assert!(deep_equal(
        &expected.into_function(0, Operand::Project(0, res)),
        &rvsdg
    ));
}

#[test]
fn rvsdg_basic_odd_branch() {
    // Bril program summing the numbers from 1 to n, multiplying by 2 if that
    // value is larger is larger than 5. This gives us a theta node and a gamma
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

    let mut expected = RvsdgTest::default();
    let zero = expected.lit_int(0);
    let one = expected.lit_int(1);
    let two = expected.lit_int(2);
    let five = expected.lit_int(5);

    // loop variables
    let res = Operand::Arg(0);
    let i = Operand::Arg(1);
    let n = Operand::Arg(2);

    let ip1 = expected.add(i, one);
    let rpi = expected.add(res, i);
    let pred = expected.lt(ip1, n);
    let theta = expected.theta(
        pred,
        &[zero, zero, Operand::Arg(0)],
        &[
            // res = res + i
            rpi, // i = i + 1
            ip1, // n = n
            n,
        ],
    );
    let res = Operand::Project(0, theta);
    let pred = expected.lt(res, five);
    let mul2 = expected.mul(Operand::Arg(0), two);
    let gamma = expected.gamma(pred, &[res], &[&[Operand::Arg(0)], &[mul2]]);
    let prog = parse_from_string(PROGRAM);
    let mut cfg = to_cfg(&prog.functions[0]);
    let got = to_rvsdg(&mut cfg).unwrap();
    assert!(deep_equal(
        &expected.into_function(1, Operand::Project(0, gamma)),
        &got
    ));
}

/// We don't want to commit to the order in which nodes are laid out, so we do a
/// DFS to check if two functions are equal for the purposes of testing.
fn deep_equal(f1: &RvsdgFunction, f2: &RvsdgFunction) -> bool {
    if f1.n_args != f2.n_args {
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
            (RvsdgBody::PureOp(l), RvsdgBody::PureOp(r)) => match (l, r) {
                (Expr::Op(vo1, as1), Expr::Op(vo2, as2)) => {
                    vo1 == vo2 && all_equal(as1, as2, f1, f2)
                }
                (Expr::Call(func1, as1), Expr::Call(func2, as2)) => {
                    func1 == func2 && all_equal(as1, as2, f1, f2)
                }
                (Expr::Const(c1, ty1, lit1), Expr::Const(c2, ty2, lit2)) => {
                    c1 == c2 && ty1 == ty2 && lit1 == lit2
                }
                (Expr::Call(_, _), Expr::Op(_, _))
                | (Expr::Call(_, _), Expr::Const(_, _, _))
                | (Expr::Const(_, _, _), Expr::Call(_, _))
                | (Expr::Const(_, _, _), Expr::Op(_, _))
                | (Expr::Op(_, _), Expr::Call(_, _))
                | (Expr::Op(_, _), Expr::Const(_, _, _)) => false,
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
            (RvsdgBody::PureOp(_), RvsdgBody::Gamma { .. })
            | (RvsdgBody::PureOp(_), RvsdgBody::Theta { .. })
            | (RvsdgBody::Gamma { .. }, RvsdgBody::Theta { .. })
            | (RvsdgBody::Gamma { .. }, RvsdgBody::PureOp(_))
            | (RvsdgBody::Theta { .. }, RvsdgBody::PureOp(_))
            | (RvsdgBody::Theta { .. }, RvsdgBody::Gamma { .. }) => false,
        }
    }

    match (&f1.result, &f2.result) {
        (Some(o1), Some(o2)) => ops_equal(o1, o2, f1, f2),
        (None, None) | (None, Some(_)) | (Some(_), None) => false,
    }
}
