use std::{collections::HashMap, rc::Rc};

use egglog::{
    ast::{Literal, Symbol},
    Term, TermDag,
};

use crate::{
    from_egglog::program_from_egglog_preserve_ctx_nodes,
    schema::{
        Assumption, BaseType, BinaryOp, Constant, Expr, TernaryOp, TreeProgram, Type, UnaryOp,
    },
};

pub(crate) struct TreeToEgglog {
    termdag: TermDag,
    // Cache for shared subexpressions
    converted_cache: HashMap<*const Expr, Term>,
}

impl TreeToEgglog {
    fn app(&mut self, f: Symbol, args: Vec<Term>) -> Term {
        self.termdag.app(f, args)
    }

    fn lit(&mut self, lit: Literal) -> Term {
        self.termdag.lit(lit)
    }
}

impl Constant {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TreeToEgglog) -> Term {
        match self {
            Constant::Int(i) => {
                let i = term_dag.lit(Literal::Int(*i));
                term_dag.app("Int".into(), vec![i])
            }
            Constant::Bool(b) => {
                let b = term_dag.lit(Literal::Bool(*b));
                term_dag.app("Bool".into(), vec![b])
            }
        }
    }

    pub(crate) fn to_egglog(&self) -> (Term, TermDag) {
        let mut state = TreeToEgglog {
            termdag: TermDag::default(),
            converted_cache: HashMap::new(),
        };
        let term = self.to_egglog_internal(&mut state);
        (term, state.termdag)
    }
}

impl BaseType {
    pub(crate) fn to_egglog_internal(&self, state: &mut TreeToEgglog) -> Term {
        match self {
            BaseType::IntT => state.app("IntT".into(), vec![]),
            BaseType::BoolT => state.app("BoolT".into(), vec![]),
            BaseType::PointerT(inner) => {
                let inner = inner.to_egglog_internal(state);
                state.app("PointerT".into(), vec![inner])
            }
            BaseType::StateT => state.app("StateT".into(), vec![]),
        }
    }
}

impl Type {
    pub(crate) fn to_egglog(&self) -> (Term, TermDag) {
        let mut state = TreeToEgglog {
            termdag: TermDag::default(),
            converted_cache: HashMap::new(),
        };
        let term = self.to_egglog_internal(&mut state);
        (term, state.termdag)
    }

    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TreeToEgglog) -> Term {
        match self {
            Type::Base(base) => {
                let baset = base.to_egglog_internal(term_dag);
                term_dag.app("Base".into(), vec![baset])
            }
            Type::TupleT(types) => {
                let types = types
                    .iter()
                    .map(|t| t.to_egglog_internal(term_dag))
                    .collect();
                let tlist = to_tlistexpr(types, term_dag);
                term_dag.app("TupleT".into(), vec![tlist])
            }
            // Unknown shouldn't show up in the egglog file, but is useful for printing
            // before types are annotated.
            Type::Unknown => term_dag.app("Unknown".into(), vec![]),
        }
    }
}

impl Assumption {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TreeToEgglog) -> Term {
        match self {
            Assumption::InLoop(lhs, rhs) => {
                let lhs = lhs.to_egglog_internal(term_dag);
                let rhs = rhs.to_egglog_internal(term_dag);
                term_dag.app("InLoop".into(), vec![lhs, rhs])
            }
            Assumption::NoContext => term_dag.app("NoContext".into(), vec![]),
            Assumption::InIf(is_then, pred, input) => {
                let pred = pred.to_egglog_internal(term_dag);
                let is_then = term_dag.lit(Literal::Bool(*is_then));
                let input = input.to_egglog_internal(term_dag);
                term_dag.app("InIf".into(), vec![is_then, pred, input])
            }
        }
    }
}

impl BinaryOp {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TreeToEgglog) -> Term {
        term_dag.app(format!("{:?}", self).into(), vec![])
    }
}

impl TernaryOp {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TreeToEgglog) -> Term {
        term_dag.app(format!("{:?}", self).into(), vec![])
    }
}

impl UnaryOp {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TreeToEgglog) -> Term {
        term_dag.app(format!("{:?}", self).into(), vec![])
    }
}

impl Expr {
    pub fn to_egglog(self: &RcExpr) -> (Term, TermDag) {
        let mut state = TreeToEgglog {
            termdag: TermDag::default(),
            converted_cache: HashMap::new(),
        };
        let term = self.to_egglog_internal(&mut state);
        (term, state.termdag)
    }

    pub(crate) fn to_egglog_internal(self: &RcExpr, term_dag: &mut TreeToEgglog) -> Term {
        if let Some(term) = term_dag.converted_cache.get(&Rc::as_ptr(self)) {
            return term.clone();
        }
        let res = match self.as_ref() {
            Expr::Const(c, ty) => {
                let child = c.to_egglog_internal(term_dag);
                let ty = ty.to_egglog_internal(term_dag);
                term_dag.app("Const".into(), vec![child, ty])
            }
            Expr::Bop(op, lhs, rhs) => {
                let lhs = lhs.to_egglog_internal(term_dag);
                let rhs = rhs.to_egglog_internal(term_dag);
                let op = op.to_egglog_internal(term_dag);
                term_dag.app("Bop".into(), vec![op, lhs, rhs])
            }
            Expr::Top(op, x, y, z) => {
                let x = x.to_egglog_internal(term_dag);
                let y = y.to_egglog_internal(term_dag);
                let z = z.to_egglog_internal(term_dag);
                let op = op.to_egglog_internal(term_dag);
                term_dag.app("Top".into(), vec![op, x, y, z])
            }
            Expr::Uop(op, expr) => {
                let expr = expr.to_egglog_internal(term_dag);
                let op = op.to_egglog_internal(term_dag);
                term_dag.app("Uop".into(), vec![op, expr])
            }
            Expr::Get(expr, index) => {
                let expr = expr.to_egglog_internal(term_dag);
                let lit_index = term_dag.lit(Literal::Int(*index as i64));
                term_dag.app("Get".into(), vec![expr, lit_index])
            }
            Expr::Alloc(id, expr, state, ty) => {
                let id = term_dag.lit(Literal::Int(*id));
                let expr = expr.to_egglog_internal(term_dag);
                let ty = ty.to_egglog_internal(term_dag);
                let state = state.to_egglog_internal(term_dag);
                term_dag.app("Alloc".into(), vec![id, expr, state, ty])
            }
            Expr::Call(name, arg) => {
                let arg = arg.to_egglog_internal(term_dag);
                let name_lit = term_dag.lit(Literal::String(name.into()));
                term_dag.app("Call".into(), vec![name_lit, arg])
            }
            Expr::Empty(ty) => {
                let ty = ty.to_egglog_internal(term_dag);
                term_dag.app("Empty".into(), vec![ty])
            }
            Expr::Single(expr) => {
                let expr = expr.to_egglog_internal(term_dag);
                term_dag.app("Single".into(), vec![expr])
            }
            Expr::Concat(lhs, rhs) => {
                let lhs = lhs.to_egglog_internal(term_dag);
                let rhs = rhs.to_egglog_internal(term_dag);
                term_dag.app("Concat".into(), vec![lhs, rhs])
            }
            Expr::Switch(expr, inputs, cases) => {
                let expr = expr.to_egglog_internal(term_dag);
                let inputs = inputs.to_egglog_internal(term_dag);
                let cases = cases
                    .iter()
                    .map(|c| c.to_egglog_internal(term_dag))
                    .collect();
                let cases = to_listexpr(cases, term_dag);
                term_dag.app("Switch".into(), vec![expr, inputs, cases])
            }
            Expr::If(cond, input, then, els) => {
                let cond = cond.to_egglog_internal(term_dag);
                let then = then.to_egglog_internal(term_dag);
                let inputs = input.to_egglog_internal(term_dag);
                let els = els.to_egglog_internal(term_dag);
                term_dag.app("If".into(), vec![cond, inputs, then, els])
            }
            Expr::DoWhile(cond, body) => {
                let cond = cond.to_egglog_internal(term_dag);
                let body = body.to_egglog_internal(term_dag);
                term_dag.app("DoWhile".into(), vec![cond, body])
            }
            Expr::Arg(ty) => {
                let ty = ty.to_egglog_internal(term_dag);
                term_dag.app("Arg".into(), vec![ty])
            }
            Expr::InContext(assumption, expr) => {
                let expr = expr.to_egglog_internal(term_dag);
                let assumption = assumption.to_egglog_internal(term_dag);
                term_dag.app("InContext".into(), vec![assumption, expr])
            }
            Expr::Function(name, ty_in, ty_out, body) => {
                let body = body.to_egglog_internal(term_dag);
                let ty_in = ty_in.to_egglog_internal(term_dag);
                let ty_out = ty_out.to_egglog_internal(term_dag);
                let name_lit = term_dag.lit(Literal::String(name.into()));
                term_dag.app("Function".into(), vec![name_lit, ty_in, ty_out, body])
            }
        };

        term_dag
            .converted_cache
            .insert(Rc::as_ptr(self), res.clone());
        res
    }
}

impl TreeProgram {
    /// DAG programs should share common subexpressions whenever possible.
    /// Otherwise, effects may happen multiple times.
    /// This function restores this invariant by converting to a Term and back again.
    pub fn restore_sharing_invariant(&self) -> TreeProgram {
        let (term, termdag) = self.to_egglog();
        program_from_egglog_preserve_ctx_nodes(term, termdag)
    }

    /// Translates an the program to an egglog term
    /// encoded with respect to `schema.egg`.
    /// Shares common subexpressions.
    pub fn to_egglog(&self) -> (Term, TermDag) {
        self.to_egglog_with_termdag(TermDag::default())
    }

    pub fn to_egglog_with_termdag(&self, termdag: TermDag) -> (Term, TermDag) {
        let mut state = TreeToEgglog {
            termdag,
            converted_cache: HashMap::new(),
        };
        (self.to_egglog_internal(&mut state), state.termdag)
    }

    // TODO Implement sharing of common subexpressions using
    // a cache and the Rc's pointer.
    fn to_egglog_internal(&self, term_dag: &mut TreeToEgglog) -> Term {
        let entry_term = self.entry.to_egglog_internal(term_dag);
        let functions_terms = self
            .functions
            .iter()
            .map(|expr| expr.to_egglog_internal(term_dag))
            .collect();
        let functions_list = to_listexpr(functions_terms, term_dag);
        term_dag.app("Program".into(), vec![entry_term, functions_list])
    }
}

fn to_listexpr(terms: Vec<Term>, term_dag: &mut TreeToEgglog) -> Term {
    let mut list = term_dag.app("Nil".into(), vec![]);
    for term in terms.into_iter().rev() {
        list = term_dag.app("Cons".into(), vec![term, list]);
    }
    list
}

fn to_tlistexpr(terms: Vec<Term>, term_dag: &mut TreeToEgglog) -> Term {
    let mut list = term_dag.app("TNil".into(), vec![]);
    for term in terms.into_iter().rev() {
        list = term_dag.app("TCons".into(), vec![term, list]);
    }
    list
}

use crate::schema::RcExpr;

#[cfg(test)]
fn test_program_parses_to(prog: TreeProgram, expected: &str) {
    let (term, mut termdag) = prog.to_egglog();
    test_parses_to(term, &mut termdag, expected);
}

#[cfg(test)]
fn test_expr_parses_to(expr: RcExpr, expected: &str) {
    let (term, mut termdag) = expr.to_egglog();
    test_parses_to(term, &mut termdag, expected);
}

#[cfg(test)]
fn test_parses_to(term: Term, termdag: &mut TermDag, expected: &str) {
    let parser = egglog::ast::parse::ExprParser::new();
    let parsed = parser.parse(expected).unwrap();
    let term2 = termdag.expr_to_term(&parsed);
    let pretty1 = termdag.term_to_expr(&term).to_sexp().pretty();
    let pretty2 = termdag.term_to_expr(&term2).to_sexp().pretty();
    assert!(pretty1 == pretty2, "Expected:\n{pretty2}\nGot:\n{pretty1}");
}

#[test]
fn convert_to_egglog_simple_arithmetic() {
    use crate::ast::*;
    let expr = add(int(1), iarg()).with_arg_types(base(intt()), base(intt()));
    test_expr_parses_to(
        expr,
        "(Bop (Add) (Const (Int 1) (Base (IntT))) (Arg (Base (IntT))))",
    );
}

#[test]
fn convert_to_egglog_switch() {
    use crate::ast::*;
    let expr = switch!(int(1), int(4); concat(single(int(1)), single(int(2))), concat(single(int(3)), single(int(4)))).with_arg_types(base(intt()), tuplet!(intt(), intt()));
    test_expr_parses_to(
        expr,
        "(Switch (Const (Int 1) (Base (IntT)))
                 (Const (Int 4) (Base (IntT)))
                 (Cons 
                  (Concat (Single (Const (Int 1) (Base (IntT)))) (Single (Const (Int 2) (Base (IntT)))))
                  (Cons 
                   (Concat (Single (Const (Int 3) (Base (IntT)))) (Single (Const (Int 4) (Base (IntT)))))
                   (Nil))))",
    );
}

#[test]
fn convert_whole_program() {
    use crate::ast::*;
    let expr = program!(
        function(
            "main",
            base(intt()),
            base(intt()),
            add(int(1), call("f", int(2)))
        ),
        function(
            "f",
            base(intt()),
            base(intt()),
            get(
                dowhile(
                    single(arg()),
                    push(add(getat(0), int(1)), single(less_than(getat(0), int(10))))
                ),
                0
            )
        )
    );
    test_program_parses_to(
        expr,
        "(Program 
            (Function \"main\" (Base (IntT)) (Base (IntT)) 
                (Bop (Add) (Const (Int 1) (Base (IntT))) (Call \"f\" (Const (Int 2) (Base (IntT)))))) 
            (Cons 
                (Function \"f\" (Base (IntT)) (Base (IntT)) 
                    (Get
                        (DoWhile (Single (Arg (Base (IntT))))
                        (Concat 
                            (Single (Bop (LessThan) (Get (Arg (TupleT (TCons (IntT) (TNil)))) 0) (Const (Int 10) (TupleT (TCons (IntT) (TNil))))))
                            (Single (Bop (Add) (Get (Arg (TupleT (TCons (IntT) (TNil)))) 0) (Const (Int 1) (TupleT (TCons (IntT) (TNil))))))))
                        0)) 
                (Nil)))",
    );
}
