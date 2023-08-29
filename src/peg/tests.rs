use crate::cfg::to_cfg;
use crate::peg::{PegBody, PegFunction};
use crate::rvsdg::{from_cfg::to_rvsdg, Expr, Id};
use crate::util::parse_from_string;
use bril_rs::{ConstOps, Literal, Type, ValueOps};

/// Utility struct for building an Peg.
#[derive(Default)]
struct PegTest {
    nodes: Vec<PegBody>,
}

impl PegTest {
    fn into_function(self, n_args: usize, output: Id) -> PegFunction {
        PegFunction {
            n_args,
            nodes: self.nodes,
            result: Some(output),
        }
    }

    fn lit_int(&mut self, i: i64) -> Id {
        self.make_node(PegBody::PureOp(Expr::Const(
            ConstOps::Const,
            Type::Int,
            Literal::Int(i),
        )))
    }

    fn lit_bool(&mut self, b: bool) -> Id {
        self.make_node(PegBody::PureOp(Expr::Const(
            ConstOps::Const,
            Type::Bool,
            Literal::Bool(b),
        )))
    }

    fn lt(&mut self, l: Id, r: Id) -> Id {
        self.make_node(PegBody::PureOp(Expr::Op(ValueOps::Lt, vec![l, r])))
    }

    fn add(&mut self, l: Id, r: Id) -> Id {
        self.make_node(PegBody::PureOp(Expr::Op(ValueOps::Add, vec![l, r])))
    }

    fn mul(&mut self, l: Id, r: Id) -> Id {
        self.make_node(PegBody::PureOp(Expr::Op(ValueOps::Mul, vec![l, r])))
    }

    fn phi(&mut self, if_: Id, then: Id, else_: Id) -> Id {
        let res = self.nodes.len();
        self.nodes.push(PegBody::Phi(if_, then, else_));
        res
    }

    fn theta(&mut self, init: Id, loop_: Id, label: usize) -> Id {
        let res = self.nodes.len();
        self.nodes.push(PegBody::Theta(init, loop_, label));
        res
    }

    fn make_node(&mut self, body: PegBody) -> Id {
        let res = self.nodes.len();
        self.nodes.push(body);
        res
    }
}

#[test]
fn peg_expr() {
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
    let peg = PegFunction::new(&to_rvsdg(&mut cfg).unwrap());

    let mut expected = PegTest::default();
    let one = expected.lit_int(1);
    let two = expected.lit_int(2);
    let res = expected.add(one, two);
    assert!(deep_equal(&expected.into_function(0, res), &peg));
}

// todo
// #[test]
// fn peg_unstructured() {
//     const PROGRAM: &str = r#"@main(): int {
//         x: int = const 4;
//         a_cond: bool = lt x x;
//         br a_cond .B .C;
//       .B:
//         a: int = const 1;
//         b_cond: bool = lt x a;
//         x: int = add x a;
//         br b_cond .C .D;
//       .C:
//         jmp .B;
//       .D:
//         ret x;
//       }"#;
//     DebugVisualizations::new(PROGRAM)
//         .write_output("/tmp/unstructured_")
//         .unwrap();
//     let prog = parse_from_string(PROGRAM);
//     let mut cfg = to_cfg(&prog.functions[0]);
//     let peg = to_rvsdg(&mut cfg).unwrap();
//     // This example is a bit less natural, and while I believe this is a
//     // faithful RVSDG, it'd be nicer to get further assurance that this is
//     // correct (e.g. by roundtripping this to bril and ensuring the same values
//     // came out).
//     let mut expected = PegTest::default();
//     let four = expected.lit_int(4);
//     let one = expected.lit_int(1);
//     let zero = expected.lit_int(0);

//     let pred = expected.lt(four, four);
//     let phi1 = expected.phi(pred, &[], &[&[four, one], &[four, zero]]);

//     // loop body:

//     let pred2 = expected.lt(Operand::Arg(0), one);
//     let in0 = expected.add(Operand::Arg(0), one);
//     let phi_inner0 = expected.phi(
//         pred2,
//         &[in0, Operand::Arg(1)],
//         &[
//             &[Operand::Arg(0), zero, Operand::Arg(1)],
//             &[Operand::Arg(0), one, one],
//         ],
//     );

//     let phi_outer = expected.phi(
//         zero,
//         &[Operand::Arg(0), Operand::Arg(1)],
//         &[
//             &[
//                 Operand::Project(0, phi_inner0),
//                 Operand::Project(1, phi_inner0),
//                 Operand::Project(2, phi_inner0),
//             ],
//             &[Operand::Arg(0), one, zero],
//         ],
//     );

//     let res = expected.theta(
//         Operand::Project(1, phi_outer),
//         &[Operand::Project(0, phi1), Operand::Project(1, phi1)],
//         &[
//             Operand::Project(0, phi_outer),
//             Operand::Project(2, phi_outer),
//         ],
//     );

//     assert!(deep_equal(
//         &expected.into_function(0, Operand::Project(0, res)),
//         &peg
//     ));
// }

// todo
// #[test]
// fn peg_basic_odd_branch() {
//     // Bril program summing the numbers from 1 to n, multiplying by 2 if that
//     // value is larger is larger than 5. This gives us a theta node and a phi
//     // node, with the phi requiring branch restructuring.
//     const PROGRAM: &str = r#"
//  @main(n: int): int {
//     res: int = const 0;
//     i: int = const 0;
//  .loop:
//     one: int = const 1;
//     res: int = add res i;
//     i: int = add i one;
//     loop_cond: bool = lt i n;
//     br loop_cond .loop .tail;
//  .tail:
//    five: int = const 5;
//    rescale_cond: bool = lt res five;
//    br rescale_cond .rescale .exit;
//  .rescale:
//    two: int = const 2;
//    res: int = mul res two;
//  .exit:
//   ret res;
// }"#;

//     let mut expected = PegTest::default();
//     let zero = expected.lit_int(0);
//     let one = expected.lit_int(1);
//     let two = expected.lit_int(2);
//     let five = expected.lit_int(5);

//     // loop variables
//     let res = Operand::Arg(0);
//     let i = Operand::Arg(1);
//     let n = Operand::Arg(2);

//     let ip1 = expected.add(i, one);
//     let rpi = expected.add(res, i);
//     let pred = expected.lt(ip1, n);
//     let theta = expected.theta(
//         pred,
//         &[zero, zero, Operand::Arg(0)],
//         &[
//             // res = res + i
//             rpi, // i = i + 1
//             ip1, // n = n
//             n,
//         ],
//     );
//     let res = Operand::Project(0, theta);
//     let pred = expected.lt(res, five);
//     let mul2 = expected.mul(Operand::Arg(0), two);
//     let phi = expected.phi(pred, &[res], &[&[Operand::Arg(0)], &[mul2]]);
//     let prog = parse_from_string(PROGRAM);
//     let mut cfg = to_cfg(&prog.functions[0]);
//     let got = PegFunction::new(&to_rvsdg(&mut cfg).unwrap());
//     assert!(deep_equal(
//         &expected.into_function(1, Operand::Project(0, phi)),
//         &got
//     ));
// }

/// We don't want to commit to the order in which nodes are laid out, so we do a
/// DFS to check if two functions are equal for the purposes of testing.
fn deep_equal(f1: &PegFunction, f2: &PegFunction) -> bool {
    if f1.n_args != f2.n_args {
        return false;
    }

    fn all_equal(ids1: &[Id], ids2: &[Id], f1: &PegFunction, f2: &PegFunction) -> bool {
        ids1.len() == ids2.len()
            && ids1
                .iter()
                .zip(ids2.iter())
                .all(|(l, r)| ids_equal(*l, *r, f1, f2))
    }

    fn labels_equal(_label1: usize, _label2: usize, _f1: &PegFunction, _f2: &PegFunction) -> bool {
        // todo: check that labels are consistent
        true
    }

    fn ids_equal(i1: Id, i2: Id, f1: &PegFunction, f2: &PegFunction) -> bool {
        match (&f1.nodes[i1], &f2.nodes[i2]) {
            (PegBody::PureOp(l), PegBody::PureOp(r)) => match (l, r) {
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
            (PegBody::PureOp(_), _) => false,
            (PegBody::Theta(p1, is1, label1), PegBody::Theta(p2, is2, label2)) => {
                ids_equal(*p1, *p2, f1, f2)
                    && ids_equal(*is1, *is2, f1, f2)
                    && labels_equal(*label1, *label2, f1, f2)
            }
            (PegBody::Theta(..), _) => false,
            (PegBody::Phi(p1, is1, os1), PegBody::Phi(p2, is2, os2)) => {
                if !ids_equal(*p1, *p2, f1, f2) || !ids_equal(*is1, *is2, f1, f2) {
                    return false;
                }
                ids_equal(*os1, *os2, f1, f2)
            }
            (PegBody::Phi(..), _) => false,
            (PegBody::Eval(i1, l1, label1), PegBody::Eval(i2, l2, label2)) => {
                ids_equal(*i1, *i2, f1, f2)
                    && ids_equal(*l1, *l2, f1, f2)
                    && labels_equal(*label1, *label2, f1, f2)
            }
            (PegBody::Eval(..), _) => false,
            (PegBody::Pass(i1, label1), PegBody::Pass(i2, label2)) => {
                ids_equal(*i1, *i2, f1, f2) && labels_equal(*label1, *label2, f1, f2)
            }
            (PegBody::Pass(..), _) => false,
            (PegBody::Arg(a1), PegBody::Arg(a2)) => a1 == a2,
            (PegBody::Arg(_), _) => false,
        }
    }

    match (&f1.result, &f2.result) {
        (Some(o1), Some(o2)) => ids_equal(*o1, *o2, f1, f2),
        (None, None) | (None, Some(_)) | (Some(_), None) => false,
    }
}
