use std::hash::Hash;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

#[derive(Copy, Clone)]
pub struct CompressedNodeSet {
    pub nodes: u8,
}

impl CompressedNodeSet {
    pub const fn new(nodes: u8) -> Self {
        Self { nodes }
    }

    pub const fn from_array<const N: usize>(nodes: [u8; N]) -> Self {
        let mut rv = Self::new(0);
        let mut i = 0;
        while i < N {
            rv.nodes |= 1 << nodes[i];
            i += 1;
        }
        rv
    }

    pub fn from_iter(nodes: impl IntoIterator<Item = u8>) -> Self {
        let mut rv = Self::new(0);
        for node in nodes {
            rv.nodes |= 1 << node;
        }
        rv
    }

    pub fn remove(&mut self, node: usize) {
        self.nodes &= !(1 << node);
    }

    pub fn insert(&mut self, node: usize) {
        self.nodes |= (1 << node);
    }

    pub const fn len(&self) -> u32 {
        self.nodes.count_ones()
    }

    pub const fn contains(&self, node: usize) -> bool {
        (self.nodes & (1 << node)) != 0
    }

    pub const fn bitand(&self, other: Self) -> Self {
        Self {
            nodes: self.nodes & other.nodes,
        }
    }

    pub const fn bitor(&self, other: Self) -> Self {
        Self {
            nodes: self.nodes | other.nodes,
        }
    }

    pub fn iter(&self) -> CompressetNodeSetIter {
        CompressetNodeSetIter {
            remaining_nodes: self.nodes,
        }
    }
}

pub struct CompressetNodeSetIter {
    remaining_nodes: u8,
}

impl Iterator for CompressetNodeSetIter {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_nodes == 0 {
            None
        } else {
            let index = self.remaining_nodes.trailing_zeros() as usize;

            // 2. Clear the lowest set bit
            self.remaining_nodes &= self.remaining_nodes - 1;

            Some(index)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.remaining_nodes.count_ones() as usize;
        (count, Some(count))
    }
}

impl IntoIterator for CompressedNodeSet {
    type Item = usize;
    type IntoIter = CompressetNodeSetIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CompressetNodeSetIter {
            remaining_nodes: self.nodes,
        }
    }
}

impl BitAnd for CompressedNodeSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            nodes: self.nodes & rhs.nodes,
        }
    }
}

impl BitAndAssign for CompressedNodeSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.nodes &= rhs.nodes;
    }
}

impl BitOr for CompressedNodeSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            nodes: self.nodes | rhs.nodes,
        }
    }
}

impl BitOrAssign for CompressedNodeSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.nodes |= rhs.nodes;
    }
}

impl Not for CompressedNodeSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { nodes: !self.nodes }
    }
}

impl Hash for CompressedNodeSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.nodes.hash(state);
    }
}
impl PartialEq for CompressedNodeSet {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl Eq for CompressedNodeSet {}

impl std::fmt::Debug for CompressedNodeSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CompressedNodeSet({:08b})", self.nodes)
    }
}
