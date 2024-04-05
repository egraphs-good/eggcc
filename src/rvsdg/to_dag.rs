//! Convert RVSDG programs to the dag encoding.
//! This is a fairly direct translation, with a minor difference being
//! that if nodes do not create a region.
//! Generating no let bindings since they are unecessary for dag semantics.
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
    ast::{add, call, dowhile, function, int, less_than, parallel_vec, program_vec, tfalse, ttrue},
    schema::{RcExpr, TreeProgram, Type},
};
use hashbrown::HashMap;

use super::RvsdgType;

impl RvsdgProgram {
    /// Converts an RVSDG program to the dag encoding.
    /// Common subexpressions are shared by the same Rc<Expr> in the dag encoding.
    /// This invariant is mainted by restore_sharing_invariant.
    pub fn to_dag_encoding(&self) -> TreeProgram {
        let last_function = self.functions.last().unwrap();
        let rest_functions = self.functions.iter().take(self.functions.len() - 1);
        program_vec(
            last_function.to_dag_encoding(),
            rest_functions
                .map(|f| f.to_dag_encoding())
                .collect::<Vec<_>>(),
        )
        .restore_sharing_invariant()
    }
}

/// Stores the location of a single value of an RVSDG node.
#[derive(Debug, Clone, PartialEq, Eq)]
enum StoredValue {
    /// The value is stored at get(arg(), usize)
    Arg(usize),
    /// ValueExpr can be used directly without a get.
    ValExpr(RcExpr),
    /// Tuple expressions, which can be projected using get.
    TupleExpr(RcExpr),
}

impl StoredValue {
    fn to_single_expr(&self) -> RcExpr {
        match self {
            StoredValue::Arg(index) => getat(*index),
            StoredValue::ValExpr(expr) => expr.clone(),
            StoredValue::TupleExpr(expr) => {
                panic!("Cannot convert tuple to single expr. Got {:?}", expr)
            }
        }
    }

    fn get_at(&self, index: usize) -> RcExpr {
        match self {
            StoredValue::Arg(argindex) => {
                assert_eq!(index, 0, "Arg can only be indexed by 0");
                getat(*argindex)
            }
            StoredValue::ValExpr(expr) => {
                assert_eq!(index, 0, "ValExpr can only be indexed by 0");
                expr.clone()
            }
            StoredValue::TupleExpr(expr) => get(expr.clone(), index),
        }
    }
}

struct DagTranslator<'a> {
    /// The values of the RVSDG arguments to this region.
    argument_values: Vec<StoredValue>,
    /// The number of tree arguments in the current environment.
    /// This is not equal to the length of `argument_values` because it refers to
    /// translated tree arguments, not the original RVSDG arguments.
    current_num_let_bound: usize,
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
    fn cache_single_res(&mut self, expr: RcExpr, id: Id) -> StoredValue {
        let res = StoredValue::ValExpr(expr.clone());
        self.stored_node.insert(id, res.clone());
        res
    }

    fn cache_tuple_res(&mut self, expr: RcExpr, id: Id) -> StoredValue {
        let res = StoredValue::TupleExpr(expr.clone());
        self.stored_node.insert(id, res.clone());
        res
    }

    /// Make a new translator for a region with
    /// num_args and the given nodes.
    fn new(
        nodes: &'a [RvsdgBody],
        argument_values: Vec<StoredValue>,
        args_let_bound: usize,
    ) -> DagTranslator {
        DagTranslator {
            current_num_let_bound: args_let_bound,
            stored_node: HashMap::new(),
            argument_values,
            nodes,
            next_alloc_id: 0,
        }
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
        argument_values: Vec<StoredValue>,
        num_let_bound: usize,
        operands: impl Iterator<Item = Operand>,
    ) -> RcExpr {
        let mut translator = DagTranslator::new(self.nodes, argument_values, num_let_bound);
        let resulting_exprs = operands.map(|operand| {
            let res = translator.translate_operand(operand);
            res.to_single_expr()
        });
        let expr = parallel_vec(resulting_exprs);
        translator.build_translation(expr)
    }

    /// Wrap the given expression in all the
    /// bindings that have been generated.
    fn build_translation(&self, inner: RcExpr) -> RcExpr {
        inner
    }

    fn translate_operand(&mut self, operand: Operand) -> StoredValue {
        match operand {
            Operand::Arg(index) => self.argument_values[index].clone(),
            Operand::Id(id) => match self.translate_node(id) {
                StoredValue::ValExpr(expr) => StoredValue::ValExpr(expr),
                StoredValue::TupleExpr(expr) => StoredValue::ValExpr(get(expr, 0)),
                StoredValue::Arg(argindex) => StoredValue::Arg(argindex),
            },
            Operand::Project(p_index, id) => {
                let values = self.translate_node(id);
                StoredValue::ValExpr(values.get_at(p_index))
            }
        }
    }

    /// Translate a node or return the index of the already-translated node.
    /// For regions, translates the region and returns the index of the
    /// tuple containing the results.
    /// It's important not to evaluate a node twice, instead using the cached index
    /// in `self.stored_node`
    fn translate_node(&mut self, id: Id) -> StoredValue {
        if let Some(stored) = self.stored_node.get(&id) {
            stored.clone()
        } else {
            let node = &self.nodes[id];
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
                        input_values.push(self.translate_operand(*input));
                    }
                    let pred = self.translate_operand(*pred).to_single_expr();
                    let then_translated = self.translate_subregion(
                        input_values.clone(),
                        self.current_num_let_bound,
                        then_branch.iter().copied(),
                    );
                    let else_translated = self.translate_subregion(
                        input_values.clone(),
                        self.current_num_let_bound,
                        else_branch.iter().copied(),
                    );

                    let expr = tif(pred, then_translated, else_translated);
                    self.cache_tuple_res(expr, id)
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

                    let mut branches = vec![];
                    for output_vec in outputs.iter().enumerate() {
                        let translated = self.translate_subregion(
                            inputs_values.clone(),
                            self.current_num_let_bound,
                            output_vec.1.iter().copied(),
                        );
                        branches.push(translated);
                    }
                    let expr = switch_vec(pred, branches);
                    self.cache_tuple_res(expr, id)
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

                    let mut input_index = 0;
                    let inner_inputs = input_values.iter().map(|_val| {
                        input_index += 1;
                        StoredValue::Arg(input_index - 1)
                    });

                    // For the sub-region, we need a new region translator
                    // with its own arguments and bindings.
                    // We then put the whole loop in a let binding and move on.
                    let loop_translated = self.translate_subregion(
                        inner_inputs.collect(),
                        input_index,
                        iter::once(pred).chain(outputs.iter()).copied(),
                    );

                    let loop_expr = dowhile(
                        parallel_vec(input_values.into_iter().map(|val| val.to_single_expr())),
                        loop_translated,
                    );

                    self.cache_tuple_res(loop_expr, id)
                }
            }
        }
    }

    /// Translate this expression at the given id,
    /// return the newly created index.
    fn translate_basic_expr(&mut self, expr: BasicExpr<Operand>, id: Id) -> StoredValue {
        match expr {
            BasicExpr::Op(op, children, ty) => {
                let children = children
                    .iter()
                    .map(|c| self.translate_operand(*c).to_single_expr())
                    .collect::<Vec<_>>();
                let expr = match (op, children.as_slice()) {
                    (ValueOps::Add, [a, b]) => add(a.clone(), b.clone()),
                    (ValueOps::Lt, [a, b]) => less_than(a.clone(), b.clone()),
                    (ValueOps::Mul, [a, b]) => mul(a.clone(), b.clone()),
                    (ValueOps::Sub, [a, b]) => sub(a.clone(), b.clone()),
                    (ValueOps::Div, [a, b]) => div(a.clone(), b.clone()),
                    (ValueOps::Eq, [a, b]) => eq(a.clone(), b.clone()),
                    (ValueOps::And, [a, b]) => and(a.clone(), b.clone()),
                    (ValueOps::Ge, [a, b]) => greater_eq(a.clone(), b.clone()),
                    (ValueOps::Le, [a, b]) => less_eq(a.clone(), b.clone()),
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
                    _ => todo!("handle {} op", op),
                };
                match op {
                    ValueOps::Alloc | ValueOps::Load => self.cache_tuple_res(expr, id),
                    _ => self.cache_single_res(expr, id),
                }
            }
            BasicExpr::Call(name, inputs, _num_ret_values, _output_type) => {
                let mut input_values = vec![];
                for input in inputs {
                    input_values.push(self.translate_operand(input));
                }
                let expr = call(
                    name.as_str(),
                    parallel_vec(input_values.into_iter().map(|val| val.to_single_expr())),
                );
                self.cache_tuple_res(expr, id)
            }
            BasicExpr::Const(_op, literal, _ty) => match literal {
                Literal::Int(n) => {
                    let expr = int(n);
                    self.cache_single_res(expr, id)
                }
                Literal::Bool(b) => {
                    let expr = if b { ttrue() } else { tfalse() };
                    self.cache_single_res(expr, id)
                }
                _ => todo!("handle other literals"),
            },
            BasicExpr::Effect(EffectOps::Print, args) => {
                assert!(args.len() == 2, "print should have 2 arguments");
                let translated = self.translate_operand(args[0]);
                let arg1 = translated.to_single_expr();
                let arg2 = self.translate_operand(args[1]).to_single_expr();

                // print outputs a new unit value
                let expr = tprint(arg1, arg2);
                self.cache_single_res(expr, id)
            }
            BasicExpr::Effect(EffectOps::Store, args) => {
                assert!(args.len() == 3, "store should have 3 arguments");
                let arg1 = self.translate_operand(args[0]).to_single_expr();
                let arg2 = self.translate_operand(args[1]).to_single_expr();
                let arg3 = self.translate_operand(args[2]).to_single_expr();
                let expr = twrite(arg1, arg2, arg3);
                self.cache_single_res(expr, id)
            }
            BasicExpr::Effect(EffectOps::Free, args) => {
                assert!(args.len() == 2, "free should have 2 arguments");
                let arg1 = self.translate_operand(args[0]).to_single_expr();
                let arg2 = self.translate_operand(args[1]).to_single_expr();
                let expr = free(arg1, arg2);
                self.cache_single_res(expr, id)
            }
            BasicExpr::Effect(effect_op, _args) => {
                panic!("Unrecognized effect op {:?}", effect_op)
            }
        }
    }
}

impl RvsdgFunction {
    /// Translates an RVSDG function to the
    /// tree encoding.
    /// It generates one let binding per
    /// node in the RVSDG, adding the value
    /// for that node to the end of the argument
    /// using the `concat` constructor.
    /// In the inner-most scope, the value of
    /// all nodes is available.
    ///
    /// When `optimize_lets` is true, the conversion will also
    /// try to prevent adding unnecessary let bindings.
    pub fn to_dag_encoding(&self) -> RcExpr {
        let argument_values: Vec<StoredValue> = self
            .args
            .iter()
            .enumerate()
            .map(|(i, _ty)| StoredValue::Arg(i))
            .collect();
        let mut translator =
            DagTranslator::new(&self.nodes, argument_values.clone(), argument_values.len());
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
            translator.build_translation(parallel_vec(translated_results)),
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
    _input_val: Value,
    _expected_val: Value,
    _expected_printlog: Vec<String>,
) {
    let prog = parse_from_string(program);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();
    let result = rvsdg.to_dag_encoding();

    /* TODO check values once dag interpreter works
    let (found_val, found_printlog) = interpret_tree_prog(&expected, &input_val);
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
    assert_eq!(
        expected_printlog, found_printlog,
        "Reference optimized program produced incorrect print log. Expected {:?}, found {:?}",
        expected_printlog, found_printlog
    );

    let (found_val, found_printlog) = interpret_tree_prog(&result, &input_val);
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

    assert_eq!(
        expected_val, found_val,
        "Resulting optimized program produced incorrect result. Expected {:?}, found {:?}",
        expected_val, found_val
    );
    assert_eq!(
        expected_printlog, found_printlog,
        "Resulting optimized program produced incorrect print log. Expected {:?}, found {:?}",
        expected_printlog, found_printlog
    );*/

    assert_progs_eq(&result, &expected, "Resulting program is incorrect");
}

#[test]
fn simple_translation_dag() {
    dag_translation_test(
        r#"
  @add(): int {
    v0: int = const 1;
    res: int = add v0 v0;
    ret res;
  }
  "#,
        program!(function(
            "add",
            Type::TupleT(vec![statet()]),
            tuplet!(intt(), statet()),
            parallel!(add(int(1), int(1)), getat(0)),
        ),),
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(2)),
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
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(1)),
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
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
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
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(2)),
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
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
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
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec!["2".to_string()],
    );
}
