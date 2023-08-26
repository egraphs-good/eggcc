use std::collections::{BTreeMap, BTreeSet};
use std::iter::once;

use bril_rs::ConstOps;

use crate::cfg::Identifier;

use super::{Expr, Id, Operand, RvsdgBody, RvsdgFunction};

const SIMPLE_NODE_SIZE: f32 = 100.0;
const STROKE_WIDTH: f32 = SIMPLE_NODE_SIZE * 0.02;
const NODE_SPACING: f32 = SIMPLE_NODE_SIZE * 0.5;
const FONT_SIZE: f32 = SIMPLE_NODE_SIZE * 0.5;
const PORT_RADIUS: f32 = STROKE_WIDTH * 2.0;
const CORNER_RADIUS: f32 = NODE_SPACING * 0.2;
const REGION_SPACING: f32 = NODE_SPACING * 0.5;

#[derive(Debug)]
struct Region {
    srcs: usize,
    dsts: usize,
    nodes: BTreeMap<Id, Node>,
    edges: Vec<Edge>,
}

#[derive(Debug)]
enum Node {
    Unit(String, usize, usize),
    Match(Vec<(String, Region)>), // vec must be nonempty
    Loop(Region),
}

// Each edge goes from an output port to an input port.
// `None` refers to the region, `Some(i)` refers to the node at index `i`.
// The second number is the index of the port that is being referred to.
type Edge = ((Option<Id>, usize), (Option<Id>, usize));

struct Size {
    width: f32,
    height: f32,
}

struct Xml {
    tag: String,
    attributes: BTreeMap<String, String>,
    body: String,
}

impl Xml {
    fn new<'a>(
        tag: &str,
        attributes: impl IntoIterator<Item = (&'a str, &'a str)>,
        body: &str,
    ) -> Xml {
        Xml {
            tag: tag.to_owned(),
            attributes: attributes
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
            body: body.to_owned(),
        }
    }

    fn group(children: impl IntoIterator<Item = Xml>) -> Xml {
        Xml {
            tag: "g".to_owned(),
            attributes: BTreeMap::new(),
            body: children.into_iter().map(|xml| xml.to_string()).collect(),
        }
    }
}

impl ToString for Xml {
    fn to_string(self: &Xml) -> String {
        use std::fmt::Write;
        let mut out = String::new();

        write!(out, "<{}", self.tag).unwrap();
        for (key, value) in &self.attributes {
            write!(out, "\n\t{key}=\"{value}\"").unwrap();
        }
        writeln!(out, ">").unwrap();
        for line in self.body.lines() {
            writeln!(out, "\t{line}").unwrap();
        }
        writeln!(out, "</{}>", self.tag).unwrap();

        out
    }
}

fn blend(width: f32, total: usize, index: usize) -> f32 {
    width / ((total + 1) as f32) * (index + 1) as f32
}

impl Node {
    fn inputs(&self, width: f32) -> Vec<f32> {
        match self {
            Node::Unit(_, inputs, _) => (0..*inputs).map(|p| blend(width, *inputs, p)).collect(),
            Node::Match(branches) => {
                let inputs = 1 + branches[0].1.srcs;
                (0..inputs).map(|p| blend(width, inputs, p)).collect()
            }
            Node::Loop(region) => (0..region.srcs)
                .map(|p| REGION_SPACING + blend(width - REGION_SPACING * 2.0, region.srcs, p))
                .collect(),
        }
    }

    fn outputs(&self, width: f32) -> Vec<f32> {
        match self {
            Node::Unit(_, _, outputs) => (0..*outputs).map(|p| blend(width, *outputs, p)).collect(),
            Node::Match(branches) => (0..branches[0].1.dsts)
                .map(|p| blend(width, branches[0].1.dsts, p))
                .collect(),
            Node::Loop(region) => (1..region.dsts)
                .map(|p| REGION_SPACING + blend(width - REGION_SPACING * 2.0, region.dsts, p))
                .collect(),
        }
    }
}

fn port(x: f32, y: f32, color: &str) -> Xml {
    Xml::new(
        "circle",
        [
            ("fill", color),
            ("stroke", "black"),
            ("stroke-width", &format!("{}", STROKE_WIDTH)),
            ("r", &format!("{}", PORT_RADIUS)),
            ("cx", &format!("{}", x)),
            ("cy", &format!("{}", y)),
        ],
        "",
    )
}

impl Node {
    fn to_xml(&self) -> (Size, Xml) {
        match self {
            Node::Unit(text, _, _) => {
                let size = Size {
                    width: SIMPLE_NODE_SIZE,
                    height: SIMPLE_NODE_SIZE,
                };
                let node = Xml::group([
                    Xml::new(
                        "rect",
                        [
                            ("fill", "#FFFF80"),
                            ("stroke", "black"),
                            ("width", &format!("{}", SIMPLE_NODE_SIZE)),
                            ("height", &format!("{}", SIMPLE_NODE_SIZE)),
                            ("stroke-width", &format!("{}", STROKE_WIDTH)),
                            ("rx", &format!("{}", CORNER_RADIUS)),
                        ],
                        "",
                    ),
                    Xml::new(
                        "text",
                        [
                            ("text-anchor", "middle"),
                            ("fill", "black"),
                            ("x", &format!("{}", SIMPLE_NODE_SIZE * 0.5)),
                            ("y", &format!("{}", FONT_SIZE * 1.25)),
                            ("font-size", &format!("{}", FONT_SIZE)),
                        ],
                        text,
                    ),
                ]);
                let inputs = self
                    .inputs(size.width)
                    .into_iter()
                    .map(|x| port(x, 0.0, "white"));
                let outputs = self
                    .outputs(size.width)
                    .into_iter()
                    .map(|x| port(x, size.height, "white"));
                let group = Xml::group(once(node).chain(inputs).chain(outputs));
                (size, group)
            }
            Node::Match(branches) => {
                let children: Vec<_> = branches.iter().map(|t| t.1.to_xml(false)).collect();
                let size = Size {
                    width: REGION_SPACING * (children.len() + 1) as f32
                        + children.iter().map(|t| t.0.width).sum::<f32>(),
                    height: FONT_SIZE * 2.0
                        + REGION_SPACING * 2.0
                        + children
                            .iter()
                            .map(|t| t.0.height)
                            .max_by(f32::total_cmp)
                            .unwrap_or(0.0),
                };

                let background = Xml::new(
                    "rect",
                    [
                        ("fill", "#80FF80"),
                        ("stroke", "black"),
                        ("width", &format!("{}", size.width)),
                        ("height", &format!("{}", size.height)),
                        ("stroke-width", &format!("{}", STROKE_WIDTH)),
                        ("rx", &format!("{}", CORNER_RADIUS)),
                    ],
                    "",
                );
                let text = Xml::new(
                    "text",
                    [
                        ("fill", "black"),
                        ("x", &format!("{}", REGION_SPACING)),
                        ("y", &format!("{}", FONT_SIZE)),
                        ("font-size", &format!("{}", FONT_SIZE)),
                    ],
                    "match",
                );

                let mut w = 0.0;
                let positions: Vec<(f32, f32)> = children
                    .iter()
                    .map(|(s, _)| {
                        w += s.width + REGION_SPACING;
                        (w - s.width, REGION_SPACING + FONT_SIZE * 2.0)
                    })
                    .collect();
                assert_eq!(w + REGION_SPACING, size.width);

                let branches = positions.iter().zip(children).zip(branches).map(
                    |(((x, y), (s, mut xml)), (label, _))| {
                        xml.attributes
                            .insert("transform".to_owned(), format!("translate({x}, {y})"));
                        let label = Xml::new(
                            "text",
                            [
                                ("text-anchor", "middle"),
                                ("fill", "black"),
                                ("x", &format!("{}", x + s.width * 0.5)),
                                ("y", &format!("{}", FONT_SIZE * 2.0)),
                                ("font-size", &format!("{}", FONT_SIZE)),
                            ],
                            label,
                        );
                        Xml::group([xml, label])
                    },
                );

                let c = |p| match p == 0 {
                    true => "black",
                    false => "white",
                };
                let inputs = self
                    .inputs(size.width)
                    .into_iter()
                    .enumerate()
                    .map(|(i, x)| port(x, 0.0, c(i)));
                let outputs = self
                    .outputs(size.width)
                    .into_iter()
                    .map(|x| port(x, size.height, "white"));
                let group = Xml::group(
                    vec![background, text]
                        .into_iter()
                        .chain(branches)
                        .chain(inputs)
                        .chain(outputs),
                );
                (size, group)
            }
            Node::Loop(region) => {
                let (s, mut xml) = region.to_xml(true);
                let size = Size {
                    width: s.width + REGION_SPACING * 2.0,
                    height: s.height + FONT_SIZE + REGION_SPACING * 2.0,
                };

                let background = Xml::new(
                    "rect",
                    [
                        ("fill", "#FF8080"),
                        ("stroke", "black"),
                        ("width", &format!("{}", size.width)),
                        ("height", &format!("{}", size.height)),
                        ("stroke-width", &format!("{}", STROKE_WIDTH)),
                        ("rx", &format!("{}", CORNER_RADIUS)),
                    ],
                    "",
                );
                let text = Xml::new(
                    "text",
                    [
                        ("fill", "black"),
                        ("x", &format!("{}", REGION_SPACING)),
                        ("y", &format!("{}", FONT_SIZE)),
                        ("font-size", &format!("{}", FONT_SIZE)),
                    ],
                    "loop",
                );

                let (x, y) = (REGION_SPACING, REGION_SPACING + FONT_SIZE);
                xml.attributes
                    .insert("transform".to_owned(), format!("translate({x}, {y})"));

                let inputs = self
                    .inputs(size.width)
                    .into_iter()
                    .map(|x| port(x, 0.0, "white"));
                let outputs = self
                    .outputs(size.width)
                    .into_iter()
                    .map(|x| port(x, size.height, "white"));
                let group = Xml::group(
                    vec![background, text, xml]
                        .into_iter()
                        .chain(inputs)
                        .chain(outputs),
                );
                (size, group)
            }
        }
    }
}

impl Region {
    fn to_xml(&self, in_loop: bool) -> (Size, Xml) {
        let mut edges = self.edges.clone();
        edges.sort();

        let mut children: BTreeMap<_, _> = self.nodes.iter().map(|t| (t.0, t.1.to_xml())).collect();

        let mut layers: Vec<Vec<Id>> = vec![];
        let mut to_order: BTreeSet<Id> = self.nodes.keys().copied().collect();
        while !to_order.is_empty() {
            let mut next_layer = to_order.clone();
            for ((a, _), (b, _)) in &edges {
                if let (Some(a), Some(b)) = (a, b) {
                    if to_order.contains(b) {
                        next_layer.remove(a);
                    }
                }
            }
            to_order.retain(|node| !next_layer.contains(node));

            let mut next_layer: Vec<Id> = next_layer.into_iter().collect();
            next_layer.sort(); // determinism
            layers.push(next_layer);
        }

        let sizes: Vec<Size> = layers
            .iter()
            .map(|layer| Size {
                width: layer.iter().map(|node| children[node].0.width).sum::<f32>(),
                height: layer
                    .iter()
                    .map(|node| children[node].0.height)
                    .max_by(f32::total_cmp)
                    .unwrap(),
            })
            .collect();

        let size = Size {
            width: sizes.iter().map(|s| s.width).sum::<f32>()
                + NODE_SPACING * (children.len() + 1) as f32,
            height: sizes.iter().map(|s| s.height).sum::<f32>()
                + NODE_SPACING * (layers.len() + 1) as f32,
        };

        let mut w = NODE_SPACING;
        let mut h = NODE_SPACING;
        let positions: BTreeMap<Id, (f32, f32)> = layers
            .iter()
            .zip(sizes)
            .rev()
            .flat_map(|(nodes, Size { height, .. })| {
                let out: Vec<_> = nodes
                    .iter()
                    .map(|node| {
                        let out = (*node, (w, h));
                        w += children[node].0.width + NODE_SPACING;
                        out
                    })
                    .collect();
                h += height + NODE_SPACING;
                out
            })
            .collect();
        assert_eq!(w, size.width);
        assert_eq!(h, size.height);

        let edges = Xml::group(edges.iter().map(|((a, i), (b, j))| {
            let (a_x, a_y) = match a {
                None => (blend(size.width, self.srcs, *i), 0.0),
                Some(a) => (
                    positions[a].0 + self.nodes[a].outputs(children[&a].0.width)[*i],
                    positions[a].1 + children[&a].0.height,
                ),
            };
            let (b_x, b_y) = match b {
                None => (blend(size.width, self.dsts, *j), size.height),
                Some(b) => (
                    positions[b].0 + self.nodes[b].inputs(children[b].0.width)[*j],
                    positions[b].1,
                ),
            };

            let break_y = match a {
                _ if b.is_none() => {
                    b_y - NODE_SPACING
                        + CORNER_RADIUS
                        + blend(NODE_SPACING - CORNER_RADIUS * 2.0, self.dsts, *j)
                }
                Some(a) => {
                    let layer = layers.iter().find(|layer| layer.contains(a)).unwrap();
                    let total = layer
                        .iter()
                        .map(|node| self.nodes[node].outputs(0.0).len())
                        .sum();
                    let mut index = 0;
                    for node in layer.iter().rev() {
                        if node == a {
                            index += *i;
                            break;
                        }
                        index += self.nodes[node].outputs(0.0).len();
                    }
                    a_y + CORNER_RADIUS + blend(NODE_SPACING - CORNER_RADIUS * 2.0, total, index)
                }
                None => {
                    a_y + CORNER_RADIUS + blend(NODE_SPACING - CORNER_RADIUS * 2.0, self.srcs, *i)
                }
            };

            let positive_offset = ((b_x - a_x) * 0.5).abs().min(CORNER_RADIUS);
            let direction_offset = match a_x < b_x {
                true => positive_offset,
                false => -positive_offset,
            };
            let arc_1_a_y = break_y - CORNER_RADIUS;
            let arc_1_b_x = a_x + direction_offset;
            let arc_2_a_x = b_x - direction_offset;
            let arc_2_b_y = break_y + CORNER_RADIUS;

            let path_string = format!(
                "M {a_x} {a_y}\
                 V {arc_1_a_y}\
                 Q {a_x} {break_y} {arc_1_b_x} {break_y}\
                 H {arc_2_a_x}\
                 Q {b_x} {break_y} {b_x} {arc_2_b_y}\
                 V {b_y}"
            );
            Xml::new(
                "path",
                [
                    ("fill", "transparent"),
                    ("stroke", "black"),
                    ("stroke-linecap", "round"),
                    ("stroke-width", &format!("{}", STROKE_WIDTH)),
                    ("d", &path_string),
                ],
                "",
            )
        }));

        let (s, d) = (self.srcs, self.dsts);
        let c = |p| if p == 0 && in_loop { "black" } else { "white" };
        let srcs = Xml::group((0..s).map(|p| port(blend(size.width, s, p), 0.0, "white")));
        let dsts = Xml::group((0..d).map(|p| port(blend(size.width, d, p), size.height, c(p))));

        let nodes = Xml::group(positions.iter().map(|(id, (x, y))| {
            let (_, mut xml) = children.remove(id).unwrap();
            xml.attributes
                .insert("transform".to_owned(), format!("translate({x}, {y})"));
            xml
        }));

        let background = Xml::new(
            "rect",
            [
                ("fill", "white"),
                ("stroke", "black"),
                ("stroke-width", &format!("{}", STROKE_WIDTH)),
                ("width", &format!("{}", size.width)),
                ("height", &format!("{}", size.height)),
                ("rx", &format!("{}", CORNER_RADIUS)),
            ],
            "",
        );

        (size, Xml::group([background, edges, nodes, srcs, dsts]))
    }
}

impl Region {
    fn to_svg(&self) -> String {
        let (size, xml) = self.to_xml(false);
        let svg = Xml::new(
            "svg",
            [
                ("version", "1.1"),
                ("width", &format!("{}", size.width)),
                ("height", &format!("{}", size.height)),
                ("xmlns", "http://www.w3.org/2000/svg"),
            ],
            &xml.to_string(),
        );

        svg.to_string()
    }
}

fn mk_node_and_input_edges(index: Id, nodes: &[RvsdgBody]) -> (Node, Vec<Edge>) {
    let (node, operands): (Node, Vec<Operand>) = match &nodes[index] {
        RvsdgBody::PureOp(Expr::Op(f, xs)) => {
            (Node::Unit(format!("{f}"), xs.len(), 1), xs.to_vec())
        }
        RvsdgBody::PureOp(Expr::Call(f, xs)) => (
            Node::Unit(
                match f {
                    Identifier::Name(s) => (**s).to_owned(),
                    Identifier::Num(x) => format!("{x}"),
                },
                xs.len(),
                1,
            ),
            xs.to_vec(),
        ),
        RvsdgBody::PureOp(Expr::Const(ConstOps::Const, _, v)) => {
            (Node::Unit(format!("{v}"), 0, 1), vec![])
        }
        RvsdgBody::Gamma {
            pred,
            inputs,
            outputs,
        } => (
            Node::Match(
                outputs
                    .iter()
                    .enumerate()
                    .map(|(pred, os)| (format!("{pred}"), mk_region(inputs.len(), os, nodes)))
                    .collect(),
            ),
            once(pred).chain(inputs).copied().collect::<Vec<_>>(),
        ),
        RvsdgBody::Theta {
            pred,
            inputs,
            outputs,
        } => (
            Node::Loop(mk_region(
                inputs.len(),
                &once(pred).chain(outputs).copied().collect::<Vec<_>>(),
                nodes,
            )),
            inputs.to_vec(),
        ),
    };
    let input_edges = operands
        .iter()
        .enumerate()
        .map(|(j, x)| {
            (
                match x {
                    Operand::Arg(i) => (None, *i),
                    Operand::Id(id) => (Some(*id), 0),
                    Operand::Project(i, id) => (Some(*id), *i),
                },
                (Some(index), j),
            )
        })
        .collect();
    (node, input_edges)
}

// returns only nodes in the same REGION as `output`
fn reachable_nodes(reachable: &mut BTreeSet<Id>, all: &[RvsdgBody], output: Operand) {
    let id = match output {
        Operand::Arg(..) => return,
        Operand::Id(id) => id,
        Operand::Project(_, id) => id,
    };
    if reachable.insert(id) {
        let inputs = match &all[id] {
            RvsdgBody::PureOp(Expr::Op(_, xs)) | RvsdgBody::PureOp(Expr::Call(_, xs)) => xs.clone(),
            RvsdgBody::PureOp(Expr::Const(..)) => vec![],
            RvsdgBody::Gamma { pred, inputs, .. } => once(pred).chain(inputs).copied().collect(),
            RvsdgBody::Theta { inputs, .. } => inputs.clone(),
        };
        for input in inputs {
            reachable_nodes(reachable, all, input);
        }
    }
}

fn mk_region(srcs: usize, dsts: &[Operand], nodes: &[RvsdgBody]) -> Region {
    let mut reachable = BTreeSet::new();
    dsts.iter().for_each(|operand| {
        reachable_nodes(&mut reachable, nodes, *operand);
    });

    let (nodes, edges): (BTreeMap<_, _>, Vec<_>) = reachable
        .iter()
        .map(|i| {
            let (node, edges) = mk_node_and_input_edges(*i, nodes);
            ((*i, node), edges)
        })
        .unzip();
    let mut edges = edges.concat();
    // add edges to dsts
    edges.extend(dsts.iter().enumerate().map(|(j, x)| {
        (
            match x {
                Operand::Arg(i) => (None, *i),
                Operand::Id(id) => (Some(*id), 0),
                Operand::Project(i, id) => (Some(*id), *i),
            },
            (None, j),
        )
    }));

    Region {
        srcs,
        dsts: dsts.len(),
        nodes,
        edges,
    }
}

impl RvsdgFunction {
    pub(crate) fn to_svg(&self) -> String {
        let dsts: Vec<_> = self.result.iter().copied().collect();
        mk_region(self.n_args, &dsts, &self.nodes).to_svg()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bril_rs::{Literal, Type, ValueOps};

    #[test]
    fn rvsdg2svg_basic() {
        let svg_old = Region {
            srcs: 2,
            dsts: 1,
            nodes: BTreeMap::from([
                (
                    0,
                    Node::Match(vec![
                        (
                            "0".to_owned(),
                            Region {
                                srcs: 2,
                                dsts: 1,
                                nodes: BTreeMap::from([(0, Node::Unit("0".to_owned(), 0, 1))]),
                                edges: vec![((Some(0), 0), (None, 0))],
                            },
                        ),
                        (
                            "1".to_owned(),
                            Region {
                                srcs: 2,
                                dsts: 1,
                                nodes: BTreeMap::from([(0, Node::Unit("add".to_owned(), 2, 1))]),
                                edges: vec![
                                    ((None, 0), (Some(0), 0)),
                                    ((None, 1), (Some(0), 1)),
                                    ((Some(0), 0), (None, 0)),
                                ],
                            },
                        ),
                    ]),
                ),
                (
                    1,
                    Node::Loop(Region {
                        srcs: 3,
                        dsts: 4,
                        nodes: BTreeMap::from([
                            (0, Node::Unit("add".to_owned(), 2, 1)),
                            (1, Node::Unit("1".to_owned(), 0, 1)),
                            (2, Node::Unit("5".to_owned(), 0, 1)),
                            (3, Node::Unit("mul".to_owned(), 2, 1)),
                            (4, Node::Unit("add".to_owned(), 2, 1)),
                            (5, Node::Unit("eq".to_owned(), 2, 1)),
                        ]),
                        edges: vec![
                            ((None, 0), (Some(0), 0)),
                            ((Some(1), 0), (Some(0), 1)),
                            ((None, 0), (Some(3), 0)),
                            ((Some(2), 0), (Some(3), 1)),
                            ((Some(2), 0), (Some(4), 0)),
                            ((None, 2), (Some(4), 1)),
                            ((Some(5), 0), (None, 0)),
                            ((Some(0), 0), (None, 1)),
                            ((Some(3), 0), (None, 2)),
                            ((Some(4), 0), (None, 3)),
                            ((Some(0), 0), (Some(5), 0)),
                            ((Some(2), 0), (Some(5), 1)),
                        ],
                    }),
                ),
                (2, Node::Unit("add".to_owned(), 2, 1)),
            ]),
            edges: vec![
                ((None, 0), (Some(0), 0)),
                ((None, 0), (Some(0), 1)),
                ((None, 1), (Some(0), 2)),
                ((None, 0), (Some(1), 0)),
                ((None, 1), (Some(1), 1)),
                ((None, 0), (Some(1), 2)),
                ((Some(0), 0), (Some(2), 0)),
                ((Some(1), 1), (Some(2), 1)),
                ((Some(2), 0), (None, 0)),
            ],
        }
        .to_svg();

        let svg_new = RvsdgFunction {
            n_args: 2,
            nodes: vec![
                RvsdgBody::PureOp(Expr::Const(ConstOps::Const, Type::Int, Literal::Int(0))),
                RvsdgBody::PureOp(Expr::Op(
                    ValueOps::Add,
                    vec![Operand::Arg(0), Operand::Arg(1)],
                )),
                RvsdgBody::Gamma {
                    pred: Operand::Arg(0),
                    inputs: vec![Operand::Arg(0), Operand::Arg(1)],
                    outputs: vec![vec![Operand::Id(0)], vec![Operand::Id(1)]],
                },
                RvsdgBody::PureOp(Expr::Op(
                    ValueOps::Add,
                    vec![Operand::Arg(0), Operand::Id(4)],
                )),
                RvsdgBody::PureOp(Expr::Const(ConstOps::Const, Type::Int, Literal::Int(1))),
                RvsdgBody::PureOp(Expr::Const(ConstOps::Const, Type::Int, Literal::Int(5))),
                RvsdgBody::PureOp(Expr::Op(
                    ValueOps::Mul,
                    vec![Operand::Arg(0), Operand::Id(5)],
                )),
                RvsdgBody::PureOp(Expr::Op(
                    ValueOps::Add,
                    vec![Operand::Id(5), Operand::Arg(2)],
                )),
                RvsdgBody::PureOp(Expr::Op(ValueOps::Eq, vec![Operand::Id(3), Operand::Id(5)])),
                RvsdgBody::Theta {
                    pred: Operand::Id(8),
                    inputs: vec![Operand::Arg(0), Operand::Arg(1), Operand::Arg(0)],
                    outputs: vec![Operand::Id(3), Operand::Id(6), Operand::Id(7)],
                },
                RvsdgBody::PureOp(Expr::Op(
                    ValueOps::Add,
                    vec![Operand::Id(2), Operand::Project(1, 9)],
                )),
            ],
            result: Some(Operand::Id(10)),
        }
        .to_svg();

        assert_eq!(svg_old, svg_new);
    }
}
