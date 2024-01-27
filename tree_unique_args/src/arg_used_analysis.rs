use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn arg_used_rule_for_ctor(ctor: Constructor) -> Option<String> {
    if ctor == Constructor::Arg {
        return Some(format!(
            "
            (rule (
                (ExprUsesArgs-demand e)
                (= e (Get (Arg id) i))
            ) (
                (set (ExprUsesArgs e) (set-of i))
            )
            :ruleset always-run)"
        ));
    }

    let children_queries = ctor
        .filter_map_fields(|field| match field.purpose {
            Purpose::Static(_)
            | Purpose::CapturingId
            | Purpose::ReferencingId
            | Purpose::CapturedExpr => None,
            Purpose::SubExpr | Purpose::SubListExpr => {
                let var = field.var();
                let sort = field.sort().name();
                Some(format!("(= args-{var} ({sort}UsesArgs {var}))"))
            }
        })
        .join(" ");

    let children_demand = ctor
        .filter_map_fields(|field| match field.purpose {
            Purpose::Static(_)
            | Purpose::CapturingId
            | Purpose::ReferencingId
            | Purpose::CapturedExpr => None,
            Purpose::SubExpr | Purpose::SubListExpr => {
                let var = field.var();
                let sort = field.sort().name();
                Some(format!("({sort}UsesArgs-demand {var})"))
            }
        })
        .join(" ");

    let fields = ctor
        .fields()
        .into_iter()
        .filter(|field| field.purpose == Purpose::SubExpr || field.purpose == Purpose::SubListExpr)
        .collect::<Vec<_>>();
    let union_expr = match fields.len() {
        0 => return None,
        1 => format!("args-{}", fields[0].var()),
        _ => {
            let mut union_expr = vec![];
            let (last_field, fields) = fields.split_last().unwrap();
            for field in fields {
                let var = field.var();
                union_expr.push(format!("(set-union args-{var} "));
            }
            union_expr.push(format!("args-{}", last_field.var()));
            for _ in fields {
                union_expr.push(")".into());
            }
            union_expr.join(" ")
        }
    };

    let ctor_pattern = ctor.construct(|field| field.var());

    let sort = ctor.sort().name();
    Some(format!(
        "
        ;; propagation of demand
        (rule (
            ({sort}UsesArgs-demand e)
            (= e {ctor_pattern})
        ) (
            {children_demand}
        )
        :ruleset always-run)

        ;; collecting set of args
        (rule (
            ({sort}UsesArgs-demand e)
            (= e {ctor_pattern})
            {children_queries}
        ) (
            (set ({sort}UsesArgs e) {union_expr})
        )
        :ruleset always-run)"
    ))
}

pub(crate) fn arg_used_analysis_rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| {
            "
            (function *UsesArgs (*) I64Set :merge (set-union old new))
            (relation *UsesArgs-demand (*))
            
            (rule ((*UsesArgs-demand e)) 
                  ((set (*UsesArgs e) (set-empty))) :ruleset always-run)
            "
            .replace('*', sort.name())
        })
        .chain(Constructor::iter().filter_map(arg_used_rule_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn test_args_used_analysis() -> Result<(), egglog::Error> {
    let build = &*"
    (let id1 (Id (i64-fresh!)))
    (let id2 (Id (i64-fresh!)))
    (let expr1
        (All (Parallel) (Pair (Let id2 (All (Parallel) (Pair (Get (Arg id1) 3) 
                                                             (Num id1 1)))
                                        (Get (Arg id2) 0))
                              (Add (Get (Arg id1) 1)
                                   (Get (Arg id1) 2)))))
    (ExprUsesArgs-demand expr1)
    "
    .to_string();
    let check = "
    (check (= (ExprUsesArgs expr1) (set-of 1 2 3)))
    ";
    crate::run_test(build, check)
}
