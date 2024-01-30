//! Rules that perform sanity checks,
//! such as checking that switch children
//! are all `Branch`es.

use crate::{
    expr::{ESort, Expr},
    ir::Constructor,
};

pub(crate) fn error_checking_rules() -> Vec<String> {
    let mut res = vec![format!(
        "
(relation IsBranchList (ListExpr))

(rule ((Switch pred outputs))
      ((IsBranchList outputs))
      :ruleset error-checking)

(rule ((IsBranchList (Cons a rest)))
      ((IsBranchList rest))
      :ruleset error-checking)
  "
    )];

    for ctor in Constructor::iter() {
        if ctor.sort() == ESort::ListExpr {
            continue;
        }
        if let Constructor::Expr(Expr::Branch(..)) = ctor {
            continue;
        }

        let pat = ctor.construct(|field| field.var());
        let ctor_name = ctor.name();
        res.push(format!(
            "
(rule ((IsBranchList (Cons {pat} rest)))
      ((panic \"Expected Branch, got {ctor_name}\"))
      :ruleset error-checking)
"
        ));
    }

    res
}

#[test]
#[should_panic(expected = "Expected Branch, got Num")]
fn test_switch_with_num_child() {
    let build = "
    (Switch
        (Num (Shared) 0)
        (Cons
            (Num (Shared) 1)
            (Cons
                (Branch (Shared) (Num (Shared) 2))
                (Nil))))
    ";
    let _ = crate::run_test(build, "");
}
