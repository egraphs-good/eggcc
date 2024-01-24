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

impl RvsdgProgram {
    pub fn to_tree_encoding(&self) -> Expr {
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
    use tree_unique_args::ast::*;
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

    rvsdg
        .to_tree_encoding()
        .assert_eq_ignoring_ids(&program(vec![function(tlet(
            concat(arg(), num(1)),
            unit(),
        ))]));
}
