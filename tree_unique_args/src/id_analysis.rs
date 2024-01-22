use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn id_analysis_rules_for_ctor(ctor: Constructor) -> String {
    let pat = ctor.construct(|field| field.var());
    let sort = ctor.sort().name();
    ctor.filter_map_fields(|field| {
        let field_var = field.var();
        let field_sort = field.sort().name();
        match field.purpose {
            Purpose::Static(_) | Purpose::CapturedExpr | Purpose::CapturingId => None,

            // Base case: constructor has referencing id specified as a field
            Purpose::ReferencingId => Some(format!(
                "(rule ({pat})
                       (({sort}HasRefId {pat} {field_var}))
                       :ruleset always-run)"
            )),

            // Constructor has referencing id of its subexpr fields
            Purpose::SubExpr | Purpose::SubListExpr => Some(format!(
                "(rule ({pat} ({field_sort}HasRefId {field_var} ref-id))
                       (({sort}HasRefId {pat} ref-id))
                       :ruleset always-run)"
            )),
        }
    })
    .join("\n")
}

pub(crate) fn id_analysis_rules() -> Vec<String> {
    ESort::iter()
        .flat_map(|sort|

            // Declare relation for ref id
            ["(relation *HasRefId (* IdSort))".replace('*', sort.name()),

            // Error checking - each (list)expr should only have a single ref id
            "(rule ((*HasRefId x id1)
                (*HasRefId x id2)
                (!= id1 id2))
                ((panic \"Ref ids don't match\"))
                :ruleset error-checking)".replace('*', sort.name())])
        .chain(Constructor::iter().map(id_analysis_rules_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn test_id_analysis() -> Result<(), egglog::Error> {
    let build = "
        (let outer-id (Id (i64-fresh!)))
        (let let0-id (Id (i64-fresh!)))
        (let let1-id (Id (i64-fresh!)))
        (Let
            let0-id
            (Num outer-id 0)
            (All
                (Parallel)
                (Pair
                    (Let
                        let1-id
                        (Num let0-id 3)
                        (Boolean let1-id true))
                    (UnitExpr let0-id)
                    )))
    ";

    let check = "
        (check (ExprHasRefId (Num outer-id 0) outer-id))
        (check (ExprHasRefId (Boolean let1-id true) let1-id))
        (check (ExprHasRefId (UnitExpr let0-id) let0-id))
        (check (ExprHasRefId 
                (Let
                        let1-id
                        (Num let0-id 3)
                        (Boolean let1-id true))
                let0-id))
        (check (ListExprHasRefId
                (Pair
                    (Let
                        let1-id
                        (Num let0-id 3)
                        (Boolean let1-id true))
                    (UnitExpr let0-id)
                    )
                let0-id
        ))
        (check (ExprHasRefId
            (Let
                let0-id
                (Num outer-id 0)
                (All
                    (Parallel)
                    (Pair
                        (Let
                            let1-id
                            (Num let0-id 3)
                            (Boolean let1-id true))
                        (UnitExpr let0-id)
                        )))
            outer-id
        ))
    ";

    crate::run_test(build, check)
}
