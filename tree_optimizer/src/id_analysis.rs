use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn id_analysis_rules_for_ctor(ctor: Constructor) -> String {
    let pat = ctor.construct(|field| field.var());
    let sort = ctor.sort().name();
    ctor.filter_map_fields(|field| {
        let field_var = field.var();
        let field_sort = field.sort().name();
        match field.purpose {
            Purpose::Static(_) | Purpose::CapturingId => None,
            Purpose::CapturedExpr => {
                // If the captured expr is shared, then this expr
                // is also shared.
                Some(format!(
                    "(rule ({pat}
                            ({field_sort}HasRefId {field_var} (Shared))  
                            ({sort}IsValid {pat}))
                           (({sort}HasRefId {pat} (Shared)))
                    :ruleset always-run)"
                ))
            }
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
            [
                format!("
(relation {sort}HasRefId ({sort} IdSort))

(relation {sort}IsValidShared ({sort}))
(relation {sort}IsValidUnique ({sort}))
(rule (({sort}IsValid expr)
       ({sort}HasRefId expr (Shared)))
      (({sort}IsValidShared expr))
        :ruleset always-run)
(rule (({sort}IsValid expr)
       ({sort}HasRefId expr (Id id)))
      (({sort}IsValidUnique expr))
        :ruleset always-run)

;; Error checking - each (list)expr should only have a single ref id
(rule (({sort}HasRefId x id1)
       ({sort}HasRefId x id2)
       (!= id1 id2))
      ((panic \"Ref ids don't match\"))
        :ruleset error-checking)
            ")
])
        .chain(Constructor::iter().map(id_analysis_rules_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn test_id_analysis() -> Result<(), egglog::Error> {
    let build = "
        (let outer-id (Id (i64-fresh!)))
        (let let0-id (Id (i64-fresh!)))
        (let let1-id (Id (i64-fresh!)))
        (ExprIsValid (Let
            let0-id
            (Num outer-id 0)
            (All let0-id
                (Parallel)
                (Pair
                    (Let
                        let1-id
                        (Num let0-id 3)
                        (Boolean let1-id true))
                    (All let0-id (Parallel) (Nil))
                    ))))
    ";

    let check = "
        (check (ExprHasRefId (Num outer-id 0) outer-id))
        (check (ExprHasRefId (Boolean let1-id true) let1-id))
        (check (ExprHasRefId (All let0-id (Parallel) (Nil)) let0-id))
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
                    (All let0-id (Parallel) (Nil))
                    )
                let0-id
        ))
        (check (ExprHasRefId
            (Let
                let0-id
                (Num outer-id 0)
                (All
                    let0-id
                    (Parallel)
                    (Pair
                        (Let
                            let1-id
                            (Num let0-id 3)
                            (Boolean let1-id true))
                        (All let0-id (Parallel) (Nil))
                        )))
            outer-id
        ))
    ";

    crate::run_test(build, check)
}

// Check that invalid expr (expr that has not been marked valid) does not have a RefId
#[test]
fn test_id_analysis_no_invalid_entry() {
    let build = "(let some-expr (UOp (Not) (Boolean (Id (i64-fresh!)) false)))";
    let check = "(fail (check (ExprHasRefId some-expr any-id)))";

    crate::run_test(build, check).unwrap()
}

// Create an id conflict for an Expr on purpose and check that we catch it
#[test]
#[should_panic]
fn test_id_analysis_expr_id_conflict_panics_if_valid() {
    let build = "
        (let id1 (Id (i64-fresh!)))
        (let id2 (Id (i64-fresh!)))
        (let conflict-expr (BOp (And) (Boolean id1 false) (Boolean id2 true)))
        (ExprIsValid conflict-expr)";
    let check = "";

    crate::run_test(build, check).unwrap()
}

#[test]
#[should_panic]
// Create an id conflict for a ListExpr on purpose and check that we catch it
fn test_id_analysis_listexpr_id_conflict_panics() {
    let build = "
        (let id1 (Id (i64-fresh!)))
        (let id2 (Id (i64-fresh!)))
        (let conflict-expr (Cons (Num id1 3) (Cons (All id2 (Parallel) (Nil)) (Nil))))
        (ListExprIsValid conflict-expr)";
    let check = "";

    crate::run_test(build, check).unwrap()
}

#[test]
#[should_panic]
// Mix shared and unique ids and catch the panic
fn test_shared_unique_id_mix_panics() {
    let build = "
        (let idouter (Id (i64-fresh!)))
        (let id2 (Shared))
        (let conflict-expr 
             (Let id2 (Num idouter 0)
                (Num id2 1)))
        (ExprIsValid conflict-expr)";
    let check = "";
    crate::run_test(build, check).unwrap()
}
