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
            Purpose::ReferencingId => Some(format!(
                "(rule ({pat})
                       (({sort}HasRefId {pat} {field_var}))
                       :ruleset always-run)"
            )),
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
    let id_check = vec!["
(rule ((ExprHasRefId x id1)
       (ExprHasRefId x id2)
       (!= id1 id2))
      ((panic \"Ref ids don't match\"))
      :ruleset error-checking)
(rule ((ListExprHasRefId x id1)
       (ListExprHasRefId x id2)
       (!= id1 id2))
      ((panic \"Ref ids don't match\"))
      :ruleset error-checking)
    "
    .to_string()];

    ESort::iter()
        .map(|sort| "(relation *HasRefId (* IdSort))".replace('*', sort.name()))
        .chain(Constructor::iter().map(id_analysis_rules_for_ctor))
        .chain(id_check)
        .collect::<Vec<_>>()
}

#[test]
fn test_id_analysis() {
    print!("{}", id_analysis_rules().join("\n"));
}
