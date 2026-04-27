// Port of persistent_btree.h — generic persistent B-tree + two instantiations.

pub type PBId = u32;
pub type DataType = u32;

pub const INIT_SIZE: usize = 4000000;
pub const GROWTH_FACTOR: usize = 4;

// BP and S represent actual sizes as power of 2
pub struct PersistentBTree<const BP: usize, const S: usize> {
    pub(crate) top: i32,

    pub(crate) len: i32,

    pub(crate) height: i32,

    pub(crate) mem: Vec<PBId>,

    pub(crate) timestamp: i32,
}

impl<const BP: usize, const S: usize> Default for PersistentBTree<BP, S> {
    fn default() -> Self {
        Self {
            top: 0,
            len: 0,
            height: 0,
            mem: Vec::new(),
            timestamp: 0,
        }
    }
}

impl<const BP: usize, const S: usize> PersistentBTree<BP, S> {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub(crate) fn B(&self) -> usize {
        1 << BP
    }

    #[inline]
    pub(crate) fn getts(&self, i: PBId) -> u32 {
        self.mem[i as usize]
    }

    #[inline]
    pub(crate) fn getts_mut(&mut self, i: PBId) -> &mut u32 {
        &mut self.mem[i as usize]
    }

    #[inline]
    pub(crate) fn getc(&self, i: PBId, j: i32) -> u32 {
        self.mem[(i as usize) + 1 + (j as usize)]
    }

    #[inline]
    pub(crate) fn getc_mut(&mut self, i: PBId, j: i32) -> &mut u32 {
        &mut self.mem[(i as usize) + 1 + (j as usize)]
    }

    pub(crate) fn new_node(&mut self) -> PBId {
        if (self.top as usize) + 1 + self.B() > self.mem.len() {
            self.mem.resize(self.mem.len() * GROWTH_FACTOR, 0);
        }
        let ret = self.top as PBId;
        self.top += 1 + self.B() as i32;
        *self.getts_mut(ret) = self.timestamp as u32;
        let b = self.B();
        for i in 0..b {
            *self.getc_mut(ret, i as i32) = 0;
        }
        ret
    }

    pub(crate) fn new_node_from(&mut self, ori: PBId) -> PBId {
        if (self.top as usize) + 1 + self.B() > self.mem.len() {
            self.mem.resize(self.mem.len() * GROWTH_FACTOR, 0);
        }
        let ret = self.top as PBId;
        self.top += 1 + self.B() as i32;
        *self.getts_mut(ret) = self.timestamp as u32;
        let b = self.B();
        let src = (ori as usize) + 1;
        let dst = (ret as usize) + 1;
        self.mem.copy_within(src..src + b, dst);
        ret
    }

    #[inline]
    pub(crate) fn single_cell_capacity_P(&self) -> usize {
        5 - S
    }

    #[inline]
    pub(crate) fn single_cell_capacity(&self) -> usize {
        (std::mem::size_of::<PBId>() * 8) >> S
    }

    #[inline]
    pub(crate) fn single_node_capacity_P(&self) -> usize {
        self.single_cell_capacity_P() + BP
    }

    #[inline]
    pub(crate) fn single_node_capacity(&self) -> usize {
        self.single_cell_capacity() << BP
    }

    #[inline]
    pub(crate) fn chid(&self, h: i32, i: i32) -> i32 {
        ((i >> ((h * BP as i32) + self.single_cell_capacity_P() as i32))
            & (self.B() as i32 - 1)) as i32
    }

    #[inline]
    pub(crate) fn pos(&self, i: i32) -> i32 {
        i & (self.single_cell_capacity() as i32 - 1)
    }

    pub(crate) fn init_recurse(&mut self, l: i32, h: i32, data: &[DataType]) -> PBId {
        let cur = self.new_node();
        if h == 0 {
            let snc = self.single_node_capacity() as i32;
            let mut i: i32 = 0;
            while i < snc && ((l + i) as usize) < data.len() {
                let chid0i = self.chid(0, i);
                let posi = self.pos(i);
                let add = data[(l + i) as usize] << (posi << S);
                let c = self.getc_mut(cur, chid0i);
                *c |= add;
                i += 1;
            }
        } else {
            let b = self.B();
            for i in 0..b {
                let cl: i32 = l + ((i as i32) << (h * BP as i32 + self.single_cell_capacity_P() as i32));
                if cl as usize >= data.len() {
                    *self.getc_mut(cur, i as i32) = u32::MAX; // -1 cast to unsigned
                } else {
                    let child = self.init_recurse(cl, h - 1, data);
                    *self.getc_mut(cur, i as i32) = child;
                }
            }
        }
        cur
    }

    pub fn new_version(&mut self) {
        self.timestamp += 1;
    }

    pub fn init(&mut self, data: &[DataType]) -> PBId {
        if self.mem.len() < INIT_SIZE {
            self.mem.resize(INIT_SIZE, 0);
        }
        self.top = 0;
        self.timestamp = 0;
        self.len = data.len() as i32;
        self.height = 0;
        let mut sum = self.single_node_capacity();
        while sum < data.len() {
            sum <<= BP;
            self.height += 1;
        }
        self.init_recurse(0, self.height, data)
    }
}

pub struct PersistentDecArray {
    pub inner: PersistentBTree<2, 1>,
}

impl Default for PersistentDecArray {
    fn default() -> Self {
        Self {
            inner: PersistentBTree::new(),
        }
    }
}

impl PersistentDecArray {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self, data: &[DataType]) -> PBId {
        self.inner.init(data)
    }

    pub fn new_version(&mut self) {
        self.inner.new_version();
    }

    pub fn dec(&mut self, root: PBId, i: i32) -> (PBId, DataType) {
        crate::debug_assert_tiger!(0 <= i && i < self.inner.len);
        let mut stack: Vec<PBId> = vec![0; (self.inner.height + 1) as usize];
        stack[0] = root;
        for h in 0..self.inner.height {
            let chid_v = self.inner.chid(self.inner.height - h, i);
            stack[(h + 1) as usize] = self.inner.getc(stack[h as usize], chid_v);
        }
        let chid0i = self.inner.chid(0, i);
        let posi = self.inner.pos(i);
        let c_val = self.inner.getc(*stack.last().unwrap(), chid0i);
        let val: u32 = (c_val >> (posi << 1)) & ((1u32 << 2) - 1);
        if val == 0 {
            return (root, 0);
        }
        let nc: u32 = c_val ^ ((val ^ (val - 1)) << (posi << 1));
        if self.inner.getts(*stack.last().unwrap()) == self.inner.timestamp as u32 {
            let last = *stack.last().unwrap();
            *self.inner.getc_mut(last, chid0i) = nc;
            (root, val)
        } else {
            let n = self.inner.new_node_from(*stack.last().unwrap());
            *self.inner.getc_mut(n, chid0i) = nc;
            *stack.last_mut().unwrap() = n;
            let mut h: i32 = self.inner.height - 1;
            while h >= 0 {
                let chid_v = self.inner.chid(self.inner.height - h, i);
                if self.inner.getc(stack[h as usize], chid_v) != stack[(h + 1) as usize] {
                    if self.inner.getts(stack[h as usize]) == self.inner.timestamp as u32 {
                        let s = stack[h as usize];
                        let next = stack[(h + 1) as usize];
                        *self.inner.getc_mut(s, chid_v) = next;
                    } else {
                        stack[h as usize] = self.inner.new_node_from(stack[h as usize]);
                        let s = stack[h as usize];
                        let next = stack[(h + 1) as usize];
                        *self.inner.getc_mut(s, chid_v) = next;
                    }
                } else {
                    break;
                }
                h -= 1;
            }
            (stack[0], val)
        }
    }
}

pub struct PersistentBitSet {
    pub inner: PersistentBTree<2, 0>,
}

impl Default for PersistentBitSet {
    fn default() -> Self {
        Self {
            inner: PersistentBTree::new(),
        }
    }
}

impl PersistentBitSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self, data: &[DataType]) -> PBId {
        self.inner.init(data)
    }

    pub fn new_version(&mut self) {
        self.inner.new_version();
    }

    pub fn getpos(&self, root: PBId, i: i32) -> bool {
        crate::debug_assert_tiger!(0 <= i && i < self.inner.len);
        let mut cur = root;
        for h in 0..self.inner.height {
            let chid_v = self.inner.chid(self.inner.height - h, i);
            cur = self.inner.getc(cur, chid_v);
        }
        let chid0i = self.inner.chid(0, i);
        let posi = self.inner.pos(i);
        let c_val = self.inner.getc(cur, chid0i);
        let val: u32 = (c_val >> posi) & 1;
        val != 0
    }

    pub fn setpos(&mut self, root: PBId, i: i32) -> (PBId, bool) {
        crate::debug_assert_tiger!(0 <= i && i < self.inner.len);
        let mut stack: Vec<PBId> = vec![0; (self.inner.height + 1) as usize];
        stack[0] = root;
        for h in 0..self.inner.height {
            let chid_v = self.inner.chid(self.inner.height - h, i);
            stack[(h + 1) as usize] = self.inner.getc(stack[h as usize], chid_v);
        }
        let chid0i = self.inner.chid(0, i);
        let posi = self.inner.pos(i);
        let c_val = self.inner.getc(*stack.last().unwrap(), chid0i);
        let nc: u32 = c_val | (1u32 << posi);
        if c_val == nc {
            (root, true)
        } else {
            if self.inner.getts(*stack.last().unwrap()) == self.inner.timestamp as u32 {
                let last = *stack.last().unwrap();
                *self.inner.getc_mut(last, chid0i) = nc;
                (root, false)
            } else {
                let n = self.inner.new_node_from(*stack.last().unwrap());
                *self.inner.getc_mut(n, chid0i) = nc;
                *stack.last_mut().unwrap() = n;
                let mut h: i32 = self.inner.height - 1;
                while h >= 0 {
                    let chid_v = self.inner.chid(self.inner.height - h, i);
                    if self.inner.getc(stack[h as usize], chid_v) != stack[(h + 1) as usize] {
                        if self.inner.getts(stack[h as usize]) == self.inner.timestamp as u32 {
                            let s = stack[h as usize];
                            let next = stack[(h + 1) as usize];
                            *self.inner.getc_mut(s, chid_v) = next;
                        } else {
                            stack[h as usize] = self.inner.new_node_from(stack[h as usize]);
                            let s = stack[h as usize];
                            let next = stack[(h + 1) as usize];
                            *self.inner.getc_mut(s, chid_v) = next;
                        }
                    } else {
                        break;
                    }
                    h -= 1;
                }
                (stack[0], false)
            }
        }
    }
}
