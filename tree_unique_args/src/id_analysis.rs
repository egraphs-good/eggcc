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
                "(rule ({pat} ({sort}IsValid {pat}))
                       (({sort}HasRefId {pat} {field_var}))
                       :ruleset always-run)"
            )),

            // Constructor has referencing id of its subexpr fields
            Purpose::SubExpr | Purpose::SubListExpr => Some(format!(
                "(rule ({pat} ({field_sort}HasRefId {field_var} ref-id) ({sort}IsValid {pat}))
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
fn test_id_analysis() -> crate::Result {
    let build = "
        (let outer-id (Id (i64-fresh!)))
        (let let0-id (Id (i64-fresh!)))
        (let let1-id (Id (i64-fresh!)))
        (ExprIsValid (Let
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
                    ))))
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

// Check that invalid expr (expr that has not been marked valid) does not have a RefId
#[test]
fn test_id_analysis_no_invalid_entry() {
    let build = "(let some-expr (Not (Boolean (Id (i64-fresh!)) false))";
    let check = "(fail (check (ExprHasRefId some-expr any-id)))";

    let _ = crate::run_test(build, check);
}

// Create an id conflict for an Expr on purpose and check that we catch it
#[test]
#[should_panic]
fn test_id_analysis_expr_id_conflict_panics_if_valid() {
    let build = "
        (let id1 (Id (i64-fresh!)))
        (let id2 (Id (i64-fresh!)))
        (let conflict-expr (And (Boolean id1 false) (Boolean id2 true)))
        (ExprIsValid conflict-expr)";
    let check = "";

    let _ = crate::run_test(build, check);
}

#[test]
#[should_panic]
// Create an id conflict for a ListExpr on purpose and check that we catch it
fn test_id_analysis_listexpr_id_conflict_panics() {
    let build = "
        (let id1 (Id (i64-fresh!)))
        (let id2 (Id (i64-fresh!)))
        (let conflict-expr (Cons (Num id1 3) (Cons (UnitExpr id2) (Nil))))
        (ListExprIsValid conflict-expr)";
    let check = "";

    let _ = crate::run_test(build, check);
}
