use crate::cfg::{function_to_cfg, BlockName};
use crate::test_util::*;
use bril2json::parse_abstract_program_from_read;
use bril_rs::{load_program_from_read, Program, Type};

fn parse_from_string(input: &str) -> Program {
    let abs_program = parse_abstract_program_from_read(input.as_bytes(), true, false, None);
    let mut buf = Vec::new();
    serde_json::to_writer_pretty(&mut buf, &abs_program).unwrap();
    buf.push(b'\n');
    let json_str = String::from_utf8(buf).unwrap();
    load_program_from_read(json_str.as_bytes())
}

// Test that a CFG is wired up correctly.
macro_rules! cfg_test_function_to_cfg {
    ($name:ident, $prog:expr, [ $($src:tt =($($edge:tt)*)=> $dst:tt,)* ]) => {
        #[test]
        fn $name() {
            let prog = parse_from_string($prog);
            let cfg = function_to_cfg(&prog.functions[0]);
            cfg_test_equiv!(cfg, [ $($src =($($edge)*)=> $dst,)* ]);
        }
    };
}

cfg_test_function_to_cfg!(
    fib_cfg,
    include_str!("../../tests/brils/failing/mem/fib.bril"),
    [
        ENTRY  = (Jmp) => "loop",
        "loop" = (true_cond("cond")) => "body",
        "loop" = (false_cond("cond")) => "done",
        "body" = (Jmp) => "loop",
        "done" = (Jmp) => EXIT,
    ]
);

cfg_test_function_to_cfg!(
    queen,
    include_str!("../../tests/small/failing/queens-func.bril"),
    [
        ENTRY = (Cond { arg: "ret_cond".into(), val: true.into(), bril_type: Type::Bool  }) => "main.next.ret",
        ENTRY = (Cond { arg: "ret_cond".into(), val: false.into(), bril_type: Type::Bool  }) => "main.for.cond",
        "main.for.cond" = (Cond { arg: "for_cond_0".into(), val: true.into(), bril_type: Type::Bool  }) => "main.for.body",
        "main.for.cond" = (Cond { arg: "for_cond_0".into(), val: false.into(), bril_type: Type::Bool  }) => "main.next.ret.1",
        "main.for.body" = (Cond { arg: "is_valid".into(), val: true.into(), bril_type: Type::Bool  }) => "main.rec.func",
        "main.for.body" = (Cond { arg: "is_valid".into(), val: false.into(), bril_type: Type::Bool  }) => "main.next.loop",
        "main.rec.func" = (Jmp) => "main.next.loop",
        "main.next.loop" = (Jmp) => "main.for.cond",
        "main.next.ret" = (Jmp) => "main.print",
        "main.next.ret.1" = (Jmp) => "main.print",
        "main.print" = (Jmp) => EXIT,
    ]
);

cfg_test_function_to_cfg!(
    implicit_return,
    include_str!("../../tests/small/implicit-return.bril"),
    [
        ENTRY = (Jmp) => EXIT,
    ]
);

cfg_test_function_to_cfg!(
    diamond,
    include_str!("../../tests/small/diamond.bril"),
    [
        ENTRY = (Cond { arg: "cond".into(), val: true.into(), bril_type: Type::Bool  }) => "B",
        ENTRY = (Cond { arg: "cond".into(), val: false.into(), bril_type: Type::Bool  }) => "C",
        "B" = (Jmp) => "D",
        "C" = (Jmp) => "D",
        "D" = (Jmp) => EXIT,
    ]
);

cfg_test_function_to_cfg!(
    block_diamond,
    include_str!("../../tests/small/block-diamond.bril"),
    [
        ENTRY = (true_cond("a_cond")) => "B",
        ENTRY = (false_cond("a_cond")) => "D",
        "B"   = (true_cond("b_cond")) => "C",
        "B"   = (false_cond("b_cond")) => "E",
        "C" = (Jmp) => "F",
        "D" = (Jmp) => "E",
        "E" = (Jmp) => "F",
        "F" = (Jmp) => EXIT,
    ]
);

cfg_test_function_to_cfg!(
    unstructured,
    include_str!("../../tests/small/should_fail/unstructured.bril"),
    [
        ENTRY = (true_cond("a_cond")) => "B",
        ENTRY = (false_cond("a_cond")) => "C",
        "B"   = (true_cond("b_cond")) => "C",
        "B"   = (false_cond("b_cond")) => "D",
        "C" = (Jmp) => "B",
        "D" = (Jmp) => EXIT,
    ]
);

cfg_test_function_to_cfg!(
    fib_shape_cfg,
    include_str!("../../tests/small/fib_shape.bril"),
    [
        ENTRY = (Jmp) => "loop",
        "loop" = (true_cond("cond")) => "body",
        "loop" = (false_cond("cond")) => "done",
        "body" = (Jmp) => "loop",
        "done" = (Jmp) => EXIT,
    ]
);
