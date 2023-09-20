use crate::cfg::program_to_cfg;
use crate::peg::{PegBody, PegFunction};
use crate::rvsdg::cfg_to_rvsdg;
use crate::rvsdg::{Expr, Id};
use crate::util::parse_from_string;
use bril_rs::{ConstOps, Literal, Type, ValueOps};
use std::fs::File;
use std::io::Write;

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
        self.make_node(PegBody::BasicOp(Expr::Const(
            ConstOps::Const,
            Literal::Int(i),
            Type::Int,
        )))
    }

    fn lit_bool(&mut self, b: bool) -> Id {
        self.make_node(PegBody::BasicOp(Expr::Const(
            ConstOps::Const,
            Literal::Bool(b),
            Type::Bool,
        )))
    }

    fn lt(&mut self, l: Id, r: Id) -> Id {
        self.make_node(PegBody::BasicOp(Expr::Op(
            ValueOps::Lt,
            vec![l, r],
            Type::Bool,
        )))
    }

    fn add(&mut self, l: Id, r: Id, ty: Type) -> Id {
        self.make_node(PegBody::BasicOp(Expr::Op(ValueOps::Add, vec![l, r], ty)))
    }

    fn mul(&mut self, l: Id, r: Id, ty: Type) -> Id {
        self.make_node(PegBody::BasicOp(Expr::Op(ValueOps::Mul, vec![l, r], ty)))
    }

    fn phi(&mut self, if_: Id, then: Id, else_: Id) -> Id {
        self.make_node(PegBody::Phi(if_, then, else_))
    }

    fn theta(&mut self, init: Id, loop_: Id, label: usize) -> Id {
        self.make_node(PegBody::Theta(init, loop_, label))
    }

    fn arg(&mut self, arg: usize) -> Id {
        self.make_node(PegBody::Arg(arg))
    }

    fn eval(&mut self, l: Id, r: Id, label: usize) -> Id {
        self.make_node(PegBody::Eval(l, r, label))
    }

    fn pass(&mut self, c: Id, label: usize) -> Id {
        self.make_node(PegBody::Pass(c, label))
    }

    fn make_node(&mut self, body: PegBody) -> Id {
        let res = self.nodes.len();
        self.nodes.push(body);
        res
    }
}

fn output_dot_graph(name: &str, peg: &PegFunction) {
    let mut file = File::create(name).unwrap();
    file.write_all(peg.graph().as_bytes()).unwrap();
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
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();
    let peg = PegFunction::new(&rvsdg.functions[0]);

    let mut expected = PegTest::default();
    let one = expected.lit_int(1);
    let two = expected.lit_int(2);
    let res = expected.add(one, two, Type::Int);
    assert_eq!(&expected.into_function(0, res), &peg);
}

#[test]
fn peg_basic_odd_branch() {
    // Bril program summing the numbers from 1 to n, multiplying by 2 if that
    // value is larger is larger than 5. We test this by simulating both a
    // hand-writte and a generated peg.
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

    let mut expected = PegTest::default();
    let zero = expected.lit_int(0);
    let one = expected.lit_int(1);
    let two = expected.lit_int(2);
    let five = expected.lit_int(5);
    let n = expected.arg(0);

    let res = expected.theta(zero, usize::MAX, 0);
    let i = expected.theta(zero, usize::MAX, 0);
    let res_plus = expected.add(i, res, Type::Int);
    let i_plus = expected.add(one, i, Type::Int);
    expected.nodes[res] = PegBody::Theta(zero, res_plus, 0);
    expected.nodes[i] = PegBody::Theta(zero, i_plus, 0);

    let pred = expected.lt(i, n);
    let pass = expected.pass(pred, 0);
    let eval = expected.eval(res, pass, 0);
    let pred = expected.lt(eval, five);
    let mul2 = expected.mul(eval, two, Type::Int);
    let phi = expected.phi(pred, mul2, eval);
    let want = expected.into_function(1, phi);

    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();
    let have = PegFunction::new(&rvsdg.functions[0]);

    let want: Vec<_> = (0..10)
        .map(|i| want.simulate(&[Literal::Int(i)]).unwrap())
        .collect();
    let have: Vec<_> = (0..10)
        .map(|i| have.simulate(&[Literal::Int(i)]).unwrap())
        .collect();
    assert_eq!(want, have);
}

#[test]
fn peg_unstructured() {
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

    // Since this test doesn't take arguments we could hardcode the output,
    // but this also serves as an example for other people writing tests.
    let mut expected = PegTest::default();
    let four = expected.lit_int(4);
    let one = expected.lit_int(1);

    let x = expected.theta(four, usize::MAX, 0);
    let x_plus_one = expected.add(x, one, Type::Int);
    expected.nodes[x] = PegBody::Theta(four, x_plus_one, 0);

    let lt = expected.lt(x, four);
    let pass = expected.pass(lt, 0);
    let eval = expected.eval(x, pass, 0);
    let add = expected.add(eval, one, Type::Int);

    let want = expected.into_function(0, add);

    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();
    let have = PegFunction::new(&rvsdg.functions[0]);

    assert_eq!(want.simulate(&[]).unwrap(), have.simulate(&[]).unwrap());
}
