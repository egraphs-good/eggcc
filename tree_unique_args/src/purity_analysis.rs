use crate::ir::{Constructor, Sort, CONSTRUCTORS};
use crate::util;

enum Purity {
    Pure,
    Impure,
    PureIfChildrenAre,
}

fn purity(ctor: &Constructor) -> Purity {
    match ctor {
        Constructor::Num => Purity::Pure,
        Constructor::Boolean => Purity::Pure,
        Constructor::UnitExpr => Purity::Pure,
        Constructor::Add => Purity::PureIfChildrenAre,
        Constructor::Sub => Purity::PureIfChildrenAre,
        Constructor::Mul => Purity::PureIfChildrenAre,
        Constructor::LessThan => Purity::PureIfChildrenAre,
        Constructor::And => Purity::PureIfChildrenAre,
        Constructor::Or => Purity::PureIfChildrenAre,
        Constructor::Not => Purity::PureIfChildrenAre,
        Constructor::Get => Purity::PureIfChildrenAre,
        Constructor::Print => Purity::Impure,
        Constructor::Read => Purity::Impure,
        Constructor::Write => Purity::Impure,
        Constructor::All => Purity::PureIfChildrenAre,
        Constructor::Switch => Purity::PureIfChildrenAre,
        Constructor::Loop => Purity::PureIfChildrenAre,
        Constructor::Body => Purity::PureIfChildrenAre,
        Constructor::Arg => Purity::PureIfChildrenAre,
        Constructor::Call => Purity::Impure,
        Constructor::Cons => Purity::PureIfChildrenAre,
        Constructor::Nil => Purity::Pure,
    }
}

fn purity_queries(sort: &Sort, var: &str) -> Vec<String> {
    match sort {
        Sort::Bool => vec![],
        Sort::I64 => vec![],
        Sort::Order => vec![],
        Sort::Expr | Sort::ListExpr => {
            let sort_name = sort.name();
            vec![format!("({sort_name}IsPure {var})")]
        }
    }
}

fn purity_rules_for_ctor(ctor: &Constructor) -> Vec<String> {
    let name = ctor.name();
    let sort_name = ctor.sort().name();
    match purity(ctor) {
        Purity::Pure => {
            let args_vars = util::n_vars(ctor.num_params(), "arg");
            let args_vars_s = args_vars.join(" ");
            vec![format!(
                "(rule (({name} {args_vars_s}))
                       (({sort_name}IsPure ({name} {args_vars_s})))
                       :ruleset fast-analyses)"
            )]
        }
        Purity::PureIfChildrenAre => {
            let args_vars = util::n_vars(ctor.num_params(), "arg");
            let args_vars_s = args_vars.join(" ");
            let args_purity_queries = ctor
                .param_sorts()
                .iter()
                .zip(args_vars)
                .map(|(sort, var)| purity_queries(sort, &var).join(" "))
                .collect::<Vec<String>>()
                .join(" ");
            vec![format!(
                "(rule (({name} {args_vars_s}) {args_purity_queries})
                       (({sort_name}IsPure ({name} {args_vars_s})))
                       :ruleset fast-analyses)"
            )]
        }
        Purity::Impure => vec![],
    }
}

pub(crate) fn purity_analysis_rules() -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for sort in [Sort::Expr, Sort::ListExpr] {
        let sort_name = sort.name();
        res.push(format!("(relation {sort_name}IsPure ({sort_name}))"));
    }
    for ctor in CONSTRUCTORS {
        match ctor.sort() {
            Sort::Expr | Sort::ListExpr => res.extend(purity_rules_for_ctor(&ctor).into_iter()),
            _ => (),
        }
    }
    res
}

#[test]
fn test_purity_analysis() -> Result<(), egglog::Error> {
    let build = &*format!(
        "
        (let id1 (i64-fresh!))
        (let pure-loop
            (Loop id1
                (All (Parallel) (Pair (Num 0) (Num 0)))
                (All (Sequential) (Pair
                    ; pred
                    (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
                    ; output
                    (All (Parallel) (Pair
                        (Add (Get (Arg id1) 0) (Num 1))
                        (Sub (Get (Arg id1) 1) (Num 1))))))))

        (let id2 (i64-fresh!))
        (let impure-loop
            (Loop id2
                (All (Parallel) (Pair (Num 0) (Num 0)))
                (All (Sequential) (Pair
                    ; pred
                    (LessThan (Get (Arg id2) 0) (Get (Arg id2) 1))
                    ; output
                    (IgnoreFirst
                        (Print (Num 1))
                        (All (Parallel) (Pair
                            (Add (Get (Arg id2) 0) (Num 1))
                            (Sub (Get (Arg id2) 1) (Num 1)))))))))
    "
    );
    let check = "
        (check (ExprIsPure pure-loop))
        (fail (check (ExprIsPure impure-loop)))
    ";
    crate::run_test(build, check)
}
