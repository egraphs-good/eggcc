//! Convert RVSDG programs to the tree
//! encoding of programs.
//! RVSDGs are close to this encoding,
//! but use a DAG-based semantics.
//! This means that nodes that are shared
//! are only computed once.
//! These shared nodes need to be let-bound so that they are only
//! computed once in the tree encoded
//! program.

#[cfg(test)]
use crate::{cfg::program_to_cfg, rvsdg::cfg_to_rvsdg, util::parse_from_string};

use crate::rvsdg::{RvsdgFunction, RvsdgProgram};
use tree_unique_args::Expr;

struct FreshIdGen {
    counter: i64,
}

impl FreshIdGen {
    fn new() -> Self {
        Self { counter: 0 }
    }

    fn fresh(&mut self) -> i64 {
        let id = self.counter;
        self.counter += 1;
        id
    }
}

impl RvsdgProgram {
    pub fn to_tree_encoding(&self) -> Expr {
        self.to_tree_encoding_helper(&mut FreshIdGen::new())
    }

    fn to_tree_encoding_helper(&self, idgen: &mut FreshIdGen) -> Expr {
        unimplemented!()
    }
}

impl RvsdgFunction {
    pub fn to_tree_encoding(&self) -> Expr {
        unimplemented!()
    }
}

#[test]
fn simple_translation() {
    use Expr::*;
    const PROGRAM: &str = r#"
    @sub() {
        v0: int = const 1;
        v1: int = const 2;
        v2: int = add v0 v1;
        print v2;
        print v1;
    }
    "#;

    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    assert_eq!(rvsdg.to_tree_encoding(), Program(vec![]));
}
