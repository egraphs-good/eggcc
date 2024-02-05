use egglog::{ast::Literal, Term, TermDag};

use crate::schema::{Assumption, BaseType, Constant, Expr, RcExpr, TreeProgram, Type};

impl Constant {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
        match self {
            Constant::Int(i) => term_dag.app("Int".into(), vec![Term::Lit(Literal::Int(*i))]),
            Constant::Bool(b) => term_dag.app("Bool".into(), vec![Term::Lit(Literal::Bool(*b))]),
        }
    }
}

impl BaseType {
    pub(crate) fn to_egglog_internal(&self, term_dag: &mut TermDag) -> Term {
        term_dag.app(format!("{:?}", self).into(), vec![])
    }
}

impl Type {
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
        }
    }
}

impl Expr {
    pub fn func_name(&self) -> Option<String> {
        match self {
            Expr::Function(name, _, _, _) => Some(name.clone()),
            _ => None,
        }
    }

    pub fn func_body(&self) -> Option<&RcExpr> {
        match self {
            Expr::Function(_, _, _, body) => Some(body),
            _ => None,
        }
    }

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
                term_dag.app(format!("{:?}", op).into(), vec![lhs, rhs])
            }
            Expr::Uop(op, expr) => {
                let expr = expr.to_egglog_internal(term_dag);
                term_dag.app(format!("{:?}", op).into(), vec![expr])
            }
            Expr::Get(expr, index) => {
                let expr = expr.to_egglog_internal(term_dag);
                let index =
                    term_dag.app("Int".into(), vec![Term::Lit(Literal::Int(*index as i64))]);
                term_dag.app("Get".into(), vec![expr, index])
            }
            Expr::Alloc(expr, ty) => {
                let expr = expr.to_egglog_internal(term_dag);
                let ty = ty.to_egglog_internal(term_dag);
                term_dag.app("Alloc".into(), vec![expr, ty])
            }
            Expr::Call(name, arg) => {
                let arg = arg.to_egglog_internal(term_dag);
                term_dag.app(
                    "Call".into(),
                    vec![Term::Lit(Literal::String(name.into())), arg],
                )
            }
            Expr::Empty => term_dag.app("Empty".into(), vec![]),
            Expr::Single(expr) => expr.to_egglog_internal(term_dag),
            Expr::Extend(order, lhs, rhs) => {
                let lhs = lhs.to_egglog_internal(term_dag);
                let rhs = rhs.to_egglog_internal(term_dag);
                term_dag.app(format!("{:?}", order).into(), vec![lhs, rhs])
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
                termdag.app("Assume".into(), vec![assumption, expr])
            }
            _ => todo!(),
        }
    }
}

impl TreeProgram {
    pub fn get_function(&self, name: &str) -> Option<&RcExpr> {
        if self.entry.func_name() == Some(name.to_string()) {
            return Some(&self.entry);
        }
        self.functions
            .iter()
            .find(|expr| expr.func_name() == Some(name.to_string()))
    }

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
        term_dag.app("TreeProgram".into(), vec![entry_term, functions_list])
    }
}

fn to_listexpr(terms: Vec<Term>, term_dag: &mut TermDag) -> Term {
    let mut list = term_dag.app("Nil".into(), vec![]);
    for term in terms {
        list = term_dag.app("Cons".into(), vec![term, list]);
    }
    list
}

fn to_tlistexpr(terms: Vec<Term>, term_dag: &mut TermDag) -> Term {
    let mut list = term_dag.app("TNil".into(), vec![]);
    for term in terms {
        list = term_dag.app("TCons".into(), vec![term, list]);
    }
    list
}

#[test]
fn convert_to_egglog_simple_arithmetic() {
    use crate::ast::*;
    let expr = add(int(1), arg());
    let (term, termdag) = expr.to_egglog();
    assert_eq!(termdag.to_string(&term), "(Add (Int 1) Arg)");
}
