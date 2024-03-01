use crate::schema::{BinaryOp, UnaryOp};
use crate::schema_helpers::{Constructor, Purpose, Sort};
use std::iter;
use strum::IntoEnumIterator;

fn bop_is_pure(bop: &BinaryOp) -> bool {
    use BinaryOp::*;
    match bop {
        Add | Sub | Mul | LessThan | And | Or | Div | PtrAdd | Eq | GreaterThan => true,
        Write => false,
    }
}

fn uop_is_pure(uop: &UnaryOp) -> bool {
    use UnaryOp::*;
    match uop {
        Not => true,
        Print | Load => false,
    }
}

// Builds rules like:
// (rule ((Bop op x y) (BinaryOpIsPure op) (ExprIsPure x) (ExprIsPure y))
//       ((ExprIsPure (Bop op x y)))
//       :ruleset always-run)
fn purity_rules_for_ctor(ctor: Constructor) -> String {
    use Constructor::*;
    match ctor {
        Function | Const | Get | Concat | Single | Switch | If | DoWhile | Let | Arg | Empty
        | Cons | Nil | InContext | Bop | Uop => {
            // e.g. ["(ExprIsPure x)", "(ExprIsPure y)"]
            let children_pure_queries = ctor.filter_map_fields(|field| match field.purpose {
                Purpose::Static(Sort::BinaryOp)
                | Purpose::Static(Sort::UnaryOp)
                | Purpose::SubExpr
                | Purpose::SubListExpr
                | Purpose::CapturedExpr => Some(format!(
                    "({sort}IsPure {var})",
                    sort = field.sort().name(),
                    var = field.var()
                )),
                Purpose::Static(_) => None,
            });

            // e.g. "(Bop op x y)"
            let ctor_pattern = ctor.construct(|field| field.var());

            let queries = iter::once(ctor_pattern.clone())
                .chain(children_pure_queries)
                .collect::<Vec<_>>()
                .join(" ");

            let sort = ctor.sort().name();
            format!(
                "
                (rule ({queries})
                      (({sort}IsPure {ctor_pattern}))
                      :ruleset always-run)"
            )
        }
        // Call also requires the function to be pure
        Call => "
            (rule ((Call _f _arg) (ExprIsPure _arg) (ExprIsPure (Function _f inty outty out)))
                  ((ExprIsPure (Call _f _arg)))
                  :ruleset always-run)"
            .to_string(),
        Alloc => "".to_string(),
    }
}

pub(crate) fn rules() -> Vec<String> {
    iter::once(
        "
        (relation ExprIsPure (Expr))
        (relation ListExprIsPure (ListExpr))
        (relation BinaryOpIsPure (BinaryOp))
        (relation UnaryOpIsPure (UnaryOp))"
            .to_string(),
    )
    .chain(BinaryOp::iter().filter_map(|bop| {
        bop_is_pure(&bop).then(|| format!("(BinaryOpIsPure ({name}))", name = bop.name()))
    }))
    .chain(UnaryOp::iter().filter_map(|uop| {
        uop_is_pure(&uop).then(|| format!("(UnaryOpIsPure ({name}))", name = uop.name()))
    }))
    .chain(Constructor::iter().map(purity_rules_for_ctor))
    .collect::<Vec<String>>()
}

#[cfg(test)]
use crate::ast::*;
#[cfg(test)]
use crate::schema::Constant;
#[cfg(test)]
use crate::Value;

#[test]
fn test_purity_analysis() -> crate::Result {
    let pureloop = dowhile(
        in_context(inlet(int(2)), single(int(1))),
        parallel!(
            less_than(get(looparg(), 0), int(3)),
            get(switch!(int(0); parallel!(int(4), int(5))), 0)
        ),
    )
    .with_arg_types(emptyt(), tuplet!(intt()));
    let impureloop = dowhile(
        in_context(inlet(int(2)), single(int(1))),
        parallel!(
            less_than(get(looparg(), 0), int(3)),
            get(
                switch!(load(alloc(int(0), intt())); parallel!(int(4), int(5))),
                0
            )
        ),
    )
    .with_arg_types(emptyt(), tuplet!(intt()));
    let build = format!("{pureloop} {impureloop}");
    let check = format!(
        "
        (check (ExprIsPure {pureloop}))
        (fail (check (ExprIsPure {impureloop})))
    "
    );
    crate::egglog_test(
        &build,
        &check,
        vec![pureloop.to_program(emptyt(), tuplet!(intt()))],
        Value::Tuple(vec![]),
        Value::Tuple(vec![Value::Const(Constant::Int(4))]),
        vec![],
    )
}
