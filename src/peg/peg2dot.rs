//! Render a PEG into Dot format;

use crate::peg::{PegBody, PegFunction, PegProgram};
use crate::rvsdg::Expr;
use bril_rs::ConstOps;
use std::fmt::Write;

impl PegProgram {
    /// Get a .dot file representation of a PegProgram.
    pub fn graph(&self) -> String {
        let mut graph = String::new();
        writeln!(graph, "digraph G {{").unwrap();
        for function in &self.functions {
            // Replace the "digraph" line with "subgraph".
            let g = function.graph();
            let mut subgraph: Vec<_> = g.lines().collect();
            subgraph[0] = "subgraph {{";
            let subgraph = subgraph.join("\n");
            writeln!(graph, "{}", subgraph).unwrap();
        }
        writeln!(graph, "}}").unwrap();
        graph
    }
}

impl PegFunction {
    /// Get a .dot file representation of a PegFunction.
    // Doesn't use petgraph because petgraph doesn't track child orderings.
    pub fn graph(&self) -> String {
        let mut nodes: Vec<String> = Vec::new();
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for (i, node) in self.nodes.iter().enumerate() {
            let mut js = Vec::new();
            let node = match node {
                PegBody::Arg(arg) => format!("arg {arg}"),
                PegBody::BasicOp(expr) => match expr {
                    Expr::Op(f, xs, _ty) => {
                        js = xs.to_vec();
                        format!("{f}")
                    }
                    Expr::Call(f, xs, _, _) => {
                        js = xs.to_vec();
                        format!("{f}")
                    }
                    Expr::Print(xs) => {
                        js = xs.to_vec();
                        "PRINT".into()
                    }
                    Expr::Const(ConstOps::Const, _, literal) => {
                        format!("{literal}")
                    }
                },
                PegBody::Phi(c, x, y) => {
                    js = vec![*c, *x, *y];
                    String::from("Φ")
                }
                PegBody::Theta(a, b, l) => {
                    js = vec![*a, *b];
                    format!("Θ_{l}")
                }
                PegBody::Eval(s, i, l) => {
                    js = vec![*s, *i];
                    format!("eval_{l}")
                }
                PegBody::Pass(s, l) => {
                    js = vec![*s];
                    format!("pass_{l}")
                }
                PegBody::Edge(x) => {
                    js = vec![*x];
                    String::from("no-op")
                }
            };
            nodes.push(node);
            edges.extend(js.into_iter().map(|j| (i, j)));
        }
        let mut graph = String::new();
        writeln!(graph, "digraph G {{").unwrap();
        writeln!(graph, "node [ordering=out];").unwrap();
        for (i, node) in nodes.into_iter().enumerate() {
            writeln!(graph, "{i} [label={node:?}];").unwrap();
        }
        for (start, end) in edges {
            writeln!(graph, "{start} -> {end};",).unwrap();
        }
        writeln!(graph, "}}").unwrap();
        graph
    }
}
