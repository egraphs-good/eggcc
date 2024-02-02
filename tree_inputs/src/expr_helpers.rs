use crate::schema::{Expr, RcExpr};

impl Expr {
    pub fn map_children(self: RcExpr, func: &impl Fn(RcExpr) -> RcExpr) -> RcExpr {
        match self.as_ref() {
            Expr::Const(_c, _v) => self.clone(),
            Expr::Bop(op, l, r) => {
                RcExpr::new(Expr::Bop(op.clone(), func(l.clone()), func(r.clone())))
            }
            Expr::Uop(op, e) => RcExpr::new(Expr::Uop(op.clone(), func(e.clone()))),
            Expr::Get(e, i) => RcExpr::new(Expr::Get(func(e.clone()), *i)),
            Expr::Read(e, ty) => RcExpr::new(Expr::Read(func(e.clone()), ty.clone())),
            Expr::Call(s, e) => RcExpr::new(Expr::Call(s.clone(), func(e.clone()))),
            Expr::All(c, o, es) => {
                let mut new_es = vec![];
                for e in es {
                    new_es.push(func(e.clone()));
                }
                RcExpr::new(Expr::All(c.clone(), o.clone(), new_es))
            }
            Expr::Switch(e, es) => {
                let mut new_es = vec![];
                for e in es {
                    new_es.push(func(e.clone()));
                }
                RcExpr::new(Expr::Switch(func(e.clone()), new_es))
            }
            Expr::If(c, t, f) => {
                RcExpr::new(Expr::If(func(c.clone()), func(t.clone()), func(f.clone())))
            }
            Expr::Input(e) => RcExpr::new(Expr::Input(func(e.clone()))),
            Expr::Arg(_i) => self.clone(),
            Expr::Let(e) => RcExpr::new(Expr::Let(func(e.clone()))),
            Expr::DoWhile(c, b, e) => RcExpr::new(Expr::DoWhile(
                func(c.clone()),
                func(b.clone()),
                func(e.clone()),
            )),
            Expr::Function(n, a, r, e) => RcExpr::new(Expr::Function(
                n.clone(),
                a.clone(),
                r.clone(),
                func(e.clone()),
            )),
        }
    }

    pub fn children(&self) -> Vec<RcExpr> {
        match self {
            Expr::Const(_, _) => vec![],
            Expr::Bop(_, l, r) => vec![l.clone(), r.clone()],
            Expr::Uop(_, e) => vec![e.clone()],
            Expr::Get(e, _) => vec![e.clone()],
            Expr::Read(e, _) => vec![e.clone()],
            Expr::Call(_, e) => vec![e.clone()],
            Expr::All(_, _, es) => es.clone(),
            Expr::Switch(e, es) => {
                let mut res = vec![e.clone()];
                for e in es {
                    res.push(e.clone());
                }
                res
            }
            Expr::If(c, t, f) => vec![c.clone(), t.clone(), f.clone()],
            Expr::Input(e) => vec![e.clone()],
            Expr::Arg(_) => vec![],
            Expr::Let(e) => vec![e.clone()],
            Expr::DoWhile(c, b, e) => vec![c.clone(), b.clone(), e.clone()],
            Expr::Function(_, _, _, e) => vec![e.clone()],
        }
    }

    /// Finds the input or argument to the given
    /// expression.
    /// If the expression refers to no inputs
    /// or arguments, returns None.
    /// Assumes a valid expression, and panics
    /// on finding multiple different inputs.
    pub fn find_input(self: RcExpr) -> Option<RcExpr> {
        match self.as_ref() {
            Expr::Input(..) => Some(self.clone()),
            Expr::Arg(..) => Some(self.clone()),
            _ => {
                let children = self.children();
                let mut res = None;
                for child in children {
                    if let Some(input) = child.find_input() {
                        if let Some(old) = res {
                            assert_eq!(old, input, "Multiple different inputs found");
                        }
                        res = Some(input);
                    }
                }
                res
            }
        }
    }

    /// Substitutes the second argument for the
    /// first in the expression.
    pub(crate) fn substitute(self: RcExpr, from: RcExpr, to: RcExpr) -> RcExpr {
        if self == from {
            to
        } else {
            self.map_children(&|e| e.substitute(from.clone(), to.clone()))
        }
    }
}
