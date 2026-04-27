// Port of egraphin.h / egraphin.cpp — minimal egraph types and helpers.
// Direct line-by-line translation.

use std::collections::VecDeque;
use std::io::{BufReader, Read};

pub type EClassId = i32;
pub type ENodeId = i32;
pub type ExtractionENodeId = i32;

// -1 used to denote an unextractable eclass
pub const UNEXTRACTABLE_ECLASS: EClassId = -1;

#[derive(Clone, Default)]
pub struct ENode {
    pub head: String,
    pub eclass: EClassId,
    pub ch: Vec<EClassId>,
}

impl ENode {
    pub fn get_name(&self) -> String {
        // C++: int pos = head.find("###"); return head.substr(0, pos);
        // If "###" is missing, fall back to the whole head string.
        let pos = self.head.find("###").unwrap_or(self.head.len());
        self.head[..pos].to_string()
    }

    pub fn get_op(&self) -> String {
        // C++: int pos = head.find("###"); return head.substr(pos + 3, head.length() - pos - 3);
        // If "###" is missing, fall back to empty string.
        match self.head.find("###") {
            Some(pos) => self.head[pos + 3..].to_string(),
            None => String::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct EClass {
    pub enodes: Vec<ENode>,
    pub isEffectful: bool,
}

impl EClass {
    pub fn nenodes(&self) -> usize {
        self.enodes.len()
    }
}

#[derive(Clone, Default)]
pub struct EGraph {
    pub eclasses: Vec<EClass>,
}

impl EGraph {
    pub fn neclasses(&self) -> usize {
        self.eclasses.len()
    }
}

// An extraction corespondes to a particular egraph

#[derive(Clone, Default)]
pub struct ExtractionENode {
    pub c: EClassId,
    pub n: ENodeId,
    pub ch: Vec<ExtractionENodeId>,
}

pub type Extraction = Vec<ExtractionENode>;

// A mapping from egraph A to B

#[derive(Clone, Default)]
pub struct EGraphMapping {
    pub eclassidmp: Vec<EClassId>,
    pub enodeidmp: Vec<Vec<ENodeId>>,
}

impl EGraphMapping {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_egraph(g: &EGraph) -> Self {
        let mut m = EGraphMapping {
            eclassidmp: vec![UNEXTRACTABLE_ECLASS; g.neclasses()],
            enodeidmp: vec![Vec::new(); g.neclasses()],
        };
        for i in 0..(g.neclasses() as EClassId) {
            let c = &g.eclasses[i as usize];
            m.enodeidmp[i as usize] = vec![UNEXTRACTABLE_ECLASS; c.nenodes()];
        }
        m
    }
}

// A reverse index for speed up things

pub type EClassParents = Vec<Vec<(EClassId, ENodeId)>>;

// An accompanying counter for optimizations

pub type ENodeCounters = Vec<Vec<i32>>;

pub fn compute_reverse_index(g: &EGraph) -> EClassParents {
    let mut ret: EClassParents = vec![Vec::new(); g.neclasses()];
    for i in 0..(g.neclasses() as EClassId) {
        let c = &g.eclasses[i as usize];
        for j in 0..(c.nenodes() as ENodeId) {
            let n = &c.enodes[j as usize];
            for k in 0..n.ch.len() {
                if n.ch[k] != UNEXTRACTABLE_ECLASS {
                    ret[n.ch[k] as usize].push((i, j));
                }
            }
        }
    }
    ret
}

pub fn initialize_enode_counters(g: &EGraph) -> ENodeCounters {
    let mut ret: ENodeCounters = vec![Vec::new(); g.neclasses()];
    for i in 0..(g.neclasses() as EClassId) {
        let c = &g.eclasses[i as usize];
        ret[i as usize].resize(c.nenodes(), 0);
        for j in 0..(c.nenodes() as ENodeId) {
            ret[i as usize][j as usize] = c.enodes[j as usize].ch.len() as i32;
        }
    }
    ret
}

pub fn inverse_egraph_mapping(gp: &EGraph, g2gp: &EGraphMapping) -> EGraphMapping {
    let mut gp2g = EGraphMapping::from_egraph(gp);
    for i in 0..(g2gp.eclassidmp.len() as EClassId) {
        if g2gp.eclassidmp[i as usize] != UNEXTRACTABLE_ECLASS {
            crate::debug_assert_tiger!(0 <= g2gp.eclassidmp[i as usize] && g2gp.eclassidmp[i as usize] < gp.neclasses() as EClassId);
            gp2g.eclassidmp[g2gp.eclassidmp[i as usize] as usize] = i;
        }
    }
    for i in 0..(g2gp.eclassidmp.len() as EClassId) {
        for j in 0..(g2gp.enodeidmp[i as usize].len() as ENodeId) {
            if g2gp.enodeidmp[i as usize][j as usize] != UNEXTRACTABLE_ECLASS {
                crate::debug_assert_tiger!(0 <= g2gp.enodeidmp[i as usize][j as usize] && g2gp.enodeidmp[i as usize][j as usize] < gp.eclasses[g2gp.eclassidmp[i as usize] as usize].nenodes() as ENodeId);
                gp2g.enodeidmp[g2gp.eclassidmp[i as usize] as usize][g2gp.enodeidmp[i as usize][j as usize] as usize] = j;
            }
        }
    }
    gp2g
}

pub fn project_extraction(f: &EGraphMapping, e: &Extraction) -> Extraction {
    let mut ne: Extraction = e.clone();
    for i in 0..(e.len() as ExtractionENodeId) {
        ne[i as usize].n = f.enodeidmp[ne[i as usize].c as usize][ne[i as usize].n as usize];
        ne[i as usize].c = f.eclassidmp[ne[i as usize].c as usize];
    }
    ne
}

// return a mapping from the old egraph ids to the new one
pub fn prune_unextractable_enodes(g: &EGraph, root: EClassId) -> (EGraph, EGraphMapping) {
    let mut extractable: Vec<bool> = vec![false; g.neclasses()];
    let parents: EClassParents = compute_reverse_index(g);
    let mut cnts: ENodeCounters = initialize_enode_counters(g);
    let mut q: VecDeque<EClassId> = VecDeque::new();
    for i in 0..(g.neclasses() as EClassId) {
        let c = &g.eclasses[i as usize];
        for j in 0..(c.nenodes() as ENodeId) {
            if cnts[i as usize][j as usize] == 0 {
                if !extractable[i as usize] {
                    extractable[i as usize] = true;
                    q.push_back(i);
                }
            }
        }
    }
    while q.len() > 0 {
        let u = *q.front().unwrap();
        q.pop_front();
        for i in 0..parents[u as usize].len() {
            let vc: EClassId = parents[u as usize][i].0;
            let vn: ENodeId = parents[u as usize][i].1;
            cnts[vc as usize][vn as usize] -= 1;
            if cnts[vc as usize][vn as usize] == 0 {
                if !extractable[vc as usize] {
                    extractable[vc as usize] = true;
                    q.push_back(vc);
                }
            }
        }
    }
    let mut reachable: Vec<bool> = vec![if root == -1 { true } else { false }; g.neclasses()];
    if root != -1 {
        reachable[root as usize] = true;
        q.push_back(root);
        while q.len() > 0 {
            let u = *q.front().unwrap();
            q.pop_front();
            let c = &g.eclasses[u as usize];
            for j in 0..(c.nenodes() as ENodeId) {
                let n = &c.enodes[j as usize];
                let mut isExtractable = true;
                for k in 0..n.ch.len() {
                    let v: EClassId = n.ch[k];
                    if v == UNEXTRACTABLE_ECLASS || !extractable[v as usize] {
                        isExtractable = false;
                        break;
                    }
                }
                if isExtractable {
                    for k in 0..n.ch.len() {
                        let v: EClassId = n.ch[k];
                        if !reachable[v as usize] {
                            reachable[v as usize] = true;
                            q.push_back(v);
                        }
                    }
                }
            }
        }
    }
    let mut gp: EGraph = EGraph::default();
    let mut mp: EGraphMapping = EGraphMapping::from_egraph(g);
    for i in 0..(g.neclasses() as EClassId) {
        if reachable[i as usize] && extractable[i as usize] {
            let c = &g.eclasses[i as usize];
            let mut nc = EClass::default();
            nc.isEffectful = c.isEffectful;
            mp.eclassidmp[i as usize] = gp.neclasses() as EClassId;
            gp.eclasses.push(nc);
        }
    }
    for i in 0..(g.neclasses() as EClassId) {
        if mp.eclassidmp[i as usize] != UNEXTRACTABLE_ECLASS {
            let c = &g.eclasses[i as usize];
            // Take immutable info we need before borrowing gp mutably below.
            let target_idx = mp.eclassidmp[i as usize] as usize;
            for j in 0..(c.nenodes() as ENodeId) {
                let n = &c.enodes[j as usize];
                let mut isExtractable = true;
                for k in 0..n.ch.len() {
                    let v: EClassId = n.ch[k];
                    if v == UNEXTRACTABLE_ECLASS || mp.eclassidmp[v as usize] == UNEXTRACTABLE_ECLASS {
                        isExtractable = false;
                        break;
                    }
                }
                if isExtractable {
                    let n = &c.enodes[j as usize];
                    let mut nn = ENode::default();
                    nn.head = n.head.clone();
                    nn.ch.resize(n.ch.len(), 0);
                    for k in 0..n.ch.len() {
                        nn.ch[k] = mp.eclassidmp[n.ch[k] as usize];
                    }
                    nn.eclass = mp.eclassidmp[i as usize];
                    let nc: &mut EClass = &mut gp.eclasses[target_idx];
                    mp.enodeidmp[i as usize][j as usize] = nc.nenodes() as ENodeId;
                    nc.enodes.push(nn);
                }
            }
        }
    }
    crate::debug_assert_tiger!(crate::debug::is_wellformed_egraph(&gp, false, true));
    crate::debug_assert_tiger!(crate::debug::is_valid_egraph_mapping(&mp, g, &gp, true, true, true, true));
    (gp, mp)
}

// Token reader that mirrors fscanf("%d") / fscanf("%zd") whitespace-delimited
// reads while still letting us pull a full line for the head string.
struct TokenReader<R: Read> {
    inner: BufReader<R>,
    // pushback for a single byte (used by peek-after-int to handle the newline)
    peeked: Option<u8>,
}

impl<R: Read> TokenReader<R> {
    fn new(r: R) -> Self {
        TokenReader { inner: BufReader::new(r), peeked: None }
    }

    fn read_byte(&mut self) -> Option<u8> {
        if let Some(b) = self.peeked.take() {
            return Some(b);
        }
        let mut buf = [0u8; 1];
        match self.inner.read(&mut buf) {
            Ok(0) => None,
            Ok(_) => Some(buf[0]),
            Err(_) => None,
        }
    }

    fn read_int_i32(&mut self) -> i32 {
        // skip whitespace
        let mut b;
        loop {
            match self.read_byte() {
                Some(c) if (c as char).is_ascii_whitespace() => continue,
                Some(c) => { b = c; break; }
                None => return 0,
            }
        }
        let mut s = String::new();
        // optional sign
        if b == b'-' || b == b'+' {
            s.push(b as char);
            b = self.read_byte().unwrap_or(b' ');
        }
        while (b as char).is_ascii_digit() {
            s.push(b as char);
            match self.read_byte() {
                Some(c) => b = c,
                None => { return s.parse::<i32>().unwrap_or(0); }
            }
        }
        // push back the non-digit byte (this is how fscanf leaves the stream)
        self.peeked = Some(b);
        s.parse::<i32>().unwrap_or(0)
    }

    fn read_int_usize(&mut self) -> usize {
        let v = self.read_int_i32();
        v as usize
    }

    // mirror fgets: read up to and including '\n' (or EOF)
    fn read_line_into(&mut self, buf: &mut String) {
        buf.clear();
        loop {
            match self.read_byte() {
                Some(c) => {
                    buf.push(c as char);
                    if c == b'\n' {
                        return;
                    }
                }
                None => return,
            }
        }
    }
}

pub fn read_egraph<R: Read>(input: &mut R) -> EGraph {
    let mut g: EGraph = EGraph::default();
    let mut tr = TokenReader::new(input);
    let mut cnt: i32 = 0;
    let mut buf: String = String::new();
    let n: i32 = tr.read_int_i32();
    g.eclasses.resize(n as usize, EClass::default());
    for i in 0..n {
        let f: i32;
        let m: i32;
        f = tr.read_int_i32();
        m = tr.read_int_i32();
        cnt += m;
        {
            let c: &mut EClass = &mut g.eclasses[i as usize];
            c.isEffectful = f != 0;
            c.enodes.resize(m as usize, ENode::default());
        }
        for j in 0..m {
            // handle names with spaces
            tr.read_line_into(&mut buf);
            tr.read_line_into(&mut buf);
            crate::debug_assert_tiger!(buf.len() > 1);
            // strip trailing newline (matches buf[strlen(buf)-1] = '\0')
            if buf.ends_with('\n') {
                buf.pop();
            }
            let l: usize = {
                // Set head + eclass before reading l so we don't hold a long borrow.
                let n_node: &mut ENode = &mut g.eclasses[i as usize].enodes[j as usize];
                n_node.head = buf.clone();
                n_node.eclass = i;
                0usize
            };
            let _ = l;
            let l: usize = tr.read_int_usize();
            {
                let n_node: &mut ENode = &mut g.eclasses[i as usize].enodes[j as usize];
                n_node.ch.resize(l, 0);
                for k in 0..l {
                    n_node.ch[k] = tr.read_int_i32();
                }
            }
            // scanf("%d", &n.cost);
        }
    }
    crate::debug_cerr!(" # eclasses: {}  # enodes : {}", n, cnt);
    g
}

pub fn print_egraph(g: &EGraph) {
    println!("{}", g.neclasses());
    for i in 0..(g.neclasses() as EClassId) {
        let c = &g.eclasses[i as usize];
        let f: i32 = if c.isEffectful { 1 } else { 0 };
        let m: i32 = c.nenodes() as i32;
        println!("{} {}", f, m);
        for j in 0..m {
            let n = &c.enodes[j as usize];
            let l: usize = n.ch.len();
            print!("{}\n{}{}", n.head, l, if l == 0 { '\n' } else { ' ' });
            for k in 0..l {
                print!("{}{}", n.ch[k], if k == l - 1 { '\n' } else { ' ' });
            }
            // printf("%d\n", n.cost);
        }
    }
}

pub fn print_extraction(g: &EGraph, e: &Extraction) {
    for i in 0..(e.len() as ExtractionENodeId) {
        print!(
            "#{} {}{} {} {}{}",
            i,
            e[i as usize].c,
            if g.eclasses[e[i as usize].c as usize].isEffectful { '!' } else { ' ' },
            e[i as usize].n,
            g.eclasses[e[i as usize].c as usize].enodes[e[i as usize].n as usize].head,
            if e[i as usize].ch.len() == 0 { '\n' } else { ' ' }
        );
        for j in 0..e[i as usize].ch.len() {
            print!("#{}{}", e[i as usize].ch[j], if j == e[i as usize].ch.len() - 1 { '\n' } else { ' ' });
        }
    }
}
