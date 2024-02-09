use egglog::{ast::Literal, Term, TermDag};

use crate::schema::{
    Assumption, BaseType, BinaryOp, Constant, Expr, Order, TreeProgram, Type, UnaryOp,
};

impl Constant {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
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
        let mut termdag = TermDag::default();
        let term = self.to_egglog_internal(&mut termdag);
        (term, termdag)
    }
}

impl BaseType {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
        term_dag.app(format!("{:?}", self).into(), vec![])
    }
}

impl Type {
    pub(crate) fn to_egglog(&self) -> (Term, TermDag) {
        let mut termdag = TermDag::default();
        let term = self.to_egglog_internal(&mut termdag);
        (term, termdag)
    }

    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
        match self {
            Type::Base(base) => {
                let baset = base.to_egglog_internal(term_dag);
                term_dag.app("Base".into(), vec![baset])
            }
            Type::PointerT(base) => {
                let base = base.to_egglog_internal(term_dag);
                term_dag.app("PointerT".into(), vec![base])
            }
            Type::TupleT(types) => {
                let types = types
                    .iter()
                    .map(|t| t.to_egglog_internal(term_dag))
                    .collect();
                let tlist = to_tlistexpr(types, term_dag);
                term_dag.app("TupleT".into(), vec![tlist])
            }
        }
    }
}

impl Assumption {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
        match self {
            Assumption::InLet(expr) => {
                let expr = expr.to_egglog_internal(term_dag);
                term_dag.app("InLet".into(), vec![expr])
            }
            Assumption::InLoop(lhs, rhs) => {
                let lhs = lhs.to_egglog_internal(term_dag);
                let rhs = rhs.to_egglog_internal(term_dag);
                term_dag.app("InLoop".into(), vec![lhs, rhs])
            }
            Assumption::InFunc(name) => {
                let name_lit = term_dag.lit(Literal::String(name.into()));
                term_dag.app("InFunc".into(), vec![name_lit])
            }
            Assumption::InIf(is_then, pred) => {
                let pred = pred.to_egglog_internal(term_dag);
                let is_then = term_dag.lit(Literal::Bool(*is_then));
                term_dag.app("InIf".into(), vec![is_then, pred])
            }
        }
    }
}

impl Order {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
        term_dag.app(format!("{:?}", self).into(), vec![])
    }
}

impl BinaryOp {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
        term_dag.app(format!("{:?}", self).into(), vec![])
    }
}

impl UnaryOp {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
        term_dag.app(format!("{:?}", self).into(), vec![])
    }
}

impl Expr {
    pub fn to_egglog(&self) -> (Term, TermDag) {
        let mut termdag = TermDag::default();
        let term = self.to_egglog_internal(&mut termdag);
        (term, termdag)
    }

    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
        match self {
            Expr::Const(c) => {
                let child = c.to_egglog_internal(term_dag);
                term_dag.app("Const".into(), vec![child])
            }
            Expr::Bop(op, lhs, rhs) => {
                let lhs = lhs.to_egglog_internal(term_dag);
                let rhs = rhs.to_egglog_internal(term_dag);
                let op = op.to_egglog_internal(term_dag);
                term_dag.app("Bop".into(), vec![op, lhs, rhs])
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
            Expr::Alloc(expr, ty) => {
                let expr = expr.to_egglog_internal(term_dag);
                let ty = ty.to_egglog_internal(term_dag);
                term_dag.app("Alloc".into(), vec![expr, ty])
            }
            Expr::Call(name, arg) => {
                let arg = arg.to_egglog_internal(term_dag);
                let name_lit = term_dag.lit(Literal::String(name.into()));
                term_dag.app("Call".into(), vec![name_lit, arg])
            }
            Expr::Empty => term_dag.app("Empty".into(), vec![]),
            Expr::Single(expr) => {
                let expr = expr.to_egglog_internal(term_dag);
                term_dag.app("Single".into(), vec![expr])
            }
            Expr::Concat(order, lhs, rhs) => {
                let lhs = lhs.to_egglog_internal(term_dag);
                let rhs = rhs.to_egglog_internal(term_dag);
                let order = order.to_egglog_internal(term_dag);
                term_dag.app("Concat".into(), vec![order, lhs, rhs])
            }
            Expr::Switch(expr, cases) => {
                let expr = expr.to_egglog_internal(term_dag);
                let cases = cases
                    .iter()
                    .map(|c| c.to_egglog_internal(term_dag))
                    .collect();
                let cases = to_listexpr(cases, term_dag);
                term_dag.app("Switch".into(), vec![expr, cases])
            }
            Expr::If(cond, then, els) => {
                let cond = cond.to_egglog_internal(term_dag);
                let then = then.to_egglog_internal(term_dag);
                let els = els.to_egglog_internal(term_dag);
                term_dag.app("If".into(), vec![cond, then, els])
            }
            Expr::Let(lhs, rhs) => {
                let lhs = lhs.to_egglog_internal(term_dag);
                let rhs = rhs.to_egglog_internal(term_dag);
                term_dag.app("Let".into(), vec![lhs, rhs])
            }
            Expr::DoWhile(cond, body) => {
                let cond = cond.to_egglog_internal(term_dag);
                let body = body.to_egglog_internal(term_dag);
                term_dag.app("DoWhile".into(), vec![cond, body])
            }
            Expr::Arg => term_dag.app("Arg".into(), vec![]),
            Expr::Assume(assumption, expr) => {
                let expr = expr.to_egglog_internal(term_dag);
                let assumption = assumption.to_egglog_internal(term_dag);
                term_dag.app("Assume".into(), vec![assumption, expr])
            }
            Expr::Function(name, ty_in, ty_out, body) => {
                let body = body.to_egglog_internal(term_dag);
                let ty_in = ty_in.to_egglog_internal(term_dag);
                let ty_out = ty_out.to_egglog_internal(term_dag);
                let name_lit = term_dag.lit(Literal::String(name.into()));
                term_dag.app("Function".into(), vec![name_lit, ty_in, ty_out, body])
            }
        }
    }
}

impl TreeProgram {
    /// Translates an the program to an egglog term
    /// encoded with respect to `schema.egg`.
    /// Shares common subexpressions.
    pub fn to_egglog(&self) -> (Term, TermDag) {
        let mut termdag = TermDag::default();
        let term = self.to_egglog_internal(&mut termdag);
        (term, termdag)
    }

    // TODO Implement sharing of common subexpressions using
    // a cache and the Rc's pointer.
    fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
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

fn to_listexpr(terms: Vec<Term>, term_dag: &mut TermDag) -> Term {
    let mut list = term_dag.app("Nil".into(), vec![]);
    for term in terms.into_iter().rev() {
        list = term_dag.app("Cons".into(), vec![term, list]);
    }
    list
}

fn to_tlistexpr(terms: Vec<Term>, term_dag: &mut TermDag) -> Term {
    let mut list = term_dag.app("TNil".into(), vec![]);
    for term in terms.into_iter().rev() {
        list = term_dag.app("TCons".into(), vec![term, list]);
    }
    list
}

#[cfg(test)]
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
    let expr = add(int(1), arg());
    test_expr_parses_to(expr, "(Bop (Add) (Const (Int 1)) (Arg))");
}

#[test]
fn convert_to_egglog_switch() {
    use crate::ast::*;
    let expr = switch!(int(1); concat_par(single(int(1)), single(int(2))), concat_par(single(int(3)), single(int(4))));
    test_expr_parses_to(
        expr,
        "(Switch (Const (Int 1))
                 (Cons 
                  (Concat (Parallel) (Single (Const (Int 1))) (Single (Const (Int 2))))
                  (Cons 
                   (Concat (Parallel) (Single (Const (Int 3))) (Single (Const (Int 4))))
                   (Nil))))",
    );
}

#[test]
fn convert_whole_program() {
    use crate::ast::*;
    let expr = program!(
        function("main", intt(), intt(), add(int(1), call("f", int(2)))),
        function(
            "f",
            intt(),
            intt(),
            dowhile(
                arg(),
                push_par(add(arg(), int(1)), single(less_than(arg(), int(10))))
            )
        )
    );
    test_program_parses_to(
        expr,
        "(Program 
            (Function \"main\" (Base (IntT)) (Base (IntT)) 
                (Bop (Add) (Const (Int 1)) (Call \"f\" (Const (Int 2))))) 
            (Cons 
                (Function \"f\" (Base (IntT)) (Base (IntT)) 
                    (DoWhile (Arg) 
                        (Concat (Parallel) 
                            (Single (Bop (LessThan) (Arg) (Const (Int 10))))
                            (Single (Bop (Add) (Arg) (Const (Int 1))))))) 
                (Nil)))",
    );
}
