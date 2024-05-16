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

use std::fs;

use bril_rs::Type;
use dag_in_context::print_with_intermediate_vars;
pub(crate) use to_block;

macro_rules! rvsdg_svg_test {
    ($name:ident, $filename:expr) => {
        #[test]
        fn $name() {
            use crate::Optimizer;

            let add = include_str!($filename);
            let rvsdg = Optimizer::program_to_rvsdg(&Optimizer::parse_bril(add).unwrap()).unwrap();
            let svg = rvsdg.to_svg();
            insta::assert_snapshot!(svg);
        }
    };
}

pub(crate) fn true_cond(name: &str) -> BranchOp {
    BranchOp::Cond {
        arg: name.into(),
        val: true.into(),
        bril_type: Type::Bool,
    }
}

pub(crate) fn false_cond(name: &str) -> BranchOp {
    BranchOp::Cond {
        arg: name.into(),
        val: false.into(),
        bril_type: Type::Bool,
    }
}

pub(crate) use rvsdg_svg_test;

macro_rules! cfg_test_equiv {
  // for the case of a single-node cfg
  ($cfg:expr, []) => {
      assert_eq!($cfg.graph.node_count(), 1);
      assert_eq!($cfg.graph.edge_count(), 0);
      assert_eq!($cfg.entry, $cfg.exit);
  };
  ($cfg:expr,  [ $($src:tt = ($($edge:tt)*)=> $dst:tt,)* ]) => {
      use $crate::cfg::BranchOp::{*};
      let mut mentioned = std::collections::HashSet::new();
          let mut block = std::collections::HashMap::new();
          $(
              mentioned.insert(to_block!($src));
              mentioned.insert(to_block!($dst));
          )*
          let cfg = $cfg;
          for i in cfg.graph.node_indices() {
              let node = cfg.graph.node_weight(i).unwrap();
              assert!(mentioned.contains(&node.name), "description does not mention block {:?}", node.name);
              block.insert(node.name.clone(), i);
          }
          $({
              let src_name = to_block!($src);
              let dst_name = to_block!($dst);
              let src = *block.get(&src_name).unwrap_or_else(|| panic!("missing block {:?}", src_name));
              let dst = *block.get(&dst_name).unwrap_or_else(|| panic!("missing block {:?}", dst_name));
              let has_edge = cfg.graph.edges_connecting(src, dst).any(|edge| {
                  edge.weight().op == $($edge)*
              });
              assert!(has_edge, "missing edge from {src_name:?} to {dst_name:?}");
          })*
  };
}

#[test]
fn test_pretty_print_to_egglog() {
    let schema = fs::read_to_string("./dag_in_context/src/schema.egg").unwrap();
    let paths = std::fs::read_dir("./tests/passing/small").unwrap();
    for path in paths {
        let path = path.unwrap();
        let program = crate::util::TestProgram::BrilFile(path.path()).read_program();
        let rvsdg = crate::Optimizer::program_to_rvsdg(&program.program).unwrap();
        let dag = rvsdg.to_dag_encoding(true);
        let (term, termdag) = dag.entry.to_egglog();
        let unfolded_program = print_with_intermediate_vars(&termdag, term);
        let folded_program =
            dag_in_context::pretty_print::PrettyPrinter::from_expr(dag.entry).to_egglog_default();
        let program = format!(
            "{schema}\n {unfolded_program} \n {folded_program} \n (check (= PROG EXPR___))"
        );
        egglog::EGraph::default()
            .parse_and_run_program(&program)
            .unwrap();
    }
}

pub(crate) use cfg_test_equiv;

use crate::cfg::BranchOp;
