pub use crate::*;

#[test]
fn subst_args() -> Result {
    let build = "
        (let args-id (id (i64-fresh!)))
        (let args (EVec (vec-of (Num args-id 0) (Num args-id 1) (Bool args-id false))))

        (let body-id (id (i64-fresh!)))
        (let body (EVec (vec-of 
            (badd (Arg body-id 1) (Num body-id 3))
            (band (Arg body-id 2) (Bool body-id true))
            (Arg body-id 0)
        )))

        (SubstArgsAll body args)
    ";
    let check = "
        (check (= (SubstArgsAll (Arg body-id 0) args) (Num args-id 0)))
        (check (= (SubstArgsAll (Arg body-id 1) args) (Num args-id 1)))
        (check (= (SubstArgsAll (Arg body-id 2) args) (Bool args-id false)))
    ";
    run_test(build, check)
}
