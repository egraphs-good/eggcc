use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use dot_structures::{Edge, EdgeTy, Graph, Id, Node, NodeId, Stmt, Subgraph, Vertex};

use crate::schema::{Expr, RcExpr, TreeProgram};

struct DotConverter {
    pub done: HashSet<*const Expr>,
    pub get_name: HashMap<*const Expr, String>,
    pub name_counter: usize,
}

impl DotConverter {
    pub fn graphviz_id(&mut self, expr: &RcExpr) -> Id {
      if let Some(name) = self.get_name.get(&Rc::as_ptr(expr)) {
        return Id::Plain(name.clone());
      }
      else {
        let name = format!("{}{}", expr.constructor().name(), self.name_counter);
        self.name_counter += 1;
        self.get_name.insert(Rc::as_ptr(expr), name.clone());
        return Id::Plain(name);
      }
    }

    pub fn graphviz_nodeid(&mut self, expr: &RcExpr) -> NodeId {
      NodeId(self.graphviz_id(expr), None)
    }

    pub fn graphviz_vertex(&mut self, expr: &RcExpr) -> Vertex {
      Vertex::N(self.graphviz_nodeid(expr))
    }
}

impl TreeProgram {
    pub fn to_dot(&self) -> Graph {
        let mut dot_converter = DotConverter {
            done: HashSet::new(),
            get_name: HashMap::new(),
            name_counter: 0,
        };

        Graph::DiGraph {
            id: Id::Plain("myprog".to_string()),
            strict: true,
            stmts: self.to_dot_with(&mut dot_converter),
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
        };

        Graph::DiGraph {
            id: Id::Plain("myprog".to_string()),
            strict: true,
            stmts: self.to_dot_with(&mut dot_converter),
        }
    }

    fn to_dot_with(self: &RcExpr, conv: &mut DotConverter) -> Vec<Stmt> {
        let id = Rc::as_ptr(self);
        if !conv.done.insert(id) {
            return vec![];
        }
        match self.as_ref() {
            Expr::DoWhile(inputs, body) => {
                let mut stmts = inputs.to_dot_with(conv);
                stmts.push(Stmt::Edge(Edge {
                    ty: EdgeTy::Pair(conv.graphviz_vertex(self), conv.graphviz_vertex(inputs)),
                    attributes: vec![],
                }));
                stmts.push(Stmt::Subgraph(Subgraph {
                    stmts: body.to_dot_with(conv),
                    id: conv.graphviz_id(self),
                }));
                stmts
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
