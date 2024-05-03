use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use dot_structures::{Attribute, Edge, EdgeTy, Graph, Id, Node, NodeId, Stmt, Subgraph, Vertex};

use crate::schema::{Constant, Expr, RcExpr, TreeProgram};
use graphviz_rust::{cmd::Format, exec, printer::PrinterContext};

// We need one node for every unique expression, scope pair
// Expressions may be shared across scopes but should be shown multiple times
#[derive(Eq, PartialEq, Hash)]
struct UniqueExpr {
    expr: *const Expr,
    scope: *const Expr,
}

impl UniqueExpr {
    fn new(scope: *const Expr, expr: &RcExpr) -> Self {
        UniqueExpr {
            expr: Rc::as_ptr(expr),
            scope,
        }
    }
}

struct DotConverter {
    pub current_scope: *const Expr,
    pub done: HashSet<UniqueExpr>,
    pub get_name: HashMap<UniqueExpr, String>,
    pub name_counter: usize,
}

impl DotConverter {
    pub fn graphviz_id(&mut self, expr: &RcExpr) -> Id {
        if let Some(name) = self
            .get_name
            .get(&UniqueExpr::new(self.current_scope, expr))
        {
            Id::Plain(name.clone())
        } else {
            let name = match expr.as_ref() {
                Expr::DoWhile(_, _) => {
                    format!("cluster_{}{}", expr.constructor().name(), self.name_counter)
                }
                Expr::If(_, _, _, _) => {
                    format!("cluster_{}{}", expr.constructor().name(), self.name_counter)
                }
                Expr::Function(name, ..) => {
                    format!("cluster_fn{}{}", name, self.name_counter)
                }
                Expr::Const(c, _, _ctx) => match c {
                    Constant::Int(i) => format!("Const{}_{}", i, self.name_counter),
                    Constant::Bool(b) => format!("Const{}_{}", b, self.name_counter),
                },
                Expr::Bop(op, ..) => {
                    format!("{}{}", op.name(), self.name_counter)
                }
                Expr::Uop(op, ..) => {
                    format!("{}{}", op.name(), self.name_counter)
                }
                Expr::Top(op, ..) => {
                    format!("{}{}", op.name(), self.name_counter)
                }
                _ => {
                    format!("{}{}", expr.constructor().name(), self.name_counter)
                }
            };
            self.name_counter += 1;
            self.get_name
                .insert(UniqueExpr::new(self.current_scope, expr), name.clone());
            Id::Plain(name)
        }
    }

    pub fn graphviz_nodeid(&mut self, expr: &RcExpr) -> NodeId {
        NodeId(self.graphviz_id(expr), None)
    }

    pub fn graphviz_vertex(&mut self, expr: &RcExpr) -> Vertex {
        Vertex::N(self.graphviz_nodeid(expr))
    }
}

pub fn tree_to_svg(prog: &TreeProgram) -> String {
    let dot_code = prog.to_dot();
    String::from_utf8(
        exec(
            dot_code,
            &mut PrinterContext::default(),
            vec![Format::Svg.into()],
        )
        .unwrap(),
    )
    .unwrap()
}

impl TreeProgram {
    pub fn to_dot(&self) -> Graph {
        let mut dot_converter = DotConverter {
            done: HashSet::new(),
            get_name: HashMap::new(),
            name_counter: 0,
            current_scope: std::ptr::null(),
        };
        let mut stmts = self.to_dot_with(&mut dot_converter);

        // constrain ordering of  edges to be as they appear
        stmts.push(Stmt::Attribute(Attribute(
            Id::Plain("ordering".to_string()),
            Id::Plain("in".to_string()),
        )));
        Graph::DiGraph {
            id: Id::Plain("myprog".to_string()),
            strict: true,
            stmts,
        }
    }

    fn to_dot_with(&self, dot_converter: &mut DotConverter) -> Vec<Stmt> {
        let mut res = vec![];
        res.extend(self.entry.to_dot_with(dot_converter));
        for expr in &self.functions {
            res.extend(expr.to_dot_with(dot_converter));
        }
        res
    }
}

impl Expr {
    pub fn to_dot(self: &RcExpr) -> Graph {
        let mut dot_converter = DotConverter {
            done: HashSet::new(),
            get_name: HashMap::new(),
            name_counter: 0,
            current_scope: Rc::as_ptr(self),
        };

        Graph::DiGraph {
            id: Id::Plain("myprog".to_string()),
            strict: true,
            stmts: self.to_dot_with(&mut dot_converter),
        }
    }

    fn to_dot_with(self: &RcExpr, conv: &mut DotConverter) -> Vec<Stmt> {
        let id = Rc::as_ptr(self);
        if !conv.done.insert(UniqueExpr::new(conv.current_scope, self)) {
            return vec![];
        }
        match self.as_ref() {
            Expr::DoWhile(inputs, body) => {
                let mut stmts = inputs.to_dot_with(conv);
                let vertex_outside = conv.graphviz_vertex(self);
                let id_outside = conv.graphviz_id(self);
                stmts.push(Stmt::Edge(Edge {
                    ty: EdgeTy::Pair(vertex_outside.clone(), conv.graphviz_vertex(inputs)),
                    attributes: vec![],
                }));

                let scope_before = conv.current_scope;
                conv.current_scope = id;
                stmts.push(Stmt::Subgraph(Subgraph {
                    stmts: body.to_dot_with(conv),
                    id: id_outside,
                }));
                stmts.push(Stmt::Edge(Edge {
                    ty: EdgeTy::Pair(vertex_outside, conv.graphviz_vertex(body)),
                    attributes: vec![],
                }));
                // restore scope of the dowhile
                conv.current_scope = scope_before;
                stmts
            }
            Expr::If(pred, inputs, then_case, else_case) => {
                let mut stmts = inputs.to_dot_with(conv);
                stmts.extend(pred.to_dot_with(conv));
                let vertex_outside = conv.graphviz_vertex(self);
                let id_outside = conv.graphviz_id(self);
                stmts.push(Stmt::Edge(Edge {
                    ty: EdgeTy::Pair(vertex_outside.clone(), conv.graphviz_vertex(inputs)),
                    attributes: vec![],
                }));

                let scope_before = conv.current_scope;
                conv.current_scope = id;
                stmts.push(Stmt::Subgraph(Subgraph {
                    stmts: then_case.to_dot_with(conv),
                    id: id_outside.clone(),
                }));
                stmts.push(Stmt::Edge(Edge {
                    ty: EdgeTy::Pair(vertex_outside.clone(), conv.graphviz_vertex(then_case)),
                    attributes: vec![],
                }));

                conv.current_scope = id;
                stmts.push(Stmt::Subgraph(Subgraph {
                    stmts: else_case.to_dot_with(conv),
                    id: id_outside,
                }));
                stmts.push(Stmt::Edge(Edge {
                    ty: EdgeTy::Pair(vertex_outside, conv.graphviz_vertex(else_case)),
                    attributes: vec![],
                }));

                // restore scope of the dowhile
                conv.current_scope = scope_before;
                stmts
            }
            Expr::Function(_name, _in_ty, _out_ty, body) => {
                conv.current_scope = id;

                let mut stmts = vec![Stmt::Node(Node {
                    id: conv.graphviz_nodeid(self),
                    attributes: vec![],
                })];
                stmts.extend(body.to_dot_with(conv));
                stmts.push(Stmt::Edge(Edge {
                    ty: EdgeTy::Pair(conv.graphviz_vertex(self), conv.graphviz_vertex(body)),
                    attributes: vec![],
                }));
                vec![Stmt::Subgraph(Subgraph {
                    stmts,
                    id: conv.graphviz_id(self),
                })]
            }
            _ => {
                let children = self.children_same_scope();
                let mut stmts = vec![Stmt::Node(Node {
                    id: conv.graphviz_nodeid(self),
                    attributes: vec![],
                })];
                for child in children {
                    let child_stmts = child.to_dot_with(conv);
                    stmts.extend(child_stmts);
                    stmts.push(Stmt::Edge(Edge {
                        ty: EdgeTy::Pair(conv.graphviz_vertex(self), conv.graphviz_vertex(&child)),
                        attributes: vec![],
                    }));
                }

                stmts
            }
        }
    }
}
