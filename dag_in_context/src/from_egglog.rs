//! Converts from an egglog AST directly to the rust representation of that AST.
//! Common subexpressions (common terms) must be converted to the same RcExpr (pointer equality).

use std::rc::Rc;

use egglog::{ast::Literal, match_term_app, Term};
use indexmap::IndexMap;

use crate::schema::{
    Assumption, BaseType, BinaryOp, Constant, Expr, RcExpr, TernaryOp, TreeProgram, Type, UnaryOp,
};

pub struct FromEgglog<'a> {
    pub termdag: &'a egglog::TermDag,
    pub conversion_cache: IndexMap<Term, RcExpr>,
}

pub fn program_from_egglog(program: Term, termdag: &egglog::TermDag) -> TreeProgram {
    let mut converter = FromEgglog {
        termdag,
        conversion_cache: IndexMap::new(),
    };
    converter.program_from_egglog(program)
}

pub fn program_from_egglog_preserve_ctx_nodes(
    program: Term,
    termdag: &mut egglog::TermDag,
) -> TreeProgram {
    let mut converter = FromEgglog {
        termdag,
        conversion_cache: IndexMap::new(),
    };
    converter.program_from_egglog(program)
}

impl<'a> FromEgglog<'a> {
    fn const_from_egglog(&mut self, constant: Term) -> Constant {
        match_term_app!(constant.clone(); {
          ("Int", [lit]) => {
            let Term::Lit(Literal::Int(integer)) = self.termdag.get(*lit) else {
              panic!("Invalid integer: {:?}", lit)
            };
            Constant::Int(*integer)
          }
          ("Bool", [lit]) => {
            let Term::Lit(Literal::Bool(boolean)) = self.termdag.get(*lit) else {
              panic!("Invalid boolean: {:?}", lit)
            };
            Constant::Bool(*boolean)
          }
          ("Float", [lit]) => {
            let Term::Lit(Literal::F64(f)) = self.termdag.get(*lit) else {
              panic!("Invalid float: {:?}", lit)
            };
            Constant::Float(*f)
          }
          _ => panic!("Invalid constant: {:?}", constant),
        })
    }

    fn basetype_from_egglog(&mut self, basetype: Term) -> BaseType {
        match_term_app!(basetype.clone(); {
          ("IntT", []) => BaseType::IntT,
          ("FloatT", []) => BaseType::FloatT,
          ("BoolT", []) => BaseType::BoolT,
          ("PointerT", [basetype]) => BaseType::PointerT(Box::new(self.basetype_from_egglog(self.termdag.get(*basetype).clone()))),
          ("StateT", []) => BaseType::StateT,
          _ => panic!("Invalid basetype: {:?}", basetype),
        })
    }

    fn vec_from_tlistexpr_helper(&mut self, tlistexpr: Term, acc: &mut Vec<BaseType>) {
        match_term_app!(tlistexpr.clone();
        {
          ("TNil", []) => (),
          ("TCons", [type_, tlistexpr]) => {
            let type_ = self.termdag.get(*type_);
            let tlistexpr = self.termdag.get(*tlistexpr);
            acc.push(self.basetype_from_egglog(type_.clone()));
            self.vec_from_tlistexpr_helper(tlistexpr.clone(), acc);
          }
          _ => panic!("Invalid tlistexpr: {:?}", tlistexpr),
        })
    }

    fn vec_from_tlistexpr(&mut self, tlistexpr: Term) -> Vec<BaseType> {
        let mut types = vec![];
        self.vec_from_tlistexpr_helper(tlistexpr, &mut types);
        types
    }

    fn vec_from_listexpr_helper(&mut self, listexpr: Term, acc: &mut Vec<RcExpr>) {
        match_term_app!(listexpr.clone();
        {
          ("Nil", []) => (),
          ("Cons", [expr, listexpr]) => {
            let expr = self.termdag.get(*expr);
            acc.push(self.expr_from_egglog(expr.clone()));
            let listexpr = self.termdag.get(*listexpr);
            self.vec_from_listexpr_helper(listexpr.clone(), acc);
          }
          _ => panic!("Invalid listexpr: {:?}", listexpr),
        })
    }

    fn vec_from_listexpr(&mut self, listexpr: Term) -> Vec<RcExpr> {
        let mut exprs = vec![];
        self.vec_from_listexpr_helper(listexpr, &mut exprs);
        exprs
    }

    pub(crate) fn type_from_egglog(&mut self, type_: Term) -> Type {
        match_term_app!(type_.clone(); {
          ("Base", [basetype]) => Type::Base(self.basetype_from_egglog(self.termdag.get(*basetype).clone())),
          ("TupleT", [types]) => {
            let types = self.termdag.get(*types);
            Type::TupleT(self.vec_from_tlistexpr(types.clone()))
          }
          _ => panic!("Invalid type: {:?}", type_),
        })
    }

    fn assumption_from_egglog(&mut self, assumption: Term) -> Assumption {
        match_term_app!(assumption.clone();
        {
          ("InLoop", [lhs, rhs]) => {
            Assumption::InLoop(
              self.expr_from_egglog(self.termdag.get(*lhs).clone()),
              self.expr_from_egglog(self.termdag.get(*rhs).clone()),
            )
          }
          ("InFunc", [str]) => {
            let Term::Lit(Literal::String(name)) = self.termdag.get(*str)
            else {
              panic!("Invalid function name in InFunc: {:?}", str)
            };
            Assumption::InFunc(name.to_string())
          }
          ("InIf", [is_then, pred_expr, input_expr]) => {
            let Term::Lit(Literal::Bool(boolean)) = self.termdag.get(*is_then)
            else {
              panic!("Invalid boolean: {:?}", is_then)
            };
            Assumption::InIf(boolean.clone(), self.expr_from_egglog(self.termdag.get(*pred_expr).clone()), self.expr_from_egglog(self.termdag.get(*input_expr).clone()))
          }
          (name, _) => {
            eprintln!("Invalid assumption: {:?}", assumption);
            Assumption::WildCard(name.into())
          }
        })
    }

    fn top_from_egglog(&mut self, top: Term) -> TernaryOp {
        match_term_app!(top.clone();
        {
          ("Write", []) => TernaryOp::Write,
          ("Select", []) => TernaryOp::Select,
          _ => panic!("Invalid top: {:?}", top),
        })
    }

    fn binop_from_egglog(&mut self, op: Term) -> BinaryOp {
        match_term_app!(op.clone();
        {
          ("Add", []) => BinaryOp::Add,
          ("Sub", []) => BinaryOp::Sub,
          ("Mul", []) => BinaryOp::Mul,
          ("Div", []) => BinaryOp::Div,
          ("Eq", []) => BinaryOp::Eq,
          ("LessThan", []) => BinaryOp::LessThan,
          ("GreaterThan", []) => BinaryOp::GreaterThan,
          ("LessEq", []) => BinaryOp::LessEq,
          ("GreaterEq", []) => BinaryOp::GreaterEq,
          ("Smax", []) => BinaryOp::Smax,
          ("Smin", []) => BinaryOp::Smin,
          ("Shl", []) => BinaryOp::Shl,
          ("Shr", []) => BinaryOp::Shr,
          ("FAdd", []) => BinaryOp::FAdd,
          ("FSub", []) => BinaryOp::FSub,
          ("FMul", []) => BinaryOp::FMul,
          ("FDiv", []) => BinaryOp::FDiv,
          ("FEq", []) => BinaryOp::FEq,
          ("FLessThan", []) => BinaryOp::FLessThan,
          ("FGreaterThan", []) => BinaryOp::FGreaterThan,
          ("FLessEq", []) => BinaryOp::FLessEq,
          ("FGreaterEq", []) => BinaryOp::FGreaterEq,
          ("Fmax", []) => BinaryOp::Fmax,
          ("Fmin", []) => BinaryOp::Fmin,
          ("And", []) => BinaryOp::And,
          ("Or", []) => BinaryOp::Or,
          ("PtrAdd", []) => BinaryOp::PtrAdd,
          ("Load", []) => BinaryOp::Load,
          ("Print", []) => BinaryOp::Print,
          ("Free", []) => BinaryOp::Free,
          _ => panic!("Invalid binary op: {:?}", op),
        })
    }

    fn uop_from_egglog(&mut self, uop: Term) -> UnaryOp {
        match_term_app!(uop.clone();
        {
          ("Not", []) => UnaryOp::Not,
          _ => panic!("Invalid unary op: {:?}", uop),
        })
    }

    pub fn expr_from_egglog(&mut self, expr: Term) -> RcExpr {
        if let Some(expr) = self.conversion_cache.get(&expr) {
            return expr.clone();
        }
        let res = match_term_app!(expr.clone();
        {
          ("Const", [constant, ty, ctx]) => {
            let constant = self.termdag.get(*constant);
            Rc::new(Expr::Const(self.const_from_egglog(constant.clone()), self.type_from_egglog(self.termdag.get(*ty).clone()), self.assumption_from_egglog(self.termdag.get(*ctx).clone())))
          }
          ("Top", [op, lhs, mid, rhs]) => {
            let op = self.termdag.get(*op);
            let lhs = self.termdag.get(*lhs);
            let mid = self.termdag.get(*mid);
            let rhs = self.termdag.get(*rhs);
            Rc::new(Expr::Top(
              self.top_from_egglog(op.clone()),
              self.expr_from_egglog(lhs.clone()),
              self.expr_from_egglog(mid.clone()),
              self.expr_from_egglog(rhs.clone()),
            ))
          }
          ("Bop", [op, lhs, rhs]) => {
            let op = self.termdag.get(*op);
            let lhs = self.termdag.get(*lhs);
            let rhs = self.termdag.get(*rhs);
            Rc::new(Expr::Bop(
              self.binop_from_egglog(op.clone()),
              self.expr_from_egglog(lhs.clone()),
              self.expr_from_egglog(rhs.clone()),
            ))
          }
          ("Uop", [op, expr]) => {
            let op = self.termdag.get(*op);
            let expr = self.termdag.get(*expr);
            Rc::new(Expr::Uop(
              self.uop_from_egglog(op.clone()),
              self.expr_from_egglog(expr.clone()),
            ))
          }
          ("Get", [expr, index]) => {
            let expr = self.termdag.get(*expr);
            let index = self.termdag.get(*index);
            let Term::Lit(Literal::Int(index)) = index else {
              panic!("Invalid index: {:?}", index)
            };
            Rc::new(Expr::Get(
              self.expr_from_egglog(expr.clone()),
              (*index).try_into().unwrap(),
            ))
          }
          ("Alloc", [alloc_id, expr, state, type_]) => {
            let alloc_id = self.termdag.get(*alloc_id);
            let Term::Lit(Literal::Int(alloc_id)) = alloc_id else {
              panic!("Invalid alloc_id: {:?}", alloc_id)
            };
            let expr = self.termdag.get(*expr);
            let basetype = self.termdag.get(*type_);
            let state = self.termdag.get(*state);
            Rc::new(Expr::Alloc(
              alloc_id.clone(),
              self.expr_from_egglog(expr.clone()),
              self.expr_from_egglog(state.clone()),
              self.basetype_from_egglog(basetype.clone()),
            ))
          }
          ("Call", [lit, expr]) => {
            let Term::Lit(Literal::String(string)) = self.termdag.get(*lit) else {
              panic!("Invalid string: {:?}", lit)
            };
            let expr = self.termdag.get(*expr);
            Rc::new(Expr::Call(
              string.to_string(),
              self.expr_from_egglog(expr.clone()),
            ))
          }
          ("Empty", [ty, ctx]) => Rc::new(Expr::Empty(self.type_from_egglog(self.termdag.get(*ty).clone()), self.assumption_from_egglog(self.termdag.get(*ctx).clone()))),
          ("Single", [expr]) => {
            let expr = self.termdag.get(*expr);
            Rc::new(Expr::Single(self.expr_from_egglog(expr.clone())))
          }
          ("Concat", [lhs, rhs]) => {
            let lhs = self.termdag.get(*lhs);
            let rhs = self.termdag.get(*rhs);
            Rc::new(Expr::Concat(
              self.expr_from_egglog(lhs.clone()),
              self.expr_from_egglog(rhs.clone()),
            ))
          }
          ("Switch", [expr, expr2, exprs]) => {
            let expr = self.termdag.get(*expr);
            let expr2 = self.termdag.get(*expr2);
            let exprs = self.termdag.get(*exprs);
            Rc::new(Expr::Switch(
              self.expr_from_egglog(expr.clone()),
              self.expr_from_egglog(expr2.clone()),
              self.vec_from_listexpr(exprs.clone()),
            ))
          }
          ("If", [cond, input, then_, else_]) => {
            let cond = self.termdag.get(*cond);
            let input = self.termdag.get(*input);
            let then_ = self.termdag.get(*then_);
            let else_ = self.termdag.get(*else_);
            Rc::new(Expr::If(
              self.expr_from_egglog(cond.clone()),
              self.expr_from_egglog(input.clone()),
              self.expr_from_egglog(then_.clone()),
              self.expr_from_egglog(else_.clone()),
            ))
          }
          ("DoWhile", [cond, body]) => {
            let cond = self.termdag.get(*cond);
            let body = self.termdag.get(*body);
            Rc::new(Expr::DoWhile(
              self.expr_from_egglog(cond.clone()),
              self.expr_from_egglog(body.clone()),
            ))
          }
          ("Arg", [ty, ctx]) => {
            let type_ = self.termdag.get(*ty);
            Rc::new(Expr::Arg(self.type_from_egglog(type_.clone()), self.assumption_from_egglog(self.termdag.get(*ctx).clone())))
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
              self.type_from_egglog(type1.clone()),
              self.type_from_egglog(type2.clone()),
              self.expr_from_egglog(expr.clone()),
            ))
          }
          _ => panic!("Invalid expr: {:?}", expr),
        });

        self.conversion_cache.insert(expr, res.clone());
        res
    }

    /// Converts a term back into a TreeProgram, but removes context nodes along the way.
    /// This is crutial for the correctness of this conversion, since context nodes can break sharing
    /// of the state edge.
    pub fn program_from_egglog(&mut self, program: Term) -> TreeProgram {
        match_term_app!(program.clone();
        {
          ("Program", [entry, functions]) => {
            let entry = self.termdag.get(*entry);
            let others = self.termdag.get(*functions);
            let entry = self.expr_from_egglog(entry.clone());
            let functions = self.vec_from_listexpr(others.clone());
            TreeProgram { entry, functions }
          }
          _ => panic!("Invalid program: {:?}", program),
        })
    }
}
