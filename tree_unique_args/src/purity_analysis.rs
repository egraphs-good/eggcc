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
        Constructor::Add => Purity::PureIfChildrenAre,
        Constructor::Print => Purity::Impure,
        Constructor::Loop => Purity::PureIfChildrenAre,
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
