use crate::{
    cfg::{function_to_cfg, program_to_cfg, to_structured::cfg_to_structured, BlockName},
    EggCCError,
};
use bril2json::parse_abstract_program_from_read;
use bril_rs::{load_program_from_read, Program};

fn parse_from_string(input: &str) -> Program {
    let abs_program = parse_abstract_program_from_read(input.as_bytes(), true, false, None);
    let mut buf = Vec::new();
    serde_json::to_writer_pretty(&mut buf, &abs_program).unwrap();
    buf.push(b'\n');
    let json_str = String::from_utf8(buf).unwrap();
    load_program_from_read(json_str.as_bytes())
}

macro_rules! to_block {
    (ENTRY) => {
        BlockName::Entry
    };
    (EXIT) => {
        BlockName::Exit
    };
    ($name:expr) => {
        BlockName::Named($name.into())
    };
}

// Test that a CFG is wired up correctly.
macro_rules! cfg_test {
    ($name:ident, $prog:expr, [ $($src:tt =($($edge:tt)*)=> $dst:tt,)* ]) => {
        #[test]
        fn $name() {
            use $crate::cfg::BranchOp;
            let prog = parse_from_string($prog);
            let cfg = function_to_cfg(&prog.functions[0]);
            let mut mentioned = std::collections::HashSet::new();
            let mut block = std::collections::HashMap::new();
            $(
                mentioned.insert(to_block!($src));
                mentioned.insert(to_block!($dst));
            )*
            for i in cfg.graph.node_indices() {
                let node = cfg.graph.node_weight(i).unwrap();
                assert!(mentioned.contains(&node.name), "description does not mention block {:?}", node.name);
                block.insert(node.name.clone(), i);
            }
            $({
                let src_name = to_block!($src);
                let dst_name = to_block!($dst);
                let src = block[&src_name];
                let dst = block[&dst_name];
                let has_edge = cfg.graph.edges_connecting(src, dst).any(|edge| {
                    edge.weight().op == BranchOp::$($edge)*
                });
                assert!(has_edge, "missing edge from {src_name:?} to {dst_name:?}");
            })*
        }
    };
}

cfg_test!(
    fib_cfg,
    include_str!("../../tests/brils/failing/mem/fib.bril"),
    [
        ENTRY  = (Jmp) => "loop",
        "loop" = (Cond { arg: "cond".into(), val: true.into() }) => "body",
        "loop" = (Cond { arg: "cond".into(), val: false.into() }) => "done",
        "body" = (Jmp) => "loop",
        "done" = (Jmp) => EXIT,
    ]
);

cfg_test!(
    queen,
    include_str!("../../tests/small/failing/queens-func.bril"),
    [
        ENTRY = (Cond { arg: "ret_cond".into(), val: true.into() }) => "next.ret",
        ENTRY = (Cond { arg: "ret_cond".into(), val: false.into() }) => "for.cond",
        "for.cond" = (Cond { arg: "for_cond_0".into(), val: true.into() }) => "for.body",
        "for.cond" = (Cond { arg: "for_cond_0".into(), val: false.into() }) => "next.ret.1",
        "for.body" = (Cond { arg: "is_valid".into(), val: true.into() }) => "rec.func",
        "for.body" = (Cond { arg: "is_valid".into(), val: false.into() }) => "next.loop",
        "rec.func" = (Jmp) => "next.loop",
        "next.loop" = (Jmp) => "for.cond",
        "next.ret" = (Jmp) => EXIT,
        "next.ret.1" = (Jmp) => EXIT,
    ]
);

cfg_test!(
    implicit_return,
    include_str!("../../tests/small/failing/implicit-return.bril"),
    [
        ENTRY = (Jmp) => EXIT,
    ]
);

cfg_test!(
    diamond,
    include_str!("../../tests/small/diamond.bril"),
    [
        ENTRY = (Cond { arg: "cond".into(), val: true.into() }) => "B",
        ENTRY = (Cond { arg: "cond".into(), val: false.into() }) => "C",
        "B" = (Jmp) => "D",
        "C" = (Jmp) => "D",
        "D" = (Jmp) => EXIT,
    ]
);

cfg_test!(
    block_diamond,
    include_str!("../../tests/small/block-diamond.bril"),
    [
        ENTRY = (Cond { arg: "a_cond".into(), val: true.into() }) => "B",
        ENTRY = (Cond { arg: "a_cond".into(), val: false.into() }) => "D",
        "B"   = (Cond { arg: "b_cond".into(), val: true.into() }) => "C",
        "B"   = (Cond { arg: "b_cond".into(), val: false.into() }) => "E",
        "C" = (Jmp) => "F",
        "D" = (Jmp) => "E",
        "E" = (Jmp) => "F",
        "F" = (Jmp) => EXIT,
    ]
);

cfg_test!(
    unstructured,
    include_str!("../../tests/small/should_fail/unstructured.bril"),
    [
        ENTRY = (Cond { arg: "a_cond".into(), val: true.into() }) => "B",
        ENTRY = (Cond { arg: "a_cond".into(), val: false.into() }) => "C",
        "B"   = (Cond { arg: "b_cond".into(), val: true.into() }) => "C",
        "B"   = (Cond { arg: "b_cond".into(), val: false.into() }) => "D",
        "C" = (Jmp) => "B",
        "D" = (Jmp) => EXIT,
    ]
);

cfg_test!(
    fib_shape_cfg,
    include_str!("../../tests/small/fib_shape.bril"),
    [
        ENTRY = (Jmp) => "loop",
        "loop" = (Cond { arg: "cond".into(), val: true.into() }) => "body",
        "loop" = (Cond { arg: "cond".into(), val: false.into() }) => "done",
        "body" = (Jmp) => "loop",
        "done" = (Jmp) => EXIT,
    ]
);

#[test]
fn unstructured_causes_error() {
    let func = &parse_from_string(include_str!(
        "../../tests/small/should_fail/unstructured.bril"
    ));
    assert!(matches!(
        cfg_to_structured(&program_to_cfg(func)),
        Err(EggCCError::UnstructuredControlFlow)
    ))
}
