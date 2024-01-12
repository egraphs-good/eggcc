use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;


fn find_invariant_rule_for_ctor(ctor: Constructor) -> String {
    if ctor == Constructor::Arg {
        return "(rewrite (SubstExpr (Arg (Id id)) v) v :ruleset always-run)".to_string();
    }

    // e.g. "(Add x y)"
    let ctor_pattern = ctor.construct(|field| field.var());

    // e.g. "(Add (SubstExpr x v) (SubstExpr y v))"
    let substed_ctor = ctor.construct(|field| match field.purpose {
        Purpose::Static(_)
        | Purpose::CapturingId
        | Purpose::CapturedExpr
        | Purpose::ReferencingId => field.var(),
        Purpose::SubExpr | Purpose::SubListExpr => {
            let var = field.var();
            let sort = field.sort().name();
            format!("(Subst{sort} {var} v)")
        }
    });

    let sort = ctor.sort().name();
    let br = "\n         ";
    format!("(rewrite (Subst{sort} {ctor_pattern} v){br}{substed_ctor}{br}:ruleset always-run)")
}

pub(crate) fn subst_rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(function Subst* (* Expr) * :unextractable)".replace('*', sort.name()))
        .chain(Constructor::iter().map(subst_rule_for_ctor))
        .collect::<Vec<_>>()
}
