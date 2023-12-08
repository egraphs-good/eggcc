pub use crate::*;

#[test]
fn loop_strength_reduction() -> Result {
    let build = "
        (let test-outer-id (id (i64-fresh!)))
        (let test-loop-id (id (i64-fresh!)))
        (let loop (
            Loop
            test-loop-id
            ; pred: i < 4
            (blt (Arg test-loop-id 0) (Num test-loop-id 4))
            ; inputs: i = 0, a = 0, b = 0, c = 3
            (EVec (vec-of
                (Num test-outer-id 0) 
                (Num test-outer-id 0)
                (Num test-outer-id 0)
                (Num test-outer-id 3)
            ))
            ; outputs: i = i + 1, a = i * c, b += a, c = c
            (EVec (vec-of
                (badd (Arg test-loop-id 0) (Num test-loop-id 1))
                (bmul (Arg test-loop-id 3) (Arg test-loop-id 0))
                (badd (Arg test-loop-id 1) (Arg test-loop-id 2))
                (Arg test-loop-id 3)
            ))
        ))
    ";
    let check = "
        (check (= 
            old-loop
            (
            Loop
            some-new-id
            ; pred: i < 4
            (blt (Arg some-new-id 0) (Num some-new-id 4))
            ; inputs: i = 0, a = 0, b = 0, c = 3, d = i * c
            (EVec (vec-of
                (Num test-outer-id 0) 
                (Num test-outer-id 0)
                (Num test-outer-id 0)
                (Num test-outer-id 3)
                (bmul (Num test-outer-id 3) (Num test-outer-id 0))
            ))
            ; outputs: i = i + 1, a = d, b += a, c = c, d += c * 1
            (EVec (vec-of
                (badd (Arg some-new-id 0) (Num some-new-id 1))
                (Arg some-new-id 4)
                (badd (Arg some-new-id 1) (Arg some-new-id 2))
                (Arg some-new-id 3)
                (badd (Arg some-new-id 4) (bmul (Arg some-new-id 3) (Num some-new-id 1)))
            ))
            )
        ))
    ";
    run_test(build, check)
}