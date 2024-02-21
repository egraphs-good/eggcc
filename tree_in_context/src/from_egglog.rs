use std::rc::Rc;

use egglog::{ast::Literal, match_term_app, Term};

use crate::schema::{
    Assumption, BaseType, BinaryOp, Constant, Expr, Order, RcExpr, TreeProgram, Type, UnaryOp,
};

pub(crate) struct FromEgglog {
    pub(crate) termdag: egglog::TermDag,
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
          _ => panic!("Invalid basetype: {:?}", basetype),
        })
    }

    fn from_tlistexpr_reversed(&self, tlistexpr: Term) -> Vec<Type> {
        match_term_app!(tlistexpr.clone();
        {
          ("TNil", []) => vec![],
          ("TCons", [type_, tlistexpr]) => {
            let type_ = self.termdag.get(*type_);
            let tlistexpr = self.termdag.get(*tlistexpr);
            let mut rest = self.from_tlistexpr(tlistexpr);
            rest.push(self.type_from_egglog(type_));
            rest
          }
          _ => panic!("Invalid tlistexpr: {:?}", tlistexpr),
        })
    }

    fn from_tlistexpr(&self, tlistexpr: Term) -> Vec<Type> {
        let mut types = self.from_tlistexpr_reversed(tlistexpr);
        types.reverse();
        types
    }

    fn from_listexpr_reversed(&self, listexpr: Term) -> Vec<RcExpr> {
        match_term_app!(listexpr.clone();
        {
          ("Nil", []) => vec![],
          ("Cons", [expr, listexpr]) => {
            let expr = self.termdag.get(*expr);
            let listexpr = self.termdag.get(*listexpr);
            let rest = self.from_listexpr(listexpr);
            rest.into_iter().chain(std::iter::once(self.expr_from_egglog(expr))).collect()
          }
          _ => panic!("Invalid listexpr: {:?}", listexpr),
        })
    }

    fn from_listexpr(&self, listexpr: Term) -> Vec<RcExpr> {
        let mut exprs = self.from_listexpr_reversed(listexpr);
        exprs.reverse();
        exprs
    }

    fn type_from_egglog(&self, type_: Term) -> Type {
        match_term_app!(type_.clone(); {
          ("Base", [basetype]) => Type::Base(self.basetype_from_egglog(self.termdag.get(*basetype))),
          ("PointerT", [basetype]) => Type::PointerT(self.basetype_from_egglog(self.termdag.get(*basetype))),
          ("TupleT", [types]) => {
            let types = self.termdag.get(*types);
            Type::TupleT(self.from_tlistexpr(types))
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

    fn binop_from_egglog(&self, op: Term) -> BinaryOp {
        match_term_app!(op.clone();
        {
          ("Add", []) => BinaryOp::Add,
          ("Sub", []) => BinaryOp::Sub,
          ("Mul", []) => BinaryOp::Mul,
          ("Div", []) => BinaryOp::Div,
          ("Eq", []) => BinaryOp::Eq,
          ("LessThan", []) => BinaryOp::LessThan,
          ("GreaterThan", []) => BinaryOp::GreaterThan,
          ("And", []) => BinaryOp::And,
          ("Or", []) => BinaryOp::Or,
          ("Write", []) => BinaryOp::Write,
          ("PtrAdd", []) => BinaryOp::PtrAdd,
          _ => panic!("Invalid binary op: {:?}", op),
        })
    }

    fn uop_from_egglog(&self, uop: Term) -> UnaryOp {
        match_term_app!(uop.clone();
        {
          ("Not", []) => UnaryOp::Not,
          ("Print", []) => UnaryOp::Print,
          ("Load", []) => UnaryOp::Load,
          _ => panic!("Invalid unary op: {:?}", uop),
        })
    }

    fn expr_from_egglog(&self, expr: Term) -> RcExpr {
        match_term_app!(expr.clone();
        {
          ("Const", [constant]) => {
            let constant = self.termdag.get(*constant);
            Rc::new(Expr::Const(self.const_from_egglog(constant)))
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
          ("Alloc", [expr, type_]) => {
            let expr = self.termdag.get(*expr);
            let type_ = self.termdag.get(*type_);
            Rc::new(Expr::Alloc(
              self.expr_from_egglog(expr),
              self.type_from_egglog(type_),
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
          ("Empty", []) => Rc::new(Expr::Empty),
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
              self.from_listexpr(exprs),
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
          ("Arg", [type_]) => {
            let type_ = self.termdag.get(*type_);
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

    pub(crate) fn program_from_egglog(&self, program: Term) -> TreeProgram {
        match_term_app!(program.clone();
        {
          ("Program", [entry, functions]) => {
            let entry = self.termdag.get(*entry);
            let others = self.termdag.get(*functions);
            let entry = self.expr_from_egglog(entry);
            let functions = self.from_listexpr(others);
            TreeProgram { entry, functions }
          }
          _ => panic!("Invalid program: {:?}", program),
        })
    }
}
