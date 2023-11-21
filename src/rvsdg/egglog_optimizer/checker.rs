use super::AST_SORTS;

pub(crate) fn checker_code() -> String {
    let mut res = vec!["
(sort Env (Vec Literal))

;; Sanity checks: make sure no constants are equal in the database
(rule ((= (Num a) (Num b)) (!= a b)) ((panic \"unioned two numbers with different values\")))
(rule ((= (Num a) (Float b))) ((panic \"num and float cannot be equal\")))
(rule ((= (Num a) (Char b))) ((panic \"num and char cannot be equal\")))
(rule ((= (Num a) (Bool b))) ((panic \"num and bool cannot be equal\")))
(rule ((= (Float a) (Float b)) (!= a b)) ((panic \"unioned two floats with different values\")))
(rule ((= (Float a) (Char b))) ((panic \"float and char cannot be equal\")))
(rule ((= (Float a) (Bool b))) ((panic \"float and bool cannot be equal\")))
(rule ((= (Char a) (Char b)) (!= a b)) ((panic \"unioned two chars with different values\")))
(rule ((= (Char a) (Bool b))) ((panic \"char and bool cannot be equal\")))
(rule ((= (Bool a) (Bool b)) (!= a b)) ((panic \"unioned two bools with different values\")))
    "
    .to_string()];

    for sort in &AST_SORTS {
        res.push(format!("(function {sort}EvalsTo (Env {sort}) Env)"));
    }

    for sort in &AST_SORTS {
        res.push(format!("(relation {sort}EvalsToDemand (Env {sort}))"));
    }

    res.join("\n")
}
