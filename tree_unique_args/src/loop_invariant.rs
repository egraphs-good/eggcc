use crate::ir::{Constructor, Purpose, Sort};
use strum::IntoEnumIterator;

fn find_invariant_rule_for_ctor(ctor: Constructor) -> String {
    //let name = ctor.name();

    let ctor_pattern = ctor.construct(|field| field.var());

    let find_inv_ctor = ctor.construct_only_fields(|field| match field.purpose {
        Purpose::Static(Sort::I64) | Purpose::Static(Sort::Bool) => {
            format!("(set (is-inv-expr loop expr) true)")
        }
        Purpose::Static(_)
        | Purpose::CapturingId
        | Purpose::CapturedExpr
        | Purpose::ReferencingId => format!(""),
        Purpose::SubExpr | Purpose::SubListExpr => {
            let var = field.var();
            let sort = field.sort().name();
            format!("(find-inv-{sort} loop {var})")
        }
    });

    let br = "\n      ";
    format!("\n(rule ((find-inv-expr loop expr){br} (= expr {ctor_pattern}){br}({find_inv_ctor}) :ruleset always-run)")

}

pub(crate) fn find_inv_expr_rules() -> Vec<String> {
    Constructor::iter()
        .map(find_invariant_rule_for_ctor)
        .collect::<Vec<_>>()
}
