use std::{rc::Rc, vec};

use egglog::{
    ast::{Literal, Symbol},
    Term, TermDag,
};
use indexmap::IndexMap;

use crate::{
    from_egglog::program_from_egglog_preserve_ctx_nodes,
    schema::{
        Assumption, BaseType, BinaryOp, Constant, Expr, TernaryOp, TreeProgram, Type, UnaryOp,
    },
};

pub(crate) struct TreeToEgglog {
    pub termdag: TermDag,
    // Cache for shared subexpressions
    pub converted_cache: IndexMap<*const Expr, Term>,
}

impl TreeToEgglog {
    // Creates a default tree to egglog
    pub fn new() -> TreeToEgglog {
        TreeToEgglog {
            termdag: TermDag::default(),
            converted_cache: IndexMap::new(),
        }
    }

    fn app(&mut self, f: Symbol, args: Vec<Term>) -> Term {
        self.termdag.app(f, args)
    }

    fn var(&mut self, f: Symbol) -> Term {
        self.termdag.var(f)
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
            Constant::Float(f) => {
                let b = term_dag.lit(Literal::F64(*f));
                term_dag.app("Float".into(), vec![b])
            }
        }
    }

    pub(crate) fn to_egglog(&self) -> (Term, TermDag) {
        let mut state = TreeToEgglog::new();
        let term = self.to_egglog_internal(&mut state);
        (term, state.termdag)
    }
}

impl BaseType {
    pub(crate) fn to_egglog_internal(&self, state: &mut TermDag) -> Term {
        match self {
            BaseType::IntT => state.app("IntT".into(), vec![]),
            BaseType::FloatT => state.app("FloatT".into(), vec![]),
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
        let mut state = TreeToEgglog::new();
        let term = self.to_egglog_internal(&mut state);
        (term, state.termdag)
    }

    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TreeToEgglog) -> Term {
        match self {
            Type::Base(base) => {
                let baset = base.to_egglog_internal(&mut term_dag.termdag);
                term_dag.app("Base".into(), vec![baset])
            }
            Type::TupleT(types) => {
                let types = types
                    .iter()
                    .map(|t| t.to_egglog_internal(&mut term_dag.termdag))
                    .collect();
                let tlist = to_tlistexpr(types, term_dag);
                term_dag.app("TupleT".into(), vec![tlist])
            }
            // Unknown shouldn't show up in the egglog file, but is useful for printing
            // before types are annotated.
            Type::Unknown => term_dag.app("Unknown".into(), vec![]),
            Type::Symbolic(str) => term_dag.var(str.into()),
        }
    }
}

impl Assumption {
    pub(crate) fn to_egglog(&self) -> (Term, TermDag) {
        let mut state = TreeToEgglog::new();
        let term = self.to_egglog_internal(&mut state);
        (term, state.termdag)
    }

    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TreeToEgglog) -> Term {
        match self {
            Assumption::InLoop(lhs, rhs) => {
                let lhs = lhs.to_egglog_with(term_dag);
                let rhs = rhs.to_egglog_with(term_dag);
                term_dag.app("InLoop".into(), vec![lhs, rhs])
            }
            Assumption::InIf(is_then, pred, input) => {
                let pred = pred.to_egglog_with(term_dag);
                let is_then = term_dag.lit(Literal::Bool(*is_then));
                let input = input.to_egglog_with(term_dag);
                term_dag.app("InIf".into(), vec![is_then, pred, input])
            }
            Assumption::InSwitch(branch, pred, input) => {
                let pred = pred.to_egglog_with(term_dag);
                let branch = term_dag.lit(Literal::Int(*branch));
                let input = input.to_egglog_with(term_dag);
                term_dag.app("InSwitch".into(), vec![branch, pred, input])
            }
            Assumption::WildCard(str) => term_dag.var(str.into()),
            Assumption::InFunc(name) => {
                let name = term_dag.lit(Literal::String(name.into()));
                term_dag.app("InFunc".into(), vec![name])
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
        let mut state = TreeToEgglog::new();
        let term = self.to_egglog_with(&mut state);
        (term, state.termdag)
    }

    pub(crate) fn to_egglog_with(self: &RcExpr, term_dag: &mut TreeToEgglog) -> Term {
        if let Some(term) = term_dag.converted_cache.get(&Rc::as_ptr(self)) {
            return term.clone();
        }
        let res = match self.as_ref() {
            Expr::Const(c, ty, ctx) => {
                let child = c.to_egglog_internal(term_dag);
                let ty = ty.to_egglog_internal(term_dag);
                let ctx = ctx.to_egglog_internal(term_dag);
                term_dag.app("Const".into(), vec![child, ty, ctx])
            }
            Expr::Bop(op, lhs, rhs) => {
                let lhs = lhs.to_egglog_with(term_dag);
                let rhs = rhs.to_egglog_with(term_dag);
                let op = op.to_egglog_internal(term_dag);
                term_dag.app("Bop".into(), vec![op, lhs, rhs])
            }
            Expr::Top(op, x, y, z) => {
                let x = x.to_egglog_with(term_dag);
                let y = y.to_egglog_with(term_dag);
                let z = z.to_egglog_with(term_dag);
                let op = op.to_egglog_internal(term_dag);
                term_dag.app("Top".into(), vec![op, x, y, z])
            }
            Expr::Uop(op, expr) => {
                let expr = expr.to_egglog_with(term_dag);
                let op = op.to_egglog_internal(term_dag);
                term_dag.app("Uop".into(), vec![op, expr])
            }
            Expr::Get(expr, index) => {
                let expr = expr.to_egglog_with(term_dag);
                let lit_index = term_dag.lit(Literal::Int(*index as i64));
                term_dag.app("Get".into(), vec![expr, lit_index])
            }
            Expr::Alloc(id, expr, state, ty) => {
                let id = term_dag.lit(Literal::Int(*id));
                let expr = expr.to_egglog_with(term_dag);
                let ty = ty.to_egglog_internal(&mut term_dag.termdag);
                let state = state.to_egglog_with(term_dag);
                term_dag.app("Alloc".into(), vec![id, expr, state, ty])
            }
            Expr::Call(name, arg) => {
                let arg = arg.to_egglog_with(term_dag);
                let name_lit = term_dag.lit(Literal::String(name.into()));
                term_dag.app("Call".into(), vec![name_lit, arg])
            }
            Expr::Empty(ty, ctx) => {
                let ty = ty.to_egglog_internal(term_dag);
                let ctx = ctx.to_egglog_internal(term_dag);
                term_dag.app("Empty".into(), vec![ty, ctx])
            }
            Expr::Single(expr) => {
                let expr = expr.to_egglog_with(term_dag);
                term_dag.app("Single".into(), vec![expr])
            }
            Expr::Concat(lhs, rhs) => {
                let lhs = lhs.to_egglog_with(term_dag);
                let rhs = rhs.to_egglog_with(term_dag);
                term_dag.app("Concat".into(), vec![lhs, rhs])
            }
            Expr::Switch(expr, inputs, cases) => {
                let expr = expr.to_egglog_with(term_dag);
                let inputs = inputs.to_egglog_with(term_dag);
                let cases = cases.iter().map(|c| c.to_egglog_with(term_dag)).collect();
                let cases = to_listexpr(cases, term_dag);
                term_dag.app("Switch".into(), vec![expr, inputs, cases])
            }
            Expr::If(cond, input, then, els) => {
                let cond = cond.to_egglog_with(term_dag);
                let then = then.to_egglog_with(term_dag);
                let inputs = input.to_egglog_with(term_dag);
                let els = els.to_egglog_with(term_dag);
                term_dag.app("If".into(), vec![cond, inputs, then, els])
            }
            Expr::DoWhile(cond, body) => {
                let cond = cond.to_egglog_with(term_dag);
                let body = body.to_egglog_with(term_dag);
                term_dag.app("DoWhile".into(), vec![cond, body])
            }
            Expr::Arg(ty, ctx) => {
                let ty = ty.to_egglog_internal(term_dag);
                let ctx = ctx.to_egglog_internal(term_dag);
                term_dag.app("Arg".into(), vec![ty, ctx])
            }
            Expr::Function(name, ty_in, ty_out, body) => {
                let body = body.to_egglog_with(term_dag);
                let ty_in = ty_in.to_egglog_internal(term_dag);
                let ty_out = ty_out.to_egglog_internal(term_dag);
                let name_lit = term_dag.lit(Literal::String(name.into()));
                term_dag.app("Function".into(), vec![name_lit, ty_in, ty_out, body])
            }
            Expr::Symbolic(name, _ty) => term_dag.var(name.into()),
            Expr::DeadCode(_subexpr) => panic!("Dead code should not be converted to egglog"),
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
        let (term, mut termdag) = self.to_egglog();
        program_from_egglog_preserve_ctx_nodes(term, &mut termdag)
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
            converted_cache: IndexMap::new(),
        };
        (self.to_egglog_with(&mut state), state.termdag)
    }

    // TODO Implement sharing of common subexpressions using
    // a cache and the Rc's pointer.
    pub(crate) fn to_egglog_with(&self, term_dag: &mut TreeToEgglog) -> Term {
        let entry_term = self.entry.to_egglog_with(term_dag);
        let functions_terms = self
            .functions
            .iter()
            .map(|expr| expr.to_egglog_with(term_dag))
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
    use egglog::ast::parse_expr;

    let parsed = parse_expr(None, expected).unwrap();
    let term2 = termdag.expr_to_term(&parsed);
    let pretty1 = termdag.term_to_expr(&term).to_sexp().pretty();
    let pretty2 = termdag.term_to_expr(&term2).to_sexp().pretty();
    assert!(pretty1 == pretty2, "Expected:\n{pretty2}\nGot:\n{pretty1}");
}

#[test]
fn convert_to_egglog_simple_arithmetic() {
    use crate::ast::*;
    use crate::schema::Assumption;
    let expr = add(int(1), iarg()).with_arg_types(base(intt()), base(intt()));
    let ctx = Assumption::dummy();
    test_expr_parses_to(
        expr,
        &format!("(Bop (Add) (Const (Int 1) (Base (IntT)) {ctx}) (Arg (Base (IntT)) {ctx}))"),
    );
}

#[test]
fn convert_to_egglog_switch() {
    use crate::ast::*;
    use crate::schema::Assumption;
    let expr = switch!(int(1), int(4); concat(single(int(1)), single(int(2))), concat(single(int(3)), single(int(4)))).with_arg_types(base(intt()), tuplet!(intt(), intt()));
    let ctx = format!("{}", Assumption::dummy());
    test_expr_parses_to(
        expr,
        &format!("(Switch (Const (Int 1) (Base (IntT)) {ctx})
                 (Const (Int 4) (Base (IntT)) {ctx})
                 (Cons 
                  (Concat (Single (Const (Int 1) (Base (IntT)) {ctx})) (Single (Const (Int 2) (Base (IntT)) {ctx})))
                  (Cons 
                   (Concat (Single (Const (Int 3) (Base (IntT)) {ctx})) (Single (Const (Int 4) (Base (IntT)) {ctx})))
                   (Nil))))"),
    );
}

#[test]
fn convert_whole_program() {
    use crate::ast::*;
    use crate::schema::Assumption;
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
    let ctx = format!("{}", Assumption::dummy());
    test_program_parses_to(
        expr,
        &format!("(Program 
            (Function \"main\" (Base (IntT)) (Base (IntT)) 
                (Bop (Add) (Const (Int 1) (Base (IntT)) {ctx}) (Call \"f\" (Const (Int 2) (Base (IntT)) {ctx})))) 
            (Cons 
                (Function \"f\" (Base (IntT)) (Base (IntT)) 
                    (Get
                        (DoWhile (Single (Arg (Base (IntT)) {ctx}))
                        (Concat 
                            (Single (Bop (LessThan) (Get (Arg (TupleT (TCons (IntT) (TNil))) {ctx}) 0) (Const (Int 10) (TupleT (TCons (IntT) (TNil))) {ctx})))
                            (Single (Bop (Add) (Get (Arg (TupleT (TCons (IntT) (TNil))) {ctx}) 0) (Const (Int 1) (TupleT (TCons (IntT) (TNil))) {ctx})))))
                        0)) 
                (Nil)))"),
    );
}
