#[cfg(test)]
use crate::egglog_test;

#[test]
fn loop_splitting() -> crate::Result {
    use crate::ast::*;

    /*
    i = 0
    do while i < 9:
        if i <= 4:
            print 1
        else:
            print 2
        i += 1
    print i
    */
    let build_loop = dowhile(
        parallel![arg(), int(0)],
        parallel![
            less_than(getat(1), int(9)),
            tif(
                less_eq(getat(1), int(4)),
                single(getat(0)),
                tprint(int(1), getat(0)),
                tprint(int(2), getat(0)),
            ),
            add(getat(1), int(1)),
        ],
    );
    let build = tprint(get(build_loop.clone(), 1), get(build_loop, 0));

    /*
    i = 0
    do while i < 4:
        print 1
        i += 1
    if 4 < 9:
        do while i < 9:
            print 2
            i += 1
    print i
    */

    let check_loop_1 = dowhile(
        parallel![arg(), int(0)],
        parallel![
            less_than(getat(1), int(4)),
            tprint(int(1), getat(0)),
            add(getat(1), int(1)),
        ],
    );
    let check_loop_2 = dowhile(
        check_loop_1.clone(),
        parallel![
            less_than(getat(1), int(9)),
            tprint(int(2), getat(0)),
            add(getat(1), int(1)),
        ],
    );
    let check_loop_2_if = tif(less_than(int(4), int(9)), arg(), check_loop_2, check_loop_1);
    let check = tprint(get(check_loop_2_if.clone(), 1), get(check_loop_2_if, 0));

    let build = build.to_program(base(statet()), base(statet()));
    let check = check.to_program(base(statet()), base(statet()));
    egglog_test(
        &format!("(let b {build})"),
        &format!("(let c {check}) (check (= b c))"),
        vec![build, check],
        statev(),
        statev(),
        vec![
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "10".to_string(),
        ],
    )
}
