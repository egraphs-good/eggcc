//! Convert RVSDG programs to the dag encoding, adding context nodes as we go.
//! This is a fairly direct translation, with a minor difference being
//! that if nodes do not create a region.
//! When we translate if nodes, we add context nodes at the inputs to the region.
//! Common sub-expressions can still be shared across branches, avoiding blowup from context nodes.
//! We are careful to add context to every leaf node (Empty, Arg, and Const)
use std::iter;

#[cfg(test)]
use crate::{cfg::program_to_cfg, rvsdg::cfg_to_rvsdg, util::parse_from_string};
use dag_in_context::ast::*;
#[cfg(test)]
use dag_in_context::interpreter::Value;
#[cfg(test)]
use dag_in_context::schema::Constant;

use crate::rvsdg::{BasicExpr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram};
use bril_rs::{EffectOps, Literal, ValueOps};
use dag_in_context::{
    ast::{add, call, dowhile, function, int, less_than, program_vec, tfalse, ttrue},
    schema::{RcExpr, TreeProgram, Type},
};
use hashbrown::HashMap;

use super::RvsdgType;

impl RvsdgProgram {
    /// Converts an RVSDG program to the dag encoding.
    /// Common subexpressions are shared by the same Rc<Expr> in the dag encoding.
    /// This invariant is maintained by restore_sharing_invariant.
    /// Also adds context to the program.
    pub fn to_dag_encoding(&self, add_context: bool) -> TreeProgram {
        let last_function = self.functions.last().unwrap();
        let rest_functions = self.functions.iter().take(self.functions.len() - 1);
        let res = program_vec(
            last_function.to_dag_encoding(),
            rest_functions
                .map(|f| f.to_dag_encoding())
                .collect::<Vec<_>>(),
        )
        .restore_sharing_invariant();
        if add_context {
            res.add_context()
        } else {
            res
        }
    }
}

/// We cache a stored value for each node in the RVSDG, representing the corresponding
/// expression in the dag encoding.
/// These can either be tuple typed or single typed, and we need to track this to properly translate projects.
#[derive(Clone, Debug)]
struct StoredValue {
    is_tuple: bool,
    expr: RcExpr,
}

impl StoredValue {
    fn to_single_expr(&self) -> RcExpr {
        if self.is_tuple {
            panic!("Cannot convert tuple to single expr. Got {:?}", self)
        } else {
            self.expr.clone()
        }
    }

    fn project(&self, index: usize) -> StoredValue {
        if self.is_tuple {
            StoredValue {
                is_tuple: false,
                expr: get(self.expr.clone(), index),
            }
        } else {
            assert_eq!(
                index, 0,
                "Tried to access index {} of non-tuple value",
                index
            );
            self.clone()
        }
    }
}

struct DagTranslator<'a> {
    /// `stored_node` is a cache of already translated rvsdg nodes.
    stored_node: HashMap<Id, StoredValue>,
    /// A reference to the nodes in the RVSDG.
    nodes: &'a [RvsdgBody],
    /// The next id to assign to an alloc.
    next_alloc_id: i64,
}

impl<'a> DagTranslator<'a> {
    /// Adds a pure expression to the cache.
    /// Essentially inlines all references to this expression instead of binding it.
    /// Importantly, on translation back to RVSDG we should ensure that pure
    /// common subexpressions are not duplicated.
    fn cache_single(&mut self, expr: RcExpr, id: Id) -> StoredValue {
        let res = StoredValue {
            is_tuple: false,
            expr: expr.clone(),
        };
        self.stored_node.insert(id, res.clone());
        res
    }

    fn tuple_res(&mut self, expr: RcExpr, id: Id) -> StoredValue {
        let res = StoredValue {
            is_tuple: true,
            expr: expr.clone(),
        };
        self.stored_node.insert(id, res.clone());
        res
    }

    /// Translate a sub-region by creating a new region
    /// translator and translating the operands.
    /// Returns the translated expression and the resulting
    /// values, assuming the new expression is let-bound
    /// using `add_region_binding`.
    /// Loops don't return their predicate, so skip
    /// the first output using `skip_outputs`.
    fn translate_subregion(
        &mut self,
        operands: impl Iterator<Item = Operand> + DoubleEndedIterator,
    ) -> RcExpr {
        let resulting_exprs = operands.map(|operand| {
            let res = self.translate_operand(operand);
            res.to_single_expr()
        });

        parallel_vec_nonempty(resulting_exprs)
    }

    fn translate_operand(&mut self, operand: Operand) -> StoredValue {
        match operand {
            Operand::Arg(index) => StoredValue {
                is_tuple: false,
                expr: get(arg(), index),
            },
            Operand::Project(p_index, id) => self.translate_node(id).project(p_index),
        }
    }

    /// Translate a node or return the index of the already-translated node.
    /// For regions, translates the region and returns the index of the
    /// tuple containing the results.
    fn translate_node(&mut self, id: Id) -> StoredValue {
        let node = &self.nodes[id];

        if let Some(cached) = self.stored_node.get(&id) {
            return cached.clone();
        }
        match node {
            RvsdgBody::BasicOp(expr) => self.translate_basic_expr(expr.clone(), id),
            RvsdgBody::If {
                pred,
                inputs,
                then_branch,
                else_branch,
            } => {
                let mut input_values = vec![];
                for input in inputs {
                    input_values.push(self.translate_operand(*input).to_single_expr());
                }
                let input_concat = parallel_vec(input_values.clone());
                let pred = self.translate_operand(*pred).to_single_expr();

                let then_translated = self.translate_subregion(then_branch.iter().copied());
                let else_translated = self.translate_subregion(else_branch.iter().copied());

                let expr = tif(pred, input_concat, then_translated, else_translated);
                self.tuple_res(expr, id)
            }
            RvsdgBody::Gamma {
                pred,
                inputs,
                outputs,
            } => {
                let mut inputs_values = vec![];
                for input in inputs {
                    inputs_values.push(self.translate_operand(*input));
                }

                let pred = self.translate_operand(*pred).to_single_expr();
                let input_concat =
                    parallel_vec(inputs_values.iter().map(|val| val.to_single_expr()));

                let mut branches = vec![];
                for output_vec in outputs.iter().enumerate() {
                    let translated = self.translate_subregion(output_vec.1.iter().copied());
                    branches.push(translated);
                }
                let expr = switch_vec(pred, input_concat, branches);
                self.tuple_res(expr, id)
            }
            RvsdgBody::Theta {
                pred,
                inputs,
                outputs,
            } => {
                let mut input_values = vec![];
                for input in inputs {
                    input_values.push(self.translate_operand(*input));
                }

                let inputs_translated =
                    parallel_vec_nonempty(input_values.iter().map(|val| val.to_single_expr()));

                // For the sub-region, we need a new region translator
                // with its own arguments and bindings.
                // We then put the whole loop in a let binding and move on.
                let loop_body_translated =
                    self.translate_subregion(iter::once(pred).chain(outputs.iter()).copied());

                let loop_expr = dowhile(inputs_translated, loop_body_translated);

                self.tuple_res(loop_expr, id)
            }
        }
    }

    /// Translate this expression at the given id,
    /// return the newly created index.
    fn translate_basic_expr(&mut self, expr: BasicExpr<Operand>, id: Id) -> StoredValue {
        if let Some(cached) = self.stored_node.get(&id) {
            return cached.clone();
        }

        match expr {
            BasicExpr::Op(op, children, ty) => {
                let children = children
                    .iter()
                    .map(|c| self.translate_operand(*c).to_single_expr())
                    .collect::<Vec<_>>();
                let expr = match (op, children.as_slice()) {
                    (ValueOps::Add, [a, b]) => add(a.clone(), b.clone()),
                    (ValueOps::Mul, [a, b]) => mul(a.clone(), b.clone()),
                    (ValueOps::Sub, [a, b]) => sub(a.clone(), b.clone()),
                    (ValueOps::Div, [a, b]) => div(a.clone(), b.clone()),

                    (ValueOps::Fadd, [a, b]) => fadd(a.clone(), b.clone()),
                    (ValueOps::Fmul, [a, b]) => fmul(a.clone(), b.clone()),
                    (ValueOps::Fsub, [a, b]) => fsub(a.clone(), b.clone()),
                    (ValueOps::Fdiv, [a, b]) => fdiv(a.clone(), b.clone()),

                    (ValueOps::Eq, [a, b]) => eq(a.clone(), b.clone()),
                    (ValueOps::Gt, [a, b]) => greater_than(a.clone(), b.clone()),
                    (ValueOps::Lt, [a, b]) => less_than(a.clone(), b.clone()),
                    (ValueOps::Ge, [a, b]) => greater_eq(a.clone(), b.clone()),
                    (ValueOps::Le, [a, b]) => less_eq(a.clone(), b.clone()),

                    (ValueOps::Feq, [a, b]) => feq(a.clone(), b.clone()),
                    (ValueOps::Fgt, [a, b]) => fgreater_than(a.clone(), b.clone()),
                    (ValueOps::Flt, [a, b]) => fless_than(a.clone(), b.clone()),
                    (ValueOps::Fge, [a, b]) => fgreater_eq(a.clone(), b.clone()),
                    (ValueOps::Fle, [a, b]) => fless_eq(a.clone(), b.clone()),

                    (ValueOps::And, [a, b]) => and(a.clone(), b.clone()),
                    (ValueOps::Or, [a, b]) => or(a.clone(), b.clone()),
                    (ValueOps::Not, [a]) => not(a.clone()),

                    (ValueOps::PtrAdd, [a, b]) => ptradd(a.clone(), b.clone()),
                    (ValueOps::Load, [a, b]) => load(a.clone(), b.clone()),
                    (ValueOps::Alloc, [a, b]) => {
                        let bril_rs::Type::Pointer(_inner) = &ty else {
                            panic!("Alloc should return a pointer type, found {:?}", ty);
                        };
                        let alloc_id = self.next_alloc_id;
                        self.next_alloc_id += 1;
                        alloc(
                            alloc_id,
                            a.clone(),
                            b.clone(),
                            RvsdgType::Bril(ty).to_tree_type().unwrap(),
                        )
                    }
                    (ValueOps::Select, [a, b, c]) => select(a.clone(), b.clone(), c.clone()),
                    _ => todo!("handle {} op", op),
                };
                match op {
                    ValueOps::Alloc | ValueOps::Load => self.tuple_res(expr, id),
                    _ => self.cache_single(expr, id),
                }
            }
            BasicExpr::Call(name, inputs, _num_ret_values, _output_type) => {
                let mut input_values = vec![];
                for input in inputs {
                    input_values.push(self.translate_operand(input));
                }
                let expr = call(
                    name.as_str(),
                    parallel_vec_nonempty(input_values.into_iter().map(|val| val.to_single_expr())),
                );
                self.tuple_res(expr, id)
            }
            BasicExpr::Const(_op, literal, _ty) => {
                let lit_expr = match literal {
                    Literal::Int(n) => int(n),
                    Literal::Bool(b) => {
                        if b {
                            ttrue()
                        } else {
                            tfalse()
                        }
                    }
                    Literal::Float(f) => float(f),
                    _ => todo!("handle other literals"),
                };
                self.cache_single(lit_expr, id)
            }
            BasicExpr::Effect(EffectOps::Print, args) => {
                assert!(args.len() == 2, "print should have 2 arguments");
                let translated = self.translate_operand(args[0]);
                let arg1 = translated.to_single_expr();
                let arg2 = self.translate_operand(args[1]).to_single_expr();

                // print outputs a new unit value
                let expr = tprint(arg1, arg2);
                self.cache_single(expr, id)
            }
            BasicExpr::Effect(EffectOps::Store, args) => {
                assert!(args.len() == 3, "store should have 3 arguments");
                let arg1 = self.translate_operand(args[0]).to_single_expr();
                let arg2 = self.translate_operand(args[1]).to_single_expr();
                let arg3 = self.translate_operand(args[2]).to_single_expr();
                let expr = twrite(arg1, arg2, arg3);
                self.cache_single(expr, id)
            }
            BasicExpr::Effect(EffectOps::Free, args) => {
                assert!(args.len() == 2, "free should have 2 arguments");
                let arg1 = self.translate_operand(args[0]).to_single_expr();
                let arg2 = self.translate_operand(args[1]).to_single_expr();
                let expr = free(arg1, arg2);
                self.cache_single(expr, id)
            }
            BasicExpr::Effect(effect_op, _args) => {
                panic!("Unrecognized effect op {:?}", effect_op)
            }
        }
    }
}

impl RvsdgFunction {
    fn to_dag_encoding(&self) -> RcExpr {
        let mut translator = DagTranslator {
            stored_node: HashMap::new(),
            nodes: &self.nodes,
            next_alloc_id: 0,
        };

        let translated_results = self
            .results
            .iter()
            .map(|r| translator.translate_operand(r.1).to_single_expr())
            .collect::<Vec<_>>();
        let result_types = self
            .results
            .iter()
            .filter_map(|r| r.0.to_tree_type())
            .collect::<Vec<_>>();

        function(
            self.name.as_str(),
            Type::TupleT(
                self.args
                    .iter()
                    .filter_map(|ty| ty.to_tree_type())
                    .collect(),
            ),
            tuplet_vec(result_types),
            parallel_vec_nonempty(translated_results),
        )
    }
}

/// Assert that two programs are equal, and
/// test them on a particular input and output value
#[cfg(test)]
fn assert_progs_eq(prog1: &TreeProgram, expected: &TreeProgram, error_msg: &str) {
    if prog1 != expected {
        panic!(
            "{error_msg}\nFound:\n{}\nExpected:\n{}",
            prog1.pretty(),
            expected.pretty()
        );
    }
}

#[cfg(test)]
fn dag_translation_test(
    program: &str,
    expected: TreeProgram,
    input_val: Value,
    expected_val: Value,
    expected_printlog: Vec<String>,
) {
    use dag_in_context::interpreter::interpret_dag_prog;

    let prog = parse_from_string(program);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();
    let result = rvsdg.to_dag_encoding(false);

    assert_progs_eq(&result, &expected, "Resulting program is incorrect");

    let (found_val, found_printlog) = interpret_dag_prog(&expected, &input_val);
    assert_eq!(
        expected_val, found_val,
        "Reference program produced incorrect result. Expected {:?}, found {:?}",
        expected_val, found_val
    );
    assert_eq!(
        expected_printlog, found_printlog,
        "Reference program produced incorrect print log. Expected {:?}, found {:?}",
        expected_printlog, found_printlog
    );

    let (found_val, found_printlog) = interpret_dag_prog(&result, &input_val);
    assert_eq!(
        expected_val, found_val,
        "Resulting program produced incorrect result. Expected {:?}, found {:?}",
        expected_val, found_val
    );
    assert_eq!(
        expected_printlog, found_printlog,
        "Resulting program produced incorrect print log. Expected {:?}, found {:?}",
        expected_printlog, found_printlog
    );
}

#[test]
fn simple_translation_dag() {
    let simple_add = r#"
    @add(): int {
      v0: int = const 1;
      res: int = add v0 v0;
      ret res;
    }
    "#;
    dag_translation_test(
        simple_add,
        program!(function(
            "add",
            Type::TupleT(vec![statet()]),
            tuplet!(intt(), statet()),
            parallel!(add(int(1), int(1)), getat(0)),
        ),),
        tuplev!(statev()),
        tuplev!(intv(2), statev()),
        vec![],
    );
}

#[test]
fn dag_translate_simple_loop() {
    const PROGRAM: &str = r#"
@myfunc(): int {
    .entry:
        one: int = const 1;
        two: int = const 2;
    .loop:
        cond: bool = lt two one;
        br cond .loop .exit;
    .exit:
        ret one;
}
"#;
    let doloop = dowhile(
        parallel!(getat(0), int(1), int(2)),
        parallel!(less_than(getat(2), getat(1)), getat(0), getat(1), getat(2)),
    );
    dag_translation_test(
        PROGRAM,
        program!(function(
            "myfunc",
            tuplet!(statet()),
            tuplet!(intt(), statet()),
            parallel!(get(doloop.clone(), 1), get(doloop, 0))
        ),),
        tuplev!(statev()),
        tuplev!(Value::Const(Constant::Int(1)), statev()),
        vec![],
    );
}

#[test]
fn dag_translate_loop() {
    const PROGRAM: &str = r#"
@main {
    .entry:
        i: int = const 0;
    .loop:
        max: int = const 10;
        one: int = const 1;
        i: int = add i one;
        cond: bool = lt i max;
        br cond .loop .exit;
    .exit:
        print i;
}
"#;

    let myloop = dowhile(
        parallel!(getat(0), int(0)),
        parallel!(
            less_than(add(getat(1), int(1)), int(10)),
            getat(0),
            add(getat(1), int(1))
        ),
    );
    let myprint = tprint(get(myloop.clone(), 1), get(myloop, 0));
    dag_translation_test(
        PROGRAM,
        program!(function(
            "main",
            tuplet!(statet()),
            tuplet!(statet()),
            parallel!(myprint),
        ),),
        tuplev!(statev()),
        tuplev!(statev()),
        vec!["10".to_string()],
    );
}

#[test]
fn dag_if_translation() {
    const PROGRAM: &str = r#"
@main(): int {
    .entry:
        v0: int = const 1;
        cond: bool = lt v0 v0;
        br cond .then .else;
    .then:
        ret v0;
    .else:
        v1: int = const 2;
        ret v1;
}"#;

    let myif = tif(
        less_than(int(1), int(1)),
        parallel!(getat(0), int(1)),
        parallel!(getat(0), getat(1)),
        parallel!(getat(0), int(2)),
    );
    dag_translation_test(
        PROGRAM,
        program!(function(
            "main",
            tuplet!(statet()),
            tuplet!(intt(), statet()),
            parallel!(get(myif.clone(), 1), get(myif, 0)),
        ),),
        Value::Tuple(vec![statev()]),
        tuplev!(intv(2), statev()),
        vec![],
    );
}

#[test]
fn dag_print_translation() {
    const PROGRAM: &str = r#"
    @add() {
        v0: int = const 1;
        v1: int = const 2;
        v2: int = add v0 v1;
        print v2;
        print v1;
    }
    "#;
    let first_print = tprint(add(int(1), int(2)), getat(0));
    let second_print = tprint(int(2), first_print);
    dag_translation_test(
        PROGRAM,
        program!(function(
            "add",
            tuplet!(statet()),
            tuplet!(statet()),
            parallel!(second_print),
        ),),
        tuplev!(statev()),
        tuplev!(statev()),
        vec!["3".to_string(), "2".to_string()],
    );
}

#[test]
fn dag_multi_function_translation() {
    const PROGRAM: &str = r#"
@myadd(): int {
    v0: int = const 1;
    res: int = add v0 v0;
    ret res;
}

@main() {
    v0: int = call @myadd;
    print v0;
}
"#;

    let mycall = call("myadd", parallel!(getat(0)));
    dag_translation_test(
        PROGRAM,
        program!(
            function(
                "main",
                tuplet!(statet()),
                tuplet!(statet()),
                parallel!(tprint(get(mycall.clone(), 0), get(mycall, 1))),
            ),
            function(
                "myadd",
                tuplet!(statet()),
                tuplet!(intt(), statet()),
                parallel!(add(int(1), int(1)), getat(0)),
            ),
        ),
        tuplev!(statev()),
        tuplev!(statev()),
        vec!["2".to_string()],
    );
}
