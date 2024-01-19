use crate::ir::{Constructor, ESort};
use strum::IntoEnumIterator;

fn id_analysis_rule_for_ctor(ctor: Constructor) -> Option<String> {
    if ctor.fields().is_empty() {
        None
    } else {
        let ctor_pattern = ctor.construct(|field| field.var());
        // TODO: this should be preceded by an underscore
        let first_var = ctor.fields()[0].var();
        let var_sort = ctor.fields()[0].sort().name();
        let sort = ctor.sort().name();

        Some(
            match ctor {
                // Base cases: Num, Boolean, UnitExpr, Arg
                // First arg for these is the id
                Constructor::Num | Constructor::Boolean | Constructor::UnitExpr | Constructor::Arg=> 
                    format!(
"(rule
    ({ctor_pattern})
    ((union (RefIdOf{sort} {ctor_pattern}) {first_var}))
    :ruleset always-run)"),

                // For loops, call, all, let, get the id of the second item
                Constructor::Loop | Constructor::Call | Constructor::All | Constructor::Let
                    => format!(
"(rule
    ({ctor_pattern} (= aid (RefIdOf{} {})))
    ((union (RefIdOf{sort} {ctor_pattern}) aid))
    :ruleset always-run)", ctor.fields()[1].sort().name(), ctor.fields()[1].var()),

                // For everything else, get the id of the first item
                _ => 
                    format!(
"(rule ({ctor_pattern} (= aid (RefIdOf{var_sort} {first_var})))
    ((union (RefIdOf{sort} {ctor_pattern}) aid))
    :ruleset always-run)")
            }
        )
    }
}

pub(crate) fn id_analysis_rules() -> Vec<String> {
    let id_check = vec!["
(rule ((= (Id a) (Id b))
    (!= a b))
  ((panic \"RefIdOf: Ids don't match\"))
  :ruleset always-run)
    ".to_string()];
    
    ESort::iter()
        .map(|sort| "(function RefIdOf* (*) IdSort :unextractable)".replace('*', sort.name()))
        .chain(Constructor::iter().filter_map(id_analysis_rule_for_ctor))
        .chain(id_check)
        .collect::<Vec<_>>()
}

#[test]
fn test_id_analysis() {
    print!("{}", id_analysis_rules().join("\n"));
}
