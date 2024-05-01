// #[test]
// fn loop_strength_reduction_passthrough_const() -> crate::Result {
//     use crate::ast::*;
//     // n = function input, arg0
//     // i = 0, x = 5, accum = 0
//     // do {
//     //    i++
//     //    accum += x * i
//     // } while (i < n)
//     // args to dowhile are :
//     //  n   i   x   accum
//     let strong_loop = dowhile(
//         parallel!(arg(), int(0), int(5), int(0)),
//         parallel!(
//             (less_than(get(arg(), 1), get(arg(), 0))), // pred
//             (get(arg(), 0)),                           // n
//             (add(get(arg(), 1), int(1))),              // i
//             (get(arg(), 2)),                           // x
//             (add(mul(get(arg(), 2), get(arg(), 1)), get(arg(), 3)))  // accum
//         ),
//     );

//     let prog = function(
//         "main",
//         base(intt()),
//         base(intt()),
//         get(strong_loop.clone(), 3),
//     )
//     .to_program(base(intt()), base(intt()))
//     .add_context()
//     .with_arg_types();

//     // n = arg0, i = 0, x = 5, accum = 0, d = 0
//     // do {
//     //     i++
//     //     d += x
//     //     accum += d
//     // }
//     let expected_loop = dowhile(
//         parallel!(arg(), int(0), int(5), int(0), int(0)),
//         parallel!(
//             (less_than(get(arg(), 1), get(arg(), 0))), // pred
//             (get(arg(), 0)),                           // n
//             (add(get(arg(), 1), int(1))),              // i
//             (get(arg(), 2)),                           // x
//             (add(get(arg(), 4), get(arg(), 3))),       // accum
//             (add(get(arg(), 4), get(arg(), 2)))        // d
//         ),
//     );
//     // TODO: can add symbolic context
//     let expected_prog = function(
//         "main",
//         base(intt()),
//         base(intt()),
//         get(expected_loop.clone(), 3),
//     )
//     .to_program(base(intt()), base(intt()))
//     .add_context()
//     .with_arg_types();

//     crate::egglog_test_and_print_program(
//         &format!("{prog}"),
//         "",
//         // &format!(
//         //     "(check (= {} {}))",
//         //     prog.entry.func_body().expect("func has body"),
//         //     expected_prog.entry.func_body().expect("func has body")
//         // ),
//         vec![prog],
//         intv(5),
//         intv(75),
//         vec![],
//     )
// }
