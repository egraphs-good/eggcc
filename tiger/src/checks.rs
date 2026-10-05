//! Invariant checks used in `debug_assert!`s. Each prints what went wrong and
//! returns `false` instead of panicking so the assertion site is reported.

use std::collections::VecDeque;

use crate::egraph::{EClassId, EGraph, EGraphMapping, Extraction, ExtractionId};
use crate::statewalk::Statewalk;

/// Every e-node's children are in range and (unless `allow_subregion_children`)
/// at most one child is effectful. Empty e-classes are allowed only if `allow_empty`.
pub fn is_wellformed(g: &EGraph, allow_empty: bool, allow_subregion_children: bool) -> bool {
    let mut ok = true;
    for c in g.class_ids() {
        let class = &g.classes[c];
        if !allow_empty && class.enodes.is_empty() {
            ok = false;
            eprintln!("Error: empty e-class {c}");
        }
        for (n, enode) in class.enodes.iter().enumerate() {
            let mut effectful_children = 0;
            for &child in &enode.children {
                if child >= g.len() {
                    ok = false;
                    eprintln!("Error: e-node ({c}, {n}) has out-of-range child {child}");
                } else if g.is_effectful(child) {
                    effectful_children += 1;
                }
            }
            if !allow_subregion_children && effectful_children > 1 {
                ok = false;
                eprintln!("Error: e-node ({c}, {n}) has a subregion child");
            }
        }
    }
    ok
}

/// `mapping` is a valid mapping from `source` into `target`.
pub fn is_valid_mapping(
    mapping: &EGraphMapping,
    source: &EGraph,
    target: &EGraph,
    partial: bool,
    injective: bool,
    surjective: bool,
    check_children: bool,
) -> bool {
    if mapping.classes.len() != source.len() || mapping.enodes.len() != source.len() {
        eprintln!("Error: mapping domain has the wrong number of e-classes");
        return false;
    }
    let mut hit: Vec<Vec<bool>> = target
        .classes
        .iter()
        .map(|class| vec![false; class.enodes.len()])
        .collect();
    for c in source.class_ids() {
        let class = &source.classes[c];
        if mapping.enodes[c].len() != class.enodes.len() {
            eprintln!("Error: mapping domain has the wrong number of e-nodes in e-class {c}");
            return false;
        }
        let Some(tc) = mapping.classes[c] else {
            if partial {
                continue;
            }
            eprintln!("Error: e-class {c} is unmapped");
            return false;
        };
        if tc >= target.len() {
            eprintln!("Error: e-class {c} maps to out-of-range e-class {tc}");
            return false;
        }
        let tclass = &target.classes[tc];
        if class.is_effectful != tclass.is_effectful {
            eprintln!("Error: e-class {c} and its image {tc} differ in effectfulness");
            return false;
        }
        for (n, enode) in class.enodes.iter().enumerate() {
            let Some(tn) = mapping.enodes[c][n] else {
                if partial {
                    continue;
                }
                eprintln!("Error: e-node ({c}, {n}) is unmapped");
                return false;
            };
            if tn >= tclass.enodes.len() {
                eprintln!("Error: e-node ({c}, {n}) maps to out-of-range e-node ({tc}, {tn})");
                return false;
            }
            if injective && hit[tc][tn] {
                eprintln!("Error: mapping is not injective at e-node ({tc}, {tn})");
                return false;
            }
            hit[tc][tn] = true;
            if check_children {
                let tenode = &tclass.enodes[tn];
                if enode.children.len() != tenode.children.len() {
                    eprintln!(
                        "Error: e-node ({c}, {n}) and its image ({tc}, {tn}) differ in arity"
                    );
                    return false;
                }
                for (&child, &tchild) in enode.children.iter().zip(&tenode.children) {
                    if mapping.classes[child] != Some(tchild) {
                        eprintln!(
                            "Error: child {child} of e-node ({c}, {n}) does not map to {tchild}"
                        );
                        return false;
                    }
                }
            }
        }
    }
    if surjective {
        for (tc, nodes) in hit.iter().enumerate() {
            if let Some(tn) = nodes.iter().position(|&h| !h) {
                eprintln!("Error: mapping is not surjective; e-node ({tc}, {tn}) is not hit");
                return false;
            }
        }
    }
    true
}

/// A region has exactly one `Arg`: one leaf e-node among its effectful e-classes.
pub fn has_single_arg(g: &EGraph) -> bool {
    let args: Vec<EClassId> = g
        .class_ids()
        .filter(|&c| g.is_effectful(c))
        .flat_map(|c| {
            g.classes[c]
                .enodes
                .iter()
                .filter(|n| n.is_leaf())
                .map(move |_| c)
        })
        .collect();
    match args.len() {
        1 => true,
        0 => {
            eprintln!("Error: region has no Arg");
            false
        }
        k => {
            eprintln!("Error: region has {k} Arg e-nodes (in e-classes {args:?})");
            false
        }
    }
}

/// `statewalk` runs from `root` down to a leaf, each step through the
/// effectful child of the previous e-node.
pub fn is_valid_statewalk(g: &EGraph, root: EClassId, statewalk: &Statewalk) -> bool {
    if statewalk.first().map(|s| s.0) != Some(root) {
        eprintln!("Error: statewalk does not start at the root");
        return false;
    }
    for (i, &(c, n)) in statewalk.iter().enumerate() {
        if c >= g.len() || n >= g.classes[c].enodes.len() {
            eprintln!("Error: statewalk step ({c}, {n}) is out of range");
            return false;
        }
        let enode = g.enode(c, n);
        match statewalk.get(i + 1) {
            Some(&(next, _)) => {
                let effectful_children: Vec<EClassId> = enode
                    .children
                    .iter()
                    .copied()
                    .filter(|&ch| g.is_effectful(ch))
                    .collect();
                if effectful_children.last() != Some(&next) {
                    eprintln!("Error: statewalk step {i} does not lead to the next e-class {next}");
                    return false;
                }
            }
            None => {
                if !enode.is_leaf() {
                    eprintln!("Error: statewalk does not end at an Arg");
                    return false;
                }
            }
        }
    }
    true
}

/// `extraction` is a well-formed term DAG over `g` with `root` last: every
/// node is in range, has the right children, and only refers to earlier nodes.
pub fn is_valid_extraction(g: &EGraph, root: EClassId, extraction: &Extraction) -> bool {
    if extraction.last().map(|en| en.class) != Some(root) {
        eprintln!("Error: the last node of the extraction is not the root");
        return false;
    }
    for (i, en) in extraction.iter().enumerate() {
        if en.class >= g.len() || en.node >= g.classes[en.class].enodes.len() {
            eprintln!("Error: extraction node {i} refers to an out-of-range e-node");
            return false;
        }
        let enode = g.enode(en.class, en.node);
        if en.children.len() != enode.children.len() {
            eprintln!("Error: extraction node {i} has the wrong number of children");
            return false;
        }
        for (&child, &class) in en.children.iter().zip(&enode.children) {
            if child >= i {
                eprintln!("Error: extraction node {i} refers to a later node {child} (cycle?)");
                return false;
            }
            if extraction[child].class != class {
                eprintln!("Error: child {child} of extraction node {i} is in the wrong e-class");
                return false;
            }
        }
    }
    true
}

/// `extraction` is effect safe: within each region, pure nodes only use
/// effectful nodes on that region's statewalk.
pub fn is_effect_safe(g: &EGraph, root: EClassId, extraction: &Extraction) -> bool {
    if !is_valid_extraction(g, root, extraction) {
        return false;
    }
    let mut checked = vec![false; extraction.len()];
    is_effect_safe_region(g, extraction.len() - 1, extraction, &mut checked)
}

fn is_effect_safe_region(
    g: &EGraph,
    root: ExtractionId,
    extraction: &Extraction,
    checked: &mut [bool],
) -> bool {
    let is_effectful = |id: ExtractionId| g.is_effectful(extraction[id].class);
    let mut on_walk = vec![false; extraction.len()];
    let mut visited = vec![false; extraction.len()];
    let mut pure_queue = VecDeque::new();
    // Follow the statewalk; recurse into subregions.
    let mut walk = vec![root];
    on_walk[root] = true;
    let mut i = 0;
    while i < walk.len() {
        let mut next = None;
        for &child in &extraction[walk[i]].children {
            if is_effectful(child) {
                if next.is_none() {
                    next = Some(child);
                    walk.push(child);
                    on_walk[child] = true;
                } else if !checked[child] && !is_effect_safe_region(g, child, extraction, checked) {
                    return false;
                }
            } else if !visited[child] {
                visited[child] = true;
                pure_queue.push_back(child);
            }
        }
        i += 1;
    }
    // Pure nodes may only depend on effectful nodes on this region's statewalk.
    while let Some(u) = pure_queue.pop_front() {
        for &child in &extraction[u].children {
            if is_effectful(child) {
                if !on_walk[child] {
                    eprintln!("Error: pure node {u} uses effectful node {child}, which is not on its region's statewalk");
                    return false;
                }
            } else if !visited[child] {
                visited[child] = true;
                pure_queue.push_back(child);
            }
        }
    }
    checked[root] = true;
    true
}
