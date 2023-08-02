use crate::{
    cfg::{to_cfg, to_structured::to_structured, BlockName},
    EggCCError,
};
use bril2json::parse_abstract_program_from_read;
use bril_rs::{load_program_from_read, Program};
use petgraph::graph::NodeIndex;

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
            let cfg = to_cfg(&prog.functions[0]);
            let mut mentioned = std::collections::HashSet::new();
            let mut block = std::collections::HashMap::new();
            $(
                mentioned.insert(to_block!($src));
                mentioned.insert(to_block!($dst));
            )*
            for (i, node) in cfg.graph.raw_nodes().iter().enumerate() {
                assert!(mentioned.contains(&node.weight.name), "description does not mention block {:?}", node.weight.name);
                block.insert(node.weight.name.clone(), NodeIndex::new(i));
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
    include_str!("../../tests/fib.bril"),
    [
        ENTRY  = (Jmp) => "loop",
        "loop" = (Cond { arg: "cond".into(), val: true }) => "body",
        "loop" = (Cond { arg: "cond".into(), val: false }) => "done",
        "body" = (Jmp) => "loop",
        "done" = (Jmp) => EXIT,
    ]
);

cfg_test!(
    queen,
    include_str!("../../tests/small/queens-func.bril"),
    [
        ENTRY = (Cond { arg: "ret_cond".into(), val: true }) => "next.ret",
        ENTRY = (Cond { arg: "ret_cond".into(), val: false }) => "for.cond",
        "for.cond" = (Cond { arg: "for_cond_0".into(), val: true }) => "for.body",
        "for.cond" = (Cond { arg: "for_cond_0".into(), val: false }) => "next.ret.1",
        "for.body" = (Cond { arg: "is_valid".into(), val: true }) => "rec.func",
        "for.body" = (Cond { arg: "is_valid".into(), val: false }) => "next.loop",
        "rec.func" = (Jmp) => "next.loop",
        "next.loop" = (Jmp) => "for.cond",
        "next.ret" = (RetVal { arg: "icount".into() }) => EXIT,
        "next.ret.1" = (RetVal { arg: "icount".into() }) => EXIT,
    ]
);

cfg_test!(
    implicit_return,
    include_str!("../../tests/small/implicit-return.bril"),
    [
        ENTRY = (Jmp) => EXIT,
    ]
);

cfg_test!(
    diamond,
    include_str!("../../tests/small/diamond.bril"),
    [
        ENTRY = (Cond { arg: "cond".into(), val: true }) => "B",
        ENTRY = (Cond { arg: "cond".into(), val: false }) => "C",
        "B" = (Jmp) => "D",
        "C" = (Jmp) => "D",
        "D" = (Jmp) => EXIT,
    ]
);

cfg_test!(
    block_diamond,
    include_str!("../../tests/small/block-diamond.bril"),
    [
        ENTRY = (Cond { arg: "a_cond".into(), val: true }) => "B",
        ENTRY = (Cond { arg: "a_cond".into(), val: false }) => "D",
        "B"   = (Cond { arg: "b_cond".into(), val: true}) => "C",
        "B"   = (Cond { arg: "b_cond".into(), val: false}) => "E",
        "C" = (Jmp) => "F",
        "D" = (Jmp) => "E",
        "E" = (Jmp) => "F",
        "F" = (Jmp) => EXIT,
    ]
);

cfg_test!(
    unstructured,
    include_str!("../../tests/small/unstructured.bril"),
    [
        ENTRY = (Cond { arg: "a_cond".into(), val: true }) => "B",
        ENTRY = (Cond { arg: "a_cond".into(), val: false }) => "C",
        "B"   = (Cond { arg: "b_cond".into(), val: true }) => "C",
        "B"   = (Cond { arg: "b_cond".into(), val: false }) => "D",
        "C" = (Jmp) => "B",
        "D" = (Jmp) => EXIT,
    ]
);

#[test]
fn unstructured_panics() {
    let func = &parse_from_string(include_str!("../../tests/small/unstructured.bril")).functions[0];
    assert!(matches!(
        to_structured(&to_cfg(func)),
        Err(EggCCError::UnstructuredControlFlow)
    ))
}
