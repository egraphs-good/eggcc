//! Converts from an egglog AST directly to the rust representation of that AST.
//! Common subexpressions (common terms) must be converted to the same RcExpr (pointer equality).

use std::rc::Rc;

use egglog::{ast::Literal, match_term_app, Term};

use crate::schema::{
    Assumption, BaseType, BinaryOp, Constant, Expr, Order, RcExpr, TernaryOp, TreeProgram, Type,
    UnaryOp,
};

pub struct FromEgglog {
    pub termdag: egglog::TermDag,
}

impl FromEgglog {
    fn const_from_egglog(&self, constant: Term) -> Constant {
        match_term_app!(constant.clone(); {
          ("Int", [lit]) => {
            let Term::Lit(Literal::Int(integer)) = self.termdag.get(*lit) else {
              panic!("Invalid integer: {:?}", lit)
            };
            Constant::Int(integer)
          }
          ("Bool", [lit]) => {
            let Term::Lit(Literal::Bool(boolean)) = self.termdag.get(*lit) else {
              panic!("Invalid boolean: {:?}", lit)
            };
            Constant::Bool(boolean)
          }
          _ => panic!("Invalid constant: {:?}", constant),
        })
    }

    fn basetype_from_egglog(&self, basetype: Term) -> BaseType {
        match_term_app!(basetype.clone(); {
          ("IntT", []) => BaseType::IntT,
          ("BoolT", []) => BaseType::BoolT,
          ("PointerT", [basetype]) => BaseType::PointerT(Box::new(self.basetype_from_egglog(self.termdag.get(*basetype)))),
          ("StateT", []) => BaseType::StateT,
          _ => panic!("Invalid basetype: {:?}", basetype),
        })
    }

    fn vec_from_tlistexpr_helper(&self, tlistexpr: Term, acc: &mut Vec<BaseType>) {
        match_term_app!(tlistexpr.clone();
        {
          ("TNil", []) => (),
          ("TCons", [type_, tlistexpr]) => {
            let type_ = self.termdag.get(*type_);
            let tlistexpr = self.termdag.get(*tlistexpr);
            acc.push(self.basetype_from_egglog(type_));
            self.vec_from_tlistexpr_helper(tlistexpr, acc);
          }
          _ => panic!("Invalid tlistexpr: {:?}", tlistexpr),
        })
    }

    fn vec_from_tlistexpr(&self, tlistexpr: Term) -> Vec<BaseType> {
        let mut types = vec![];
        self.vec_from_tlistexpr_helper(tlistexpr, &mut types);
        types
    }

    fn vec_from_listexpr_helper(&self, listexpr: Term, acc: &mut Vec<RcExpr>) {
        match_term_app!(listexpr.clone();
        {
          ("Nil", []) => (),
          ("Cons", [expr, listexpr]) => {
            let expr = self.termdag.get(*expr);
            acc.push(self.expr_from_egglog(expr));
            let listexpr = self.termdag.get(*listexpr);
            self.vec_from_listexpr_helper(listexpr, acc);
          }
          _ => panic!("Invalid listexpr: {:?}", listexpr),
        })
    }

    fn vec_from_listexpr(&self, listexpr: Term) -> Vec<RcExpr> {
        let mut exprs = vec![];
        self.vec_from_listexpr_helper(listexpr, &mut exprs);
        exprs
    }

    fn type_from_egglog(&self, type_: Term) -> Type {
        match_term_app!(type_.clone(); {
          ("Base", [basetype]) => Type::Base(self.basetype_from_egglog(self.termdag.get(*basetype))),
          ("TupleT", [types]) => {
            let types = self.termdag.get(*types);
            Type::TupleT(self.vec_from_tlistexpr(types))
          }
          _ => panic!("Invalid type: {:?}", type_),
        })
    }

    fn assumption_from_egglog(&self, assumption: Term) -> Assumption {
        match_term_app!(assumption.clone();
        {
          ("InLet", [expr]) => {
            Assumption::InLet(self.expr_from_egglog(self.termdag.get(*expr)))
          }
          ("InLoop", [lhs, rhs]) => {
            Assumption::InLoop(
              self.expr_from_egglog(self.termdag.get(*lhs)),
              self.expr_from_egglog(self.termdag.get(*rhs)),
            )
          }
          ("InFunc", [lit]) => {
            let Term::Lit(Literal::String(string)) = self.termdag.get(*lit) else {
              panic!("Invalid string: {:?}", lit)
            };
            Assumption::InFunc(string.to_string())
          }
          ("InIf", [is_then, expr]) => {
            let Term::Lit(Literal::Bool(boolean)) = self.termdag.get(*is_then)
            else {
              panic!("Invalid boolean: {:?}", is_then)
            };
            Assumption::InIf(boolean, self.expr_from_egglog(self.termdag.get(*expr)))
          }
          _ => panic!("Invalid assumption: {:?}", assumption),
        })
    }

    fn order_from_egglog(&self, order: Term) -> Order {
        match_term_app!(order.clone();
        {
          ("Parallel", []) => Order::Parallel,
          ("Sequential", []) => Order::Sequential,
          ("Reversed", []) => Order::Reversed,
          _ => panic!("Invalid order: {:?}", order),
        })
    }

    fn top_from_egglog(&self, top: Term) -> TernaryOp {
        match_term_app!(top.clone();
        {
          ("Write", []) => TernaryOp::Write,
          _ => panic!("Invalid top: {:?}", top),
        })
    }

    fn binop_from_egglog(&self, op: Term) -> BinaryOp {
        match_term_app!(op.clone();
        {
          ("Add", []) => BinaryOp::Add,
          ("Sub", []) => BinaryOp::Sub,
          ("Mul", []) => BinaryOp::Mul,
          ("Div", []) => BinaryOp::Div,
          ("Eq", []) => BinaryOp::Eq,
          ("Load", []) => BinaryOp::Load,
          ("LessThan", []) => BinaryOp::LessThan,
          ("GreaterThan", []) => BinaryOp::GreaterThan,
          ("LessEq", []) => BinaryOp::LessEq,
          ("GreaterEq", []) => BinaryOp::GreaterEq,
          ("And", []) => BinaryOp::And,
          ("Or", []) => BinaryOp::Or,
          ("PtrAdd", []) => BinaryOp::PtrAdd,
          ("Print", []) => BinaryOp::Print,
          ("Free", []) => BinaryOp::Free,
          _ => panic!("Invalid binary op: {:?}", op),
        })
    }

    fn uop_from_egglog(&self, uop: Term) -> UnaryOp {
        match_term_app!(uop.clone();
        {
          ("Not", []) => UnaryOp::Not,
          _ => panic!("Invalid unary op: {:?}", uop),
        })
    }

    fn expr_from_egglog(&self, expr: Term) -> RcExpr {
        match_term_app!(expr.clone();
        {
          ("Const", [constant, ty]) => {
            let constant = self.termdag.get(*constant);
            Rc::new(Expr::Const(self.const_from_egglog(constant), self.type_from_egglog(self.termdag.get(*ty))))
          }
          ("Top", [op, lhs, mid, rhs]) => {
            let op = self.termdag.get(*op);
            let lhs = self.termdag.get(*lhs);
            let mid = self.termdag.get(*mid);
            let rhs = self.termdag.get(*rhs);
            Rc::new(Expr::Top(
              self.top_from_egglog(op),
              self.expr_from_egglog(lhs),
              self.expr_from_egglog(mid),
              self.expr_from_egglog(rhs),
            ))
          }
          ("Bop", [op, lhs, rhs]) => {
            let op = self.termdag.get(*op);
            let lhs = self.termdag.get(*lhs);
            let rhs = self.termdag.get(*rhs);
            Rc::new(Expr::Bop(
              self.binop_from_egglog(op),
              self.expr_from_egglog(lhs),
              self.expr_from_egglog(rhs),
            ))
          }
          ("Uop", [op, expr]) => {
            let op = self.termdag.get(*op);
            let expr = self.termdag.get(*expr);
            Rc::new(Expr::Uop(
              self.uop_from_egglog(op),
              self.expr_from_egglog(expr),
            ))
          }
          ("Get", [expr, index]) => {
            let expr = self.termdag.get(*expr);
            let index = self.termdag.get(*index);
            let Term::Lit(Literal::Int(index)) = index else {
              panic!("Invalid index: {:?}", index)
            };
            Rc::new(Expr::Get(
              self.expr_from_egglog(expr),
              index.try_into().unwrap(),
            ))
          }
          ("Alloc", [expr, state, type_]) => {
            let expr = self.termdag.get(*expr);
            let basetype = self.termdag.get(*type_);
            let state = self.termdag.get(*state);
            Rc::new(Expr::Alloc(
              self.expr_from_egglog(expr),
              self.expr_from_egglog(state),
              self.basetype_from_egglog(basetype),
            ))
          }
          ("Call", [lit, expr]) => {
            let Term::Lit(Literal::String(string)) = self.termdag.get(*lit) else {
              panic!("Invalid string: {:?}", lit)
            };
            let expr = self.termdag.get(*expr);
            Rc::new(Expr::Call(
              string.to_string(),
              self.expr_from_egglog(expr),
            ))
          }
          ("Empty", [ty]) => Rc::new(Expr::Empty(self.type_from_egglog(self.termdag.get(*ty)))),
          ("Single", [expr]) => {
            let expr = self.termdag.get(*expr);
            Rc::new(Expr::Single(self.expr_from_egglog(expr)))
          }
          ("Concat", [order, lhs, rhs]) => {
            let order = self.termdag.get(*order);
            let lhs = self.termdag.get(*lhs);
            let rhs = self.termdag.get(*rhs);
            Rc::new(Expr::Concat(
              self.order_from_egglog(order),
              self.expr_from_egglog(lhs),
              self.expr_from_egglog(rhs),
            ))
          }
          ("Switch", [expr, exprs]) => {
            let expr = self.termdag.get(*expr);
            let exprs = self.termdag.get(*exprs);
            Rc::new(Expr::Switch(
              self.expr_from_egglog(expr),
              self.vec_from_listexpr(exprs),
            ))
          }
          ("If", [cond, then_, else_]) => {
            let cond = self.termdag.get(*cond);
            let then_ = self.termdag.get(*then_);
            let else_ = self.termdag.get(*else_);
            Rc::new(Expr::If(
              self.expr_from_egglog(cond),
              self.expr_from_egglog(then_),
              self.expr_from_egglog(else_),
            ))
          }
          ("Let", [lhs, rhs]) => {
            let lhs = self.termdag.get(*lhs);
            let rhs = self.termdag.get(*rhs);
            Rc::new(Expr::Let(
              self.expr_from_egglog(lhs),
              self.expr_from_egglog(rhs),
            ))
          }
          ("DoWhile", [cond, body]) => {
            let cond = self.termdag.get(*cond);
            let body = self.termdag.get(*body);
            Rc::new(Expr::DoWhile(
              self.expr_from_egglog(cond),
              self.expr_from_egglog(body),
            ))
          }
          ("Arg", [ty]) => {
            let type_ = self.termdag.get(*ty);
            Rc::new(Expr::Arg(self.type_from_egglog(type_)))
          }
          ("InContext", [assumption, expr]) => {
            let assumption = self.termdag.get(*assumption);
            let expr = self.termdag.get(*expr);
            Rc::new(Expr::InContext(
              self.assumption_from_egglog(assumption),
              self.expr_from_egglog(expr),
            ))
          }
          ("Function", [lit, type1, type2, expr]) => {
            let Term::Lit(Literal::String(string)) = self.termdag.get(*lit) else {
              panic!("Invalid string: {:?}", lit)
            };
            let type1 = self.termdag.get(*type1);
            let type2 = self.termdag.get(*type2);
            let expr = self.termdag.get(*expr);
            Rc::new(Expr::Function(
              string.to_string(),
              self.type_from_egglog(type1),
              self.type_from_egglog(type2),
              self.expr_from_egglog(expr),
            ))
          }
          _ => panic!("Invalid expr: {:?}", expr),
        })
    }

    pub fn program_from_egglog(&self, program: Term) -> TreeProgram {
        match_term_app!(program.clone();
        {
          ("Program", [entry, functions]) => {
            let entry = self.termdag.get(*entry);
            let others = self.termdag.get(*functions);
            let entry = self.expr_from_egglog(entry);
            let functions = self.vec_from_listexpr(others);
            TreeProgram { entry, functions }
          }
          _ => panic!("Invalid program: {:?}", program),
        })
    }
}
