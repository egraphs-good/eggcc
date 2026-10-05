//! Persistent (versioned) arrays of small counters and bits, backed by a
//! copy-on-write B-tree. The statewalk DP keeps one version per DP state.
//!
//! Nodes are stored in one flat `Vec<u32>`: a version stamp followed by `B`
//! child slots. Leaves pack several values per slot. Updating a version
//! created in an earlier `new_version` copies the path from the leaf to the
//! root; updates within the current version are done in place.

/// Index of a node in the arena; also the handle of a version (its root).
pub type Id = u32;

const GROWTH_FACTOR: usize = 4;

/// `BP`: log2 of the branching factor. `S`: log2 of the bits per value.
pub struct PersistentTree<const BP: usize, const S: usize> {
    /// First free index in `mem`.
    top: usize,
    len: usize,
    height: usize,
    mem: Vec<u32>,
    version: u32,
}

impl<const BP: usize, const S: usize> Default for PersistentTree<BP, S> {
    fn default() -> Self {
        Self {
            top: 0,
            len: 0,
            height: 0,
            mem: Vec::new(),
            version: 0,
        }
    }
}

impl<const BP: usize, const S: usize> PersistentTree<BP, S> {
    const BRANCHING: usize = 1 << BP;
    /// log2 of the values per leaf slot.
    const SLOT_VALUES_P: usize = 5 - S;
    const SLOT_VALUES: usize = 32 >> S;
    const LEAF_VALUES: usize = Self::SLOT_VALUES << BP;

    fn stamp(&self, node: Id) -> u32 {
        self.mem[node as usize]
    }

    fn child(&self, node: Id, i: usize) -> u32 {
        self.mem[node as usize + 1 + i]
    }

    fn child_mut(&mut self, node: Id, i: usize) -> &mut u32 {
        &mut self.mem[node as usize + 1 + i]
    }

    fn alloc_node(&mut self) -> Id {
        if self.top + 1 + Self::BRANCHING > self.mem.len() {
            let new_len = (self.mem.len() * GROWTH_FACTOR).max(self.top + 1 + Self::BRANCHING);
            self.mem.resize(new_len, 0);
        }
        let node = self.top as Id;
        self.top += 1 + Self::BRANCHING;
        self.mem[node as usize] = self.version;
        node
    }

    fn new_node(&mut self) -> Id {
        let node = self.alloc_node();
        for i in 0..Self::BRANCHING {
            *self.child_mut(node, i) = 0;
        }
        node
    }

    fn copy_node(&mut self, from: Id) -> Id {
        let node = self.alloc_node();
        let src = from as usize + 1;
        self.mem
            .copy_within(src..src + Self::BRANCHING, node as usize + 1);
        node
    }

    /// Which child slot of a node at height `h` leads to value `i`.
    fn slot(h: usize, i: usize) -> usize {
        (i >> (h * BP + Self::SLOT_VALUES_P)) & (Self::BRANCHING - 1)
    }

    /// Bit offset of value `i` inside its leaf slot.
    fn shift(i: usize) -> u32 {
        ((i & (Self::SLOT_VALUES - 1)) << S) as u32
    }

    fn build(&mut self, first: usize, h: usize, data: &[u32]) -> Id {
        let node = self.new_node();
        if h == 0 {
            for i in 0..Self::LEAF_VALUES.min(data.len().saturating_sub(first)) {
                *self.child_mut(node, Self::slot(0, i)) |= data[first + i] << Self::shift(i);
            }
        } else {
            for s in 0..Self::BRANCHING {
                let child_first = first + (s << (h * BP + Self::SLOT_VALUES_P));
                let child = if child_first >= data.len() {
                    u32::MAX
                } else {
                    self.build(child_first, h - 1, data)
                };
                *self.child_mut(node, s) = child;
            }
        }
        node
    }

    /// Start a new version: later updates copy instead of writing in place.
    pub fn new_version(&mut self) {
        self.version += 1;
    }

    /// Reset the arena and build the initial version holding `data`.
    pub fn init(&mut self, data: &[u32]) -> Id {
        self.top = 0;
        self.version = 0;
        self.len = data.len();
        self.height = 0;
        let mut capacity = Self::LEAF_VALUES;
        while capacity < data.len() {
            capacity <<= BP;
            self.height += 1;
        }
        // Enough for the initial tree; grows as versions are added.
        let nodes = 2 * (data.len() / Self::LEAF_VALUES + 2) * (self.height + 1);
        let needed = nodes * (1 + Self::BRANCHING);
        if self.mem.len() < needed {
            self.mem.resize(needed, 0);
        }
        self.build(0, self.height, data)
    }

    /// Nodes from the root of `version` down to the leaf holding value `i`.
    fn path(&self, version: Id, i: usize) -> Vec<Id> {
        let mut path = Vec::with_capacity(self.height + 1);
        path.push(version);
        for h in 0..self.height {
            let next = self.child(path[h], Self::slot(self.height - h, i));
            path.push(next);
        }
        path
    }

    fn leaf_slot(&self, leaf: Id, i: usize) -> u32 {
        self.child(leaf, Self::slot(0, i))
    }

    /// Value `i` in `version`.
    pub fn get(&self, version: Id, i: usize) -> u32 {
        debug_assert!(i < self.len);
        let leaf = *self.path(version, i).last().unwrap();
        (self.leaf_slot(leaf, i) >> Self::shift(i)) & ((1 << (1 << S)) - 1)
    }

    /// Replace the leaf slot holding `i` in `version`, copying any node that
    /// belongs to an older version. Returns the (possibly new) root.
    fn set_slot(&mut self, version: Id, i: usize, slot_value: u32) -> Id {
        let mut path = self.path(version, i);
        let leaf = *path.last().unwrap();
        let slot = Self::slot(0, i);
        if self.stamp(leaf) == self.version {
            *self.child_mut(leaf, slot) = slot_value;
            return version;
        }
        let new_leaf = self.copy_node(leaf);
        *self.child_mut(new_leaf, slot) = slot_value;
        *path.last_mut().unwrap() = new_leaf;
        for h in (0..self.height).rev() {
            let slot = Self::slot(self.height - h, i);
            if self.child(path[h], slot) == path[h + 1] {
                break;
            }
            if self.stamp(path[h]) != self.version {
                path[h] = self.copy_node(path[h]);
            }
            *self.child_mut(path[h], slot) = path[h + 1];
        }
        path[0]
    }
}

/// Persistent array of 2-bit counters that only ever decrease.
#[derive(Default)]
pub struct PersistentCounters {
    tree: PersistentTree<2, 1>,
}

impl PersistentCounters {
    pub fn init(&mut self, data: &[u32]) -> Id {
        self.tree.init(data)
    }

    pub fn new_version(&mut self) {
        self.tree.new_version();
    }

    /// Decrement counter `i` (saturating at zero). Returns the new version and
    /// the counter's value *before* the decrement.
    pub fn decrement(&mut self, version: Id, i: usize) -> (Id, u32) {
        debug_assert!(i < self.tree.len);
        let leaf = *self.tree.path(version, i).last().unwrap();
        let slot_value = self.tree.leaf_slot(leaf, i);
        let shift = PersistentTree::<2, 1>::shift(i);
        let value = (slot_value >> shift) & 0b11;
        if value == 0 {
            return (version, 0);
        }
        let new_slot = slot_value ^ ((value ^ (value - 1)) << shift);
        (self.tree.set_slot(version, i, new_slot), value)
    }
}

/// Persistent bit set.
#[derive(Default)]
pub struct PersistentBitSet {
    tree: PersistentTree<2, 0>,
}

impl PersistentBitSet {
    pub fn init(&mut self, data: &[u32]) -> Id {
        self.tree.init(data)
    }

    pub fn new_version(&mut self) {
        self.tree.new_version();
    }

    pub fn contains(&self, version: Id, i: usize) -> bool {
        self.tree.get(version, i) != 0
    }

    /// Set bit `i`. Returns the new version and whether the bit was already set.
    pub fn insert(&mut self, version: Id, i: usize) -> (Id, bool) {
        debug_assert!(i < self.tree.len);
        let leaf = *self.tree.path(version, i).last().unwrap();
        let slot_value = self.tree.leaf_slot(leaf, i);
        let new_slot = slot_value | (1 << PersistentTree::<2, 0>::shift(i));
        if new_slot == slot_value {
            return (version, true);
        }
        (self.tree.set_slot(version, i, new_slot), false)
    }
}
