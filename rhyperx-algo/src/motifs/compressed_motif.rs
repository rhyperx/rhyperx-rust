use duplicate::duplicate_item;
use foldhash::fast::FixedState;
use hashbrown::HashSet;
use std::fmt::{Binary, Debug};
use std::hash::Hash;
use std::ops::{AddAssign, Deref, Not, RangeInclusive};
use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, IndexMut, Shl},
};

use crate::motifs::compressed_node_set::CompressedNodeSet;
use crate::motifs::fingerprint::{Fingerprint3, Fingerprint4, Fingerprint5};
use crate::util::const_operations::{binomial_coefficient, factorial, max_hyperedge_count};
use crate::util::permutations::BinPerm;
use num_traits::{AsPrimitive, One, PrimInt, Unsigned, Zero};

#[cfg(feature = "bindings")]
use pyo3::pyclass;

#[cfg(feature = "bindings")]
use pyo3_stub_gen::derive::gen_stub_pyclass;

macro_rules! impl_motif_configurator {
    ($ct:ty, $order:literal, $fingerprint: ty) => {
        impl CompactMotifConfigurator for CompactMotif<$order> {
            type ContainerType = $ct;
            type FingerprintType = $fingerprint;

            const MAX_EDGE_COUNT: usize = max_hyperedge_count($order, 1, $order);
            const SIZE: usize = $order;

            const CONTAINER_ZERO: Self::ContainerType = 0;
            const CONTAINER_ONE: Self::ContainerType = 1;
            const CONTAINER_FULL: Self::ContainerType =
                (1 << max_hyperedge_count($order, 1, $order)) - 1;

            const ZERO: Self = Self::new(Self::CONTAINER_ZERO);
            const ONE: Self = Self::new(Self::CONTAINER_ONE);
            const FULL: Self = Self::new(Self::CONTAINER_ONE);

            type AdjType = [Self; $order];
            type FullOverlapsType = [Self; Self::MAX_EDGE_COUNT];
            type PartOverlapsType = [Self; Self::MAX_EDGE_COUNT];
            type NodeMapType = [CompressedNodeSet; Self::MAX_EDGE_COUNT];
            type EdgeMapType = [u8; 1 << $order];
            type InclusionMapType = [Self; Self::MAX_EDGE_COUNT];

            type RelabelingMapIndex = [u8; Self::MAX_EDGE_COUNT];
            type RelabelingMap = [Self::RelabelingMapIndex; factorial($order)];
            type EdgeFilterBitmaskType = [Self; Self::MAX_EDGE_COUNT];

            const ADJ: Self::AdjType = const {
                let mut raw_adj = [0; $order];
                iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
                    let mut i = 0;
                    while i < edge_size {
                        raw_adj[edge[i]] |= 1 << edge_idx;
                        i = i + 1;
                    }
                });

                let mut adj = [Self::new(0); $order];
                let mut i = 0;
                while i < $order {
                    adj[i] = Self::new(raw_adj[i]);
                    i += 1;
                }
                adj
            };

            const NODE_MAP: Self::NodeMapType = const {
                let mut rv = [CompressedNodeSet::new(0); Self::MAX_EDGE_COUNT];
                iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
                    let mut i = 0;
                    let mut bitset = 0;
                    while i < edge_size {
                        bitset |= 1 << edge[i];
                        i = i + 1;
                    }
                    rv[edge_idx] = CompressedNodeSet::new(bitset);
                });
                rv
            };

            const EDGE_MAP: Self::EdgeMapType = const {
                let mut rv = [0; 1 << $order];
                iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
                    let mut i = 0;
                    let mut bitset = 0;
                    while i < edge_size {
                        bitset |= 1 << edge[i];
                        i = i + 1;
                    }
                    rv[bitset] = edge_idx as u8;
                });
                rv
            };

            const FULL_OVERLAPS: Self::FullOverlapsType = const {
                let mut rv_raw = [!0; Self::MAX_EDGE_COUNT];
                iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
                    let mut i = 0;
                    let mut bitset = 0u32;
                    while i < edge_size {
                        bitset |= 1 << edge[i];
                        i = i + 1;
                    }
                    bitset = !bitset & ((1 << $order) - 1);
                    while bitset != 0 {
                        let node = bitset.trailing_zeros() as usize;
                        bitset &= !(1 << node);
                        rv_raw[edge_idx] &= !Self::ADJ[node].container;
                    }
                });

                let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
                let mut i = 0;
                while i < Self::MAX_EDGE_COUNT {
                    rv[i] = Self::new(rv_raw[i] & ((1 << Self::MAX_EDGE_COUNT) - 1));
                    i += 1;
                }
                rv
            };

            const PART_OVERLAPS: Self::PartOverlapsType = const {
                let adj = Self::ADJ;
                let mut rv_raw = [0; Self::MAX_EDGE_COUNT];
                iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
                    let mut i = 0;
                    while i < edge_size {
                        rv_raw[edge_idx] |= adj[edge[i]].container;
                        i = i + 1;
                    }
                });

                let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
                let mut i = 0;
                while i < Self::MAX_EDGE_COUNT {
                    rv[i] = Self::new(rv_raw[i] & ((1 << Self::MAX_EDGE_COUNT) - 1));
                    i += 1;
                }
                rv
            };

            const INCLUSION_MAP: Self::InclusionMapType = const {
                let mut rv_raw = [0; Self::MAX_EDGE_COUNT];
                iter_hyperedges!($order, 1..=$order, |_edge, _edge_size, edge_idx| {
                    let mut iter = Self::FULL_OVERLAPS[edge_idx].container;
                    while iter != 0 {
                        let inner = iter.trailing_zeros() as usize;
                        iter &= !(1 << inner);
                        rv_raw[inner] |= 1 << edge_idx;
                    }
                });

                let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
                let mut i = 0;
                while i < Self::MAX_EDGE_COUNT {
                    rv[i] = Self::new(rv_raw[i]);
                    i += 1;
                }
                rv
            };

            const EDGE_FILTER_BITMASK: Self::EdgeFilterBitmaskType = const {
                let mut rv_raw = [0; Self::MAX_EDGE_COUNT];

                let mut shift_offset = 0;
                let mut i = 1;

                while i <= $order {
                    let curr_count = max_hyperedge_count($order, i, i);

                    rv_raw[i] = ((1 << curr_count) - 1) << shift_offset;

                    shift_offset += curr_count;
                    i += 1;
                }

                let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
                let mut i = 0;
                while i < Self::MAX_EDGE_COUNT {
                    rv[i] = Self::new(rv_raw[i]);
                    i += 1;
                }
                rv
            };

            const RELABELING_MAP: Self::RelabelingMap = const {
                let node_map = Self::NODE_MAP;
                let edge_map = Self::EDGE_MAP;

                let mut relabeling_map = [[0u8; Self::MAX_EDGE_COUNT]; factorial($order)];

                let mut i = 0;
                while i < factorial($order) {
                    let perm = BinPerm::from_usize(i).decode::<$order>();
                    let mut j = 0;

                    while j < Self::MAX_EDGE_COUNT {
                        let mut old_nodes = node_map[j].nodes;
                        let mut new_nodes = 0u8;

                        while old_nodes != 0 {
                            let old_node = old_nodes.trailing_zeros() as usize;
                            old_nodes &= !(1 << old_node);

                            let new_node = perm[old_node];
                            new_nodes |= 1 << new_node;
                        }

                        relabeling_map[i][j] = edge_map[new_nodes as usize];

                        j += 1;
                    }
                    i += 1;
                }

                relabeling_map
            };
        }

        impl CMAssociated for $fingerprint {
            type CMType = CompactMotif<$order>;
        }
    };
}

macro_rules! impl_motif {
    ($ct:ty, $order:literal) => {
        impl CompactMotif<$order> {
            pub const fn const_bitor(self, other: Self) -> Self {
                Self::new(self.container | other.container)
            }

            pub const fn const_bitand(self, other: Self) -> Self {
                Self::new(self.container & other.container)
            }

            pub const fn const_bitor_assign(&mut self, other: Self) {
                self.container |= other.container;
            }

            pub const fn const_bitand_assign(&mut self, other: Self) {
                self.container &= other.container;
            }

            pub const fn const_shl(self, rhs: usize) -> Self {
                Self::new(self.container << rhs)
            }

            pub const fn const_shl_assign(&mut self, rhs: usize) {
                self.container <<= rhs
            }

            pub const fn const_shr(self, rhs: usize) -> Self {
                Self::new(self.container >> rhs)
            }

            pub const fn const_shr_assign(&mut self, rhs: usize) {
                self.container >>= rhs
            }

            pub const fn const_not(mut self) -> Self {
                self.container = !self.container
                    & ((Self::CONTAINER_ONE << Self::MAX_EDGE_COUNT) - Self::CONTAINER_ONE);
                self
            }

            pub const fn const_edge_count(&self) -> u32 {
                self.container.count_ones()
            }

            pub const fn const_is_empty(&self) -> bool {
                self.container == 0
            }

            pub const fn const_add_edge(&mut self, edge_number: usize) {
                self.container |= Self::CONTAINER_ONE << edge_number;
            }

            pub const fn const_add_edge_with_nodes(&mut self, node_set: CompressedNodeSet) {
                let edge_number = Self::EDGE_MAP[node_set.nodes as usize];
                self.const_add_edge(edge_number as usize);
            }

            pub const fn const_remove_edge(&mut self, edge_number: usize) {
                self.container &= !(Self::CONTAINER_ONE << edge_number);
            }

            pub const fn const_filter_by_order(&mut self, order: usize) {
                self.container &= Self::EDGE_FILTER_BITMASK[order].container
            }

            pub const fn filtered_by_order(mut self, order: usize) -> Self {
                self.const_filter_by_order(order);
                self
            }

            pub const fn const_remove_order(&mut self, order: usize) {
                self.container &= !Self::EDGE_FILTER_BITMASK[order].container
            }

            pub const fn const_without_order(mut self, order: usize) -> Self {
                self.const_remove_order(order);
                self
            }

            pub const fn max_edge_count(order: usize) -> usize {
                binomial_coefficient($order, order)
            }

            pub const fn max_edge_count_tot() -> usize {
                let mut i = 0;
                let mut count = 0;
                while i <= $order {
                    count += binomial_coefficient($order, i);
                    i += 1;
                }
                count
            }
        }
    };
}

macro_rules! define_compact_motif {
    ($ct:ty, $order:literal, $fingerprint: ty) => {
        impl_motif_configurator!($ct, $order, $fingerprint);
        impl_motif!($ct, $order);
    };
}

pub trait CompactMotifConfigurator
where
    Self: Sized + Display,
    Self::ContainerType: Unsigned
        + PrimInt
        + Hash
        + BitAndAssign
        + BitOrAssign
        + AddAssign
        + AsPrimitive<usize>
        + Binary,
    Self::EdgeMapType: Index<usize, Output = u8>
        + IndexMut<usize, Output = u8>
        + IntoIterator<Item = u8>
        + Default,
    Self::AdjType: IntoIterator<Item = Self> + Index<usize, Output = Self>,
    Self::FullOverlapsType: IntoIterator<Item = Self> + Index<usize, Output = Self>,
    Self::PartOverlapsType: IntoIterator<Item = Self> + Index<usize, Output = Self>,
    Self::NodeMapType:
        IntoIterator<Item = CompressedNodeSet> + Index<usize, Output = CompressedNodeSet>,
    Self::InclusionMapType: IntoIterator<Item = Self> + Index<usize, Output = Self>,
    Self::RelabelingMapIndex: IntoIterator<Item = u8> + Index<usize, Output = u8> + Debug,
    Self::RelabelingMap: IntoIterator<Item = Self::RelabelingMapIndex>
        + Index<usize, Output = Self::RelabelingMapIndex>,
    Self::FingerprintType: Hash + Eq + PartialEq + From<Self> + Into<Self> + Debug + Clone,
    Self::EdgeFilterBitmaskType: IntoIterator<Item = Self> + Index<usize, Output = Self>,
{
    type ContainerType;
    type FingerprintType;

    const MAX_EDGE_COUNT: usize;
    const SIZE: usize;

    const CONTAINER_ZERO: Self::ContainerType;
    const CONTAINER_ONE: Self::ContainerType;
    const CONTAINER_FULL: Self::ContainerType;
    const ZERO: Self;
    const ONE: Self;
    const FULL: Self;

    type AdjType;
    type FullOverlapsType;
    type PartOverlapsType;
    type NodeMapType;
    type EdgeMapType;
    type InclusionMapType;

    type RelabelingMap;
    type RelabelingMapIndex;
    type EdgeFilterBitmaskType;

    const ADJ: Self::AdjType;
    const FULL_OVERLAPS: Self::FullOverlapsType;
    const PART_OVERLAPS: Self::PartOverlapsType;
    const NODE_MAP: Self::NodeMapType;
    const INCLUSION_MAP: Self::InclusionMapType;

    const EDGE_MAP: Self::EdgeMapType;
    const RELABELING_MAP: Self::RelabelingMap;
    const EDGE_FILTER_BITMASK: Self::EdgeFilterBitmaskType;
}

pub trait CMAssociated {
    type CMType;
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct CompactMotif<const N: usize>
where
    Self: CompactMotifConfigurator,
{
    pub container: <Self as CompactMotifConfigurator>::ContainerType,
}

impl<const N: usize> CompactMotif<N>
where
    Self: CompactMotifConfigurator,
{
    pub const fn new(container: <Self as CompactMotifConfigurator>::ContainerType) -> Self {
        Self { container }
    }

    /// Yields the number of each edge contained in the motif
    pub fn iter_edges(&self) -> CompactMotifEdgeIter<N> {
        self.into_iter()
    }

    /// Yields the set of nodes that each edge in the motifs has
    pub fn iter_nodes(&self) -> CompactMotifNodeIter<N> {
        CompactMotifNodeIter {
            remaining_edges: self.container,
        }
    }

    // pub fn edge_count(&self) -> u32 {
    //     self.container.count_ones()
    // }
    //
    // pub fn add_edge(&mut self, edge_number: usize) {
    //     self.container |= Self::CONTAINER_ONE << edge_number;
    // }
    //
    // pub fn remove_edge(&mut self, edge_number: usize) {
    //     self.container &= !(Self::CONTAINER_ONE << edge_number);
    // }
    //
    // pub fn part_ovelaps(&self, edge_number: usize) -> Self {
    //     self.bitand(Self::PART_OVERLAPS[edge_number])
    // }
    //
    // pub fn is_empty(&self) -> bool {
    //     self.container == 0
    // }

    pub fn full_ovelaps(&self, edge_number: usize) -> Self {
        *self & Self::FULL_OVERLAPS[edge_number]
    }

    pub fn inclusions(&self, e: usize) -> Self {
        Self::new(Self::INCLUSION_MAP[e].container & self.container)
    }

    pub const fn one() -> Self {
        <Self as CompactMotifConfigurator>::ONE
    }

    pub const fn zero() -> Self {
        <Self as CompactMotifConfigurator>::ZERO
    }

    pub const fn full() -> Self {
        <Self as CompactMotifConfigurator>::FULL
    }

    pub fn is_empty(&self) -> bool {
        self.container == Self::CONTAINER_ZERO
    }

    pub fn contains_edge(&self, edge_number: usize) -> bool {
        (self.container & (Self::CONTAINER_ONE << edge_number)) != Self::CONTAINER_ZERO
    }

    pub fn edge_count(&self) -> u32 {
        self.container.count_ones()
    }
    pub fn add_edge(&mut self, edge_number: usize) {
        self.container |= Self::CONTAINER_ONE << edge_number;
    }

    pub fn add_edge_with_nodes(&mut self, node_set: CompressedNodeSet) {
        let edge_number = Self::EDGE_MAP[node_set.nodes as usize];
        self.add_edge(edge_number as usize);
    }

    pub fn remove_edge(&mut self, edge_number: usize) {
        self.container &= !(Self::CONTAINER_ONE << edge_number);
    }

    pub fn remove_edge_with_nodes(&mut self, node_set: CompressedNodeSet) {
        let edge_number = Self::EDGE_MAP[node_set.nodes as usize];
        self.remove_edge(edge_number as usize);
    }

    pub fn part_ovelaps(&self, edge_number: usize) -> Self {
        *self & Self::PART_OVERLAPS[edge_number]
    }

    pub fn iter_all_combinations() -> CompactMotifCombinationsIterator<N> {
        CompactMotifCombinationsIterator::new()
    }

    pub fn fingerprint(&self) -> <Self as CompactMotifConfigurator>::FingerprintType
    where
        <Self as CompactMotifConfigurator>::FingerprintType: From<Self>,
    {
        (*self).into()
    }

    pub fn is_connected(&self) -> bool {
        if self.is_empty() {
            return false;
        }

        let mut covered_nodes = 0u8;
        for e in self {
            covered_nodes |= Self::NODE_MAP[e].nodes;
        }

        if covered_nodes != (1 << Self::SIZE) - 1 {
            return false;
        }

        let first_edge = self.iter_edges().next();
        if first_edge.is_none() {
            return false;
        }
        let first_edge = first_edge.unwrap();

        let mut visited_edges = Self::CONTAINER_ONE << first_edge;
        let mut queue = Self::CONTAINER_ONE << first_edge;

        while !queue.is_zero() {
            let e = queue.trailing_zeros() as usize;
            queue &= queue - Self::CONTAINER_ONE;

            let neighbors = self.part_ovelaps(e).container & !visited_edges;
            visited_edges |= neighbors;
            queue |= neighbors;
        }

        visited_edges == self.container
    }

    pub fn enum_motifs(range: RangeInclusive<usize>) -> CompactMotifCombinationsIterator<N> {
        Self::iter_all_combinations().with_range(range)
    }

    pub fn enum_connected_motifs(range: RangeInclusive<usize>) -> impl Iterator<Item = Self> {
        Self::iter_all_combinations()
            .with_range(range)
            .filter(|motif| motif.is_connected())
    }

    pub fn relabeled(&self, perm: BinPerm) -> Self {
        let mut rv = Self::ZERO;
        for e in self.iter_edges() {
            rv.add_edge(Self::RELABELING_MAP[perm.container][e] as usize)
        }
        rv
    }

    pub fn enum_isomorphism<F>(&self, mut f: F)
    where
        F: FnMut(Self),
    {
        for p in BinPerm::iter_all::<N>() {
            f(self.clone().relabeled(p));
        }
    }

    pub fn isomorphism_count(&self) -> usize {
        let mut set = HashSet::with_hasher(FixedState::default());
        self.enum_isomorphism(|iso| {
            set.insert(iso);
        });
        set.len()
    }

    pub fn to_vec(&self) -> Vec<Vec<usize>> {
        let mut edges = Vec::new();
        for e in self {
            let mut nodes = Vec::with_capacity(Self::SIZE);
            for n in Self::NODE_MAP[e] {
                nodes.push(n);
            }
            edges.push(nodes);
        }
        edges
    }

    pub fn neighbors(&self, node: usize) -> Self {
        Self::ADJ[node] & *self
    }
}

impl<const N: usize> BitAnd for CompactMotif<N>
where
    Self: CompactMotifConfigurator,
{
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            container: self.container & rhs.container,
        }
    }
}

impl<const N: usize> BitAndAssign for CompactMotif<N>
where
    Self: CompactMotifConfigurator,
{
    fn bitand_assign(&mut self, rhs: Self) {
        self.container &= rhs.container;
    }
}

impl<const N: usize> BitOr for CompactMotif<N>
where
    Self: CompactMotifConfigurator,
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            container: self.container | rhs.container,
        }
    }
}

impl<const N: usize> BitOrAssign for CompactMotif<N>
where
    Self: CompactMotifConfigurator,
{
    fn bitor_assign(&mut self, rhs: Self) {
        self.container |= rhs.container;
    }
}

impl<const N: usize> Shl<usize> for CompactMotif<N>
where
    Self: CompactMotifConfigurator,
{
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Self {
            container: self.container << rhs,
        }
    }
}

impl<const N: usize> Not for CompactMotif<N>
where
    Self: CompactMotifConfigurator,
{
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            container: !self.container
                & ((Self::CONTAINER_ONE << Self::MAX_EDGE_COUNT) - Self::CONTAINER_ONE),
        }
    }
}

impl<const N: usize> Display for CompactMotif<N>
where
    Self: CompactMotifConfigurator,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut edges = Vec::new();
        for e in self {
            let mut nodes = Vec::with_capacity(8);
            for n in Self::NODE_MAP[e] {
                nodes.push(n);
            }
            edges.push(nodes);
        }
        f.write_str(format!("{:?}", edges).as_str())
    }
}

impl<const N: usize> Debug for CompactMotif<N>
where
    Self: CompactMotifConfigurator,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut edges = Vec::new();
        for e in self {
            let mut nodes = Vec::with_capacity(8);
            for n in Self::NODE_MAP[e] {
                nodes.push(n);
            }
            edges.push(nodes);
        }
        f.write_str(format!("{:?}", edges).as_str())
    }
}

pub struct CompactMotifEdgeIter<const N: usize>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    remaining_edges: <CompactMotif<N> as CompactMotifConfigurator>::ContainerType,
}

impl<const N: usize> Iterator for CompactMotifEdgeIter<N>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // type T = ;
        if self.remaining_edges.is_zero() {
            None
        } else {
            // 1. Grab the index of the lowest set bit
            let index = self.remaining_edges.trailing_zeros() as usize;

            // 2. Clear the lowest set bit blazingly fast via bitwise AND
            self.remaining_edges &= self.remaining_edges
                - <CompactMotif<N> as CompactMotifConfigurator>::ContainerType::one();

            Some(index)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.remaining_edges.count_ones() as usize;
        (count, Some(count))
    }
}

impl<const N: usize> ExactSizeIterator for CompactMotifEdgeIter<N>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    fn len(&self) -> usize {
        self.remaining_edges.count_ones() as usize
    }
}

impl<const N: usize> IntoIterator for CompactMotif<N>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    type Item = usize;
    type IntoIter = CompactMotifEdgeIter<N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CompactMotifEdgeIter {
            remaining_edges: self.container,
        }
    }
}

impl<'a, const N: usize> IntoIterator for &'a CompactMotif<N>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    type Item = usize;
    type IntoIter = CompactMotifEdgeIter<N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CompactMotifEdgeIter {
            remaining_edges: self.container,
        }
    }
}

pub struct CompactMotifNodeIter<const N: usize>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    remaining_edges: <CompactMotif<N> as CompactMotifConfigurator>::ContainerType,
}

impl<const N: usize> Iterator for CompactMotifNodeIter<N>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    type Item = CompressedNodeSet;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // type T = ;
        if self.remaining_edges.is_zero() {
            None
        } else {
            // 1. Grab the index of the lowest set bit
            let index = self.remaining_edges.trailing_zeros() as usize;

            // 2. Clear the lowest set bit blazingly fast via bitwise AND
            self.remaining_edges &= self.remaining_edges - CompactMotif::<N>::CONTAINER_ONE;
            Some(CompactMotif::<N>::NODE_MAP[index])
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.remaining_edges.count_ones() as usize;
        (count, Some(count))
    }
}

impl<const N: usize> ExactSizeIterator for CompactMotifNodeIter<N>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    fn len(&self) -> usize {
        self.remaining_edges.count_ones() as usize
    }
}

pub struct CompactMotifCombinationsIterator<const N: usize>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    iteration_count: <CompactMotif<N> as CompactMotifConfigurator>::ContainerType,
    max_iterations: usize,
    base: usize,
}

impl<const N: usize> CompactMotifCombinationsIterator<N>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    pub fn new() -> Self {
        Self {
            iteration_count: CompactMotif::<N>::CONTAINER_ZERO,
            max_iterations: (CompactMotif::<N>::CONTAINER_ONE << CompactMotif::<N>::MAX_EDGE_COUNT)
                .as_(),
            base: 0,
        }
    }
    pub fn with_range(self, range: RangeInclusive<usize>) -> Self {
        let previous_edges_count = max_hyperedge_count(N, 1, (*range.start() - 1).min(N));

        // println!(
        //     "hx count {}",
        //     max_hyperedge_count(N, (*range.start()).max(1), *range.end() + 1),
        // );

        let base = if *range.start() == 0 {
            0
        } else {
            previous_edges_count
        };
        Self {
            iteration_count: CompactMotif::<N>::CONTAINER_ZERO,
            max_iterations: CompactMotif::<N>::CONTAINER_ONE.as_()
                << max_hyperedge_count(N, (*range.start()).max(1), *range.end() + 1),
            base,
        }
    }
}

impl<const N: usize> Iterator for CompactMotifCombinationsIterator<N>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    type Item = CompactMotif<N>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // println!(
        //     "iteration_count {:032b}, max_iterations {:032b}, base {}",
        //     self.iteration_count.as_(),
        //     self.max_iterations,
        //     self.base,
        // );
        //
        if self.iteration_count.as_() >= self.max_iterations {
            None
        } else {
            let rv = Some(CompactMotif::new(self.iteration_count << self.base));
            self.iteration_count += Self::Item::CONTAINER_ONE;
            rv
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<const N: usize> ExactSizeIterator for CompactMotifCombinationsIterator<N>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    fn len(&self) -> usize {
        let remaining = self.max_iterations - self.iteration_count.as_();
        // println!(
        //     "iteration_count {:032b}, max_iterations {:032b}, base {:032b}",
        //     self.iteration_count.as_(),
        //     self.max_iterations,
        //     self.base,
        // );
        remaining
    }
}

define_compact_motif!(u8, 3, Fingerprint3);
define_compact_motif!(u16, 4, Fingerprint4);
define_compact_motif!(u32, 5, Fingerprint5);

#[duplicate_item(motif_name      size; [CompactMotif3] [3]; [CompactMotif4] [4];)]
#[derive(Clone)]
#[cfg_attr(
    feature = "bindings",
    gen_stub_pyclass(module = "rust_core._core.motifs.types"),
    pyclass(from_py_object, str, frozen, eq, hash)
)]
pub struct motif_name {
    inner: CompactMotif<size>,
}

#[duplicate_item(motif_name      size; [CompactMotif3] [3]; [CompactMotif4] [4];)]
#[cfg_attr(
    feature = "bindings",
    gen_stub_pymethods(module = "rust_core._core.motifs.types"),
    pyo3::pymethods
)]
impl motif_name {
    pub fn into_edges(&self) -> Vec<Vec<usize>> {
        self.inner.to_vec()
    }
}

#[duplicate_item(motif_name      size; [CompactMotif3] [3]; [CompactMotif4] [4];)]
impl Deref for motif_name {
    type Target = CompactMotif<size>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[duplicate_item(motif_name      size; [CompactMotif3] [3]; [CompactMotif4] [4];)]
impl From<CompactMotif<size>> for motif_name {
    fn from(value: CompactMotif<size>) -> Self {
        Self { inner: value }
    }
}

#[duplicate_item(motif_name      size; [CompactMotif3] [3]; [CompactMotif4] [4];)]
impl Into<CompactMotif<size>> for motif_name {
    fn into(self) -> CompactMotif<size> {
        self.inner
    }
}

#[duplicate_item(motif_name      size; [CompactMotif3] [3]; [CompactMotif4] [4];)]
impl Display for motif_name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <CompactMotif<size> as Display>::fmt(self, f)
    }
}

#[duplicate_item(motif_name      size; [CompactMotif3] [3]; [CompactMotif4] [4];)]
impl PartialEq for motif_name {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[duplicate_item(motif_name      size; [CompactMotif3] [3]; [CompactMotif4] [4];)]
impl Eq for motif_name {}

#[duplicate_item(motif_name      size; [CompactMotif3] [3]; [CompactMotif4] [4];)]
impl Hash for motif_name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}
