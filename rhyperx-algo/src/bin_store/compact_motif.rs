use crate::util::const_operations::max_hyperedge_count;
use crate::util::const_operations::{binomial_coefficient, factorial};
use crate::util::permutations::BinPerm;
use crate::{bin_store::BinStore, iter_hyperedges};
use core::fmt::Display;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, RangeInclusive, Shl};
use duplicate::duplicate_item;
use rhyperx_macros::hoist_mod;

/// Max supported order.  Compile-time tables grow as ~2^N, so N is capped at 8
/// (tables reach ≈2.5 MB at N=8).
pub const MAX_SUPPORTED_ORDER: usize = 8;

/// Hyperedge motif encoded as a flat bit-vector.
///
/// Each bit position corresponds to one possible hyperedge over N nodes.
/// The mapping from (nodes, edge-size) to bit index is fixed at compile time
/// by iterating all combinations with `iter_hyperedges!`.
///
/// # Type parameters
/// * `TM`  — storage word for the edge bit-vector (u8 … u128).
/// * `N`   — number of nodes in the motif (≤ 8).
/// * `M`   — total number of possible hyperedges (Σ_{k=1..N} C(N,k)).
/// * `AM`  — number of `TM` words needed to store M bits.
/// * `P`   — factorial(N), i.e. number of node relabelings.
///
/// # Example
/// ```ignore
/// use rhyperx_algo::CompactMotif;
///
/// // A 3-node motif with only the {0,1} edge:
/// let mut m: CompactMotif!(3) = CompactMotif::zero();
/// m.add_edge_with_nodes(0b011);
/// assert_eq!(m.edge_count(), 1);
/// ```
///
/// The `CompactMotif!` macro expands to the full type:
/// ```ignore
/// type M3 = CompactMotif!(3);  // CompactMotif<u8, 3, 7, 1, 6>
/// ```
/// Use `compact_motif!(3)` to create an empty instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompactMotif<TM, const N: usize, const M: usize, const AM: usize, const P: usize> {
    pub(self) bits: BinStore<TM, AM>,
}

// ──────────────────────────────────────────────
// Compile-time tables & inherent methods
// (duplicated for every TM ∈ {u8, u16, u32, u64, u128})
// ──────────────────────────────────────────────
#[hoist_mod(attr(duplicate_item(tm_type; [u8]; [u16]; [u32]; [u64]; [u128];)))]
mod __ {

    // ── Compile-time tables ──────────────────────

    impl<const N: usize, const M: usize, const AM: usize, const P: usize>
        CompactMotif<tm_type, N, M, AM, P>
    {
        const _LIMIT_CHECK: () = assert!(
            N <= MAX_SUPPORTED_ORDER,
            "Compile Error: N must be ≤ MAX_SUPPORTED_ORDER",
        );

        /// Empty motif (no edges).
        pub const EMPTY: Self = Self {
            bits: BinStore::<tm_type, AM>::ZERO,
        };

        /// Motif with every possible edge set.
        const FULL: Self = Self {
            bits: BinStore::<tm_type, AM>::ONE.not(),
        };

        /// Map a 1-hot u8 node-set to an edge ID.
        const fn edge_id_from_bitset(bitset: BinStore<u8, 1>) -> usize {
            let idx = bitset.get_bit(0) as usize;
            if idx < M { Self::EDGE_MAP[idx] } else { M - 1 }
        }

        /// `ADJ[node]` = bitmask of edges incident to `node`.
        const ADJ: [Self; N] = const {
            let mut adj = [Self::EMPTY; N];
            iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
                let mut i = 0;
                while i < edge_size {
                    adj[edge[i]].bits.set_bit(edge_idx);
                    i = i + 1;
                }
            });
            adj
        };

        /// `NODE_MAP[edge_id]` = u8 bitset of nodes in that edge.
        const NODE_MAP: [u8; M] = const {
            let mut rv = [0u8; M];
            iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
                let mut i = 0;
                while i < edge_size {
                    rv[edge_idx] |= 1 << edge[i];
                    i = i + 1;
                }
            });
            rv
        };

        /// `EDGE_MAP[node_bitset]` = edge ID (inverse of `NODE_MAP`).
        const EDGE_MAP: [usize; M] = const {
            let mut rv = [0; M];
            iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
                let mut i = 0;
                let mut bitset = 0;
                while i < edge_size {
                    bitset |= 1 << edge[i];
                    i = i + 1;
                }
                rv[bitset] = edge_idx;
            });
            rv
        };

        /// `FULL_OVERLAPS[e]` = edges whose node-set is a subset of `e`'s nodes.
        const FULL_OVERLAPS: [Self; M] = const {
            let mut rv = [Self::FULL; M];
            iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
                let mut bitset = 0u8;
                let mut i = 0;
                while i < edge_size {
                    bitset |= 1 << edge[i];
                    i = i + 1;
                }
                bitset = !bitset;
                bitset &= (1 << N) - 1;

                while bitset != 0 {
                    let node = bitset.trailing_zeros() as usize;
                    bitset &= bitset - 1;
                    rv[edge_idx].bits.bitand_assign(&Self::ADJ[node].bits.not());
                }
                rv[edge_idx].bits.retain_lsb(M);
            });
            rv
        };

        /// `PART_OVERLAPS[e]` = edges sharing at least one node with `e`.
        const PART_OVERLAPS: [Self; M] = const {
            let adj = Self::ADJ;
            let mut rv = [Self::EMPTY; M];
            iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
                let mut i = 0;
                while i < edge_size {
                    rv[edge_idx].bits.bitor_assign(&adj[edge[i]].bits);
                    i = i + 1;
                }
            });
            rv
        };

        /// `INCLUSION_MAP[e]` = edges that fully contain `e`.
        const INCLUSION_MAP: [Self; M] = const {
            let mut rv = [Self::EMPTY; M];
            iter_hyperedges!(N, 1..=N, |_edge, _edge_size, edge_idx| {
                let mut full_overlaps = Self::FULL_OVERLAPS[edge_idx].bits;
                while !full_overlaps.is_empty() {
                    let inner = full_overlaps.pop();
                    rv[inner].bits.set_bit(edge_idx);
                }
            });
            rv
        };

        /// `EDGE_FILTER_BITMASK[k]` = bitmask selecting only edges of size `k` (1-indexed).
        /// Index 0 stays empty.
        pub const EDGE_FILTER_BITMASK: [Self; M] = const {
            let mut rv = [Self::EMPTY; M];

            let mut shift_offset = 0;
            let mut k = 1;

            while k <= N {
                let curr_count = binomial_coefficient(N, k);
                let mut edge_idx = 0;
                while edge_idx < curr_count {
                    rv[k].bits.set_bit(shift_offset + edge_idx);
                    edge_idx += 1;
                }
                shift_offset += curr_count;
                k += 1;
            }
            rv
        };

        /// `RELABELING_MAP[perm_id][e]` = edge ID after applying the `perm_id`-th node permutation.
        pub const RELABELING_MAP: [[usize; M]; P] = const {
            let node_map = Self::NODE_MAP;
            let edge_map = Self::EDGE_MAP;

            let mut relabeling_map = [[0usize; M]; P];

            let mut i = 0;
            while i < factorial(N) {
                let perm = BinPerm::from_usize(i).decode::<N>();
                let mut j = 0;

                while j < M {
                    let mut old_nodes = node_map[j];
                    let mut new_nodes = 0u8;

                    while old_nodes != 0 {
                        let old_node = old_nodes.trailing_zeros() as usize;
                        old_nodes &= old_nodes - 1;
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

    // ── Inherent methods ─────────────────────────

    impl<const N: usize, const M: usize, const AM: usize, const P: usize>
        CompactMotif<tm_type, N, M, AM, P>
    {
        /// Create a new motif from a `BinStore` bit-vector. This is a low-level constructor; use
        /// `compact_motif!(N)` to create an empty motif.
        pub const fn new(bits: BinStore<tm_type, AM>) -> Self {
            Self { bits }
        }

        /// Shift bits left by `rhs` positions
        pub const fn shl_assign(&mut self, rhs: usize) {
            self.bits.shift_left_assign(rhs);
        }

        /// Shift bits right by `rhs` positions
        pub const fn shr(self, rhs: usize) -> Self {
            Self {
                bits: self.bits.shift_right(rhs),
            }
        }

        /// Shift bits right by `rhs` positions
        pub const fn shr_assign(&mut self, rhs: usize) {
            self.bits.shift_right_assign(rhs);
        }

        /// Number of edges in this motif
        pub const fn edge_count(&self) -> usize {
            self.bits.count_ones()
        }

        /// Returns ture if the motif contains no edges
        pub const fn is_empty(&self) -> bool {
            self.bits.is_empty()
        }

        /// Set the bit for edge `edge_number`.
        pub const fn add_edge(&mut self, edge_number: usize) {
            self.bits.set_bit(edge_number);
        }

        /// Add an edge with the specified nodes
        /// Returns `false` if the node array is too large or contains a duplicate.
        pub const fn add_edge_with_nodes<const NN: usize>(&mut self, nodes: [u8; NN]) -> bool {
            if NN > N {
                return false;
            }
            let mut i = 0;
            let mut set = BinStore::<u8, 1>::ZERO;
            while i < NN {
                let node = nodes[i] as usize;
                if set.get_bit(node) {
                    return false;
                }
                set.set_bit(node);
                i += 1;
            }

            let edge_number = Self::edge_id_from_bitset(set);
            self.add_edge(edge_number);
            true
        }

        /// Add an edge with the specified nodes; no duplicate/overflow checks are performed.
        pub const fn add_edge_with_nodes_unchecked<const NN: usize>(&mut self, nodes: [u8; NN]) {
            let mut i = 0;
            let mut set = BinStore::<u8, 1>::ZERO;
            while i < NN {
                let node = nodes[i] as usize;
                set.set_bit(node);
                i += 1;
            }

            let edge_number = Self::edge_id_from_bitset(set);
            self.add_edge(edge_number);
        }

        /// Clear the bit for edge `edge_number`.
        pub const fn remove_edge(&mut self, edge_number: usize) -> bool {
            let rv = self.bits.get_bit(edge_number);
            self.bits.clear_bit(edge_number);
            rv
        }

        /// Remove an edge with the specified nodes
        /// Returns `false` if the node set is too large or contains a duplicate.
        pub const fn remove_edge_with_nodes(&mut self, nodes: &[u8]) -> Result<bool, ()> {
            if nodes.len() > N {
                return Err(());
            }
            let mut i = 0;
            let mut set = BinStore::<u8, 1>::ZERO;
            while i < nodes.len() {
                let node = nodes[i] as usize;
                if set.get_bit(node) {
                    return Err(());
                }
                set.set_bit(node);
                i += 1;
            }
            let edge_number = Self::edge_id_from_bitset(set);
            Ok(self.remove_edge(edge_number))
        }

        /// Remove an edge with the specified nodes; no duplicate/overflow checks are performed.
        pub fn remove_edge_with_nodes_unchecked(&mut self, nodes: &[u8]) -> bool {
            let mut i = 0;
            let mut set = BinStore::<u8, 1>::ZERO;
            while i < nodes.len() {
                let node = nodes[i] as usize;
                set.set_bit(node);
                i += 1;
            }
            let edge_number = Self::edge_id_from_bitset(set);
            self.remove_edge(edge_number)
        }

        /// Keep only edges of the given size.
        pub const fn filter_by_order(&mut self, order: usize) {
            self.bits
                .bitand_assign(&Self::EDGE_FILTER_BITMASK[order].bits);
        }

        /// Return a copy filtered to edges of the given size.
        pub const fn filtered_by_order(mut self, order: usize) -> Self {
            self.filter_by_order(order);
            self
        }

        /// Remove all edges of the given size.
        pub const fn remove_order(&mut self, order: usize) {
            let mut mask = Self::EDGE_FILTER_BITMASK[order].bits;
            mask.negate();
            mask.retain_lsb(M);
            self.bits.bitand_assign(&mask);
        }

        /// Return a copy with all edges of the given size removed.
        pub const fn without_order(mut self, order: usize) -> Self {
            self.remove_order(order);
            self
        }

        /// Number of edges of a given size (binomial coefficient C(N, order)).
        pub const fn max_edge_count(order: usize) -> usize {
            binomial_coefficient(N, order)
        }

        /// Total number of possible edges across all sizes.
        pub const fn max_edge_count_tot() -> usize {
            let mut i = 1;
            let mut count = 0;
            while i <= N {
                count += binomial_coefficient(N, i);
                i += 1;
            }
            count
        }

        /// Subset of edges whose nodes are fully contained in `edge_number`.
        pub const fn full_ovelaps(&self, edge_number: usize) -> Self {
            Self::new(self.bits.bitand(&Self::FULL_OVERLAPS[edge_number].bits))
        }

        /// Subset of edges that fully contain edge `e`.
        pub const fn inclusions(&self, e: usize) -> Self {
            Self::new(Self::INCLUSION_MAP[e].bits.bitand(&self.bits))
        }

        /// Check whether edge `edge_number` is present.
        pub const fn contains_edge(&self, edge_number: usize) -> bool {
            self.bits.get_bit(edge_number)
        }

        /// Subset of edges sharing at least one node with `edge_number`.
        pub const fn part_ovelaps(&self, edge_number: usize) -> Self {
            Self::new(self.bits.bitand(&Self::PART_OVERLAPS[edge_number].bits))
        }

        /// Subset of edges incident to `node`.
        pub const fn neighbors(&self, node: usize) -> Self {
            Self::new(Self::ADJ[node].bits.bitand(&self.bits))
        }

        /// Iterator over every possible motif over the full edge space.
        pub fn iter_all_combinations() -> CompactMotifCombinationsIterator<tm_type, N, M, AM, P> {
            <CompactMotifCombinationsIterator<tm_type, N, M, AM, P>>::new()
        }

        /// Iterate over edge IDs present in the motif.
        pub fn iter_edges(&self) -> CompactMotifEdgeIter<tm_type, AM> {
            CompactMotifEdgeIter {
                remaining_edges: self.bits,
                remaining_count: self.bits.count_ones(),
            }
        }

        /// Iterate over node indices 0..N.
        pub fn iter_nodes(&self) -> impl Iterator<Item = u8> {
            0u8..(N as u8)
        }

        /// BFS-based connectivity check.
        pub fn is_connected(&self) -> bool {
            if self.is_empty() {
                return false;
            }

            let mut covered_nodes = 0u8;
            for e in self.iter_edges() {
                covered_nodes |= Self::NODE_MAP[e];
            }
            let full_mask = (1 << N) - 1;
            if covered_nodes != full_mask {
                return false;
            }

            let first_edge = match self.iter_edges().next() {
                Some(e) => e,
                None => return false,
            };

            let mut visited = BinStore::<tm_type, AM>::ZERO;
            let mut queue = BinStore::<tm_type, AM>::ZERO;
            visited.set_bit(first_edge);
            queue.set_bit(first_edge);

            while !queue.is_empty() {
                let e = queue.trailing_zeros();
                queue.clear_bit(e);

                let mut neighbors = Self::PART_OVERLAPS[e].bits;
                neighbors.bitand_assign(&visited.not());
                visited.bitor_assign(&neighbors);
                queue.bitor_assign(&neighbors);
            }

            visited == self.bits
        }

        /// Utility function used as a filter in iterator operations
        fn is_connected_filter(motif: &Self) -> bool {
            motif.is_connected()
        }

        /// Enumerate every motif whose edges fall within `range` (by edge size).
        pub fn enum_motifs(
            range: RangeInclusive<usize>,
        ) -> CompactMotifCombinationsIterator<tm_type, N, M, AM, P> {
            Self::iter_all_combinations().with_range(range)
        }

        /// Enumerate connected motifs within `range`.
        pub fn enum_connected_motifs(range: RangeInclusive<usize>) -> impl Iterator<Item = Self> {
            Self::iter_all_combinations()
                .with_range(range)
                .filter(Self::is_connected_filter)
        }

        /// Apply a node permutation to the motif.
        pub fn relabeled(&self, perm: BinPerm) -> Self {
            let mut rv = Self::EMPTY;
            for e in self.iter_edges() {
                rv.add_edge(Self::RELABELING_MAP[perm.container][e]);
            }
            rv
        }

        /// Call `f` for every isomorphic relabeling of the motif.
        pub fn enum_isomorphism<F>(&self, mut f: F)
        where
            F: FnMut(Self),
        {
            for p in BinPerm::iter_all::<N>() {
                f(self.relabeled(p));
            }
        }

        /// Number of distinct isomorphism classes reachable from this motif.
        pub fn isomorphism_count(&self) -> usize {
            let mut set = std::collections::HashSet::new();
            self.enum_isomorphism(|iso| {
                set.insert(iso);
            });
            set.len()
        }

        /// Convert the motif to `Vec<Vec<usize>>` — each edge is a vector of node indices.
        pub fn to_vec(&self) -> Vec<Vec<usize>> {
            let mut edges = Vec::new();
            for e in self.iter_edges() {
                let mut nodes = Vec::with_capacity(N);
                let mut node_bits = Self::NODE_MAP[e];
                while node_bits != 0 {
                    let n = node_bits.trailing_zeros() as usize;
                    node_bits &= node_bits - 1;
                    nodes.push(n);
                }
                edges.push(nodes);
            }
            edges
        }
    }

    // ── Iterator impls ──────────────────────────

    impl<const AM: usize> Iterator for CompactMotifEdgeIter<tm_type, AM> {
        type Item = usize;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.remaining_count == 0 {
                None
            } else {
                let index = self.remaining_edges.trailing_zeros();
                self.remaining_edges.clear_bit(index);
                self.remaining_count -= 1;
                Some(index)
            }
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.remaining_count, Some(self.remaining_count))
        }
    }

    impl<const AM: usize> ExactSizeIterator for CompactMotifEdgeIter<tm_type, AM> {
        fn len(&self) -> usize {
            self.remaining_count
        }
    }

    // ── Trait impls ─────────────────────────────

    impl<const N: usize, const M: usize, const AM: usize, const P: usize> BitAnd
        for CompactMotif<tm_type, N, M, AM, P>
    {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            Self {
                bits: self.bits.bitand(&rhs.bits),
            }
        }
    }

    impl<const N: usize, const M: usize, const AM: usize, const P: usize> BitAnd
        for &CompactMotif<tm_type, N, M, AM, P>
    {
        type Output = CompactMotif<tm_type, N, M, AM, P>;

        fn bitand(self, rhs: Self) -> Self::Output {
            CompactMotif {
                bits: self.bits.bitand(&rhs.bits),
            }
        }
    }

    impl<const N: usize, const M: usize, const AM: usize, const P: usize> BitAndAssign
        for CompactMotif<tm_type, N, M, AM, P>
    {
        fn bitand_assign(&mut self, rhs: Self) {
            self.bits.bitand_assign(&rhs.bits);
        }
    }

    impl<const N: usize, const M: usize, const AM: usize, const P: usize> BitOr
        for CompactMotif<tm_type, N, M, AM, P>
    {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            Self {
                bits: self.bits.bitor(&rhs.bits),
            }
        }
    }

    impl<const N: usize, const M: usize, const AM: usize, const P: usize> BitOrAssign
        for CompactMotif<tm_type, N, M, AM, P>
    {
        fn bitor_assign(&mut self, rhs: Self) {
            self.bits.bitor_assign(&rhs.bits);
        }
    }

    impl<const N: usize, const M: usize, const AM: usize, const P: usize> Shl<usize>
        for CompactMotif<tm_type, N, M, AM, P>
    {
        type Output = Self;

        fn shl(self, rhs: usize) -> Self::Output {
            Self {
                bits: self.bits.shift_left(rhs),
            }
        }
    }

    impl<const N: usize, const M: usize, const AM: usize, const P: usize> Not
        for CompactMotif<tm_type, N, M, AM, P>
    {
        type Output = Self;

        fn not(self) -> Self::Output {
            let mut rv = Self { bits: self.bits };
            rv.bits.negate();
            rv.bits.retain_lsb(M);
            rv
        }
    }

    impl<const N: usize, const M: usize, const AM: usize, const P: usize> Display
        for CompactMotif<tm_type, N, M, AM, P>
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let mut edges = Vec::new();
            for e in self.iter_edges() {
                let mut nodes = Vec::with_capacity(N);
                let mut node_bits = Self::NODE_MAP[e];
                while node_bits != 0 {
                    let n = node_bits.trailing_zeros() as usize;
                    node_bits &= node_bits - 1;
                    nodes.push(n);
                }
                edges.push(nodes);
            }
            f.write_str(format!("{:?}", edges).as_str())
        }
    }

    // ── Combinations Iterator ───────────────────

    impl<const N: usize, const M: usize, const AM: usize, const P: usize>
        CompactMotifCombinationsIterator<tm_type, N, M, AM, P>
    {
        /// Iterate over all 2^M possible motifs.
        pub fn new() -> Self {
            Self {
                bits: BinStore::<tm_type, AM>::ZERO,
                shift: 0,
                target: BinStore::<tm_type, AM>::ZERO,
                finished: false,
            }
        }

        /// Restrict to motifs whose edges belong to the given size range.
        ///
        /// `range` is an inclusive range of edge sizes (1-indexed), e.g. `2..=3`.
        /// The iterator yields every subset of edges whose sizes fall within this range.
        pub fn with_range(self, range: RangeInclusive<usize>) -> Self {
            // Number of edges with size < range.start()
            let shift = if *range.start() <= 1 {
                0
            } else {
                max_hyperedge_count(N, 1, *range.start() - 1)
            };
            // ID of the last edge in the range (0-based).
            let last_edge_id = max_hyperedge_count(N, 1, *range.end().min(&N)).saturating_sub(1);
            // target = all bits in [0, last_edge_id] set
            let mut target = BinStore::<tm_type, AM>::ZERO;
            target.set_bit(last_edge_id);
            target.sub_assign_raw(1);
            target.set_bit(last_edge_id);

            Self {
                bits: BinStore::<tm_type, AM>::ZERO,
                shift,
                target,
                finished: false,
            }
        }
    }

    impl<const N: usize, const M: usize, const AM: usize, const P: usize> Iterator
        for CompactMotifCombinationsIterator<tm_type, N, M, AM, P>
    {
        type Item = CompactMotif<tm_type, N, M, AM, P>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.finished {
                return None;
            }
            // The motif is `self.bits` shifted into the edge-size range.
            let mut rv = <CompactMotif<tm_type, N, M, AM, P>>::new(self.bits);
            rv.bits.shift_left_assign(self.shift);

            // Advance to the next combination.
            if rv.bits == self.target {
                self.finished = true;
            } else {
                self.bits.add_assign_raw(1);
            }
            Some(rv)
        }
    }
}

// ── Iterator structs ────────────────────────────

/// Iterator over the edge IDs present in a motif.
///
/// Yields edge IDs one-by-one by consuming a copy of the edge bit-vector.
pub struct CompactMotifEdgeIter<TM, const AM: usize> {
    remaining_edges: BinStore<TM, AM>,
    remaining_count: usize,
}

/// Iterator over every subset of possible edges, optionally filtered by edge size.
///
/// Internally counts from 0 to the last combination using `BinStore` arithmetic,
/// shifting the result into the bit-range that corresponds to the desired edge sizes.
///
/// # Fields
/// * `bits`     — counter value for the current combination.
/// * `shift`    — number of bits to left-shift `bits` before yielding.
/// * `target`   — the last valid combination; iteration stops when `bits == target`.
/// * `finished` — flag set after the last combination has been yielded.
pub struct CompactMotifCombinationsIterator<
    TM,
    const N: usize,
    const M: usize,
    const AM: usize,
    const P: usize,
> {
    bits: BinStore<TM, AM>,
    shift: usize,
    target: BinStore<TM, AM>,
    finished: bool,
}

// ── Macros ──────────────────────────────────────

/// Expands to the `CompactMotif` type for N nodes (2..=8).
///
/// # Example
/// ```ignore
/// type M4 = CompactMotif!(4);  // CompactMotif<u16, 4, 15, 1, 24>
/// ```
#[macro_export]
macro_rules! CompactMotif {
    (2) => {
        $crate::bin_store::CompactMotif::<u8, 2, { (1 << 2) - 1 }, 1, 2>
    };
    (3) => {
        $crate::bin_store::CompactMotif::<
            u8,
            3,
            { (1 << 3) - 1 },
            { ((1usize << 3) - 1).div_ceil(u8::BITS as usize) },
            { $crate::util::const_operations::factorial(3) },
        >
    };
    (4) => {
        $crate::bin_store::CompactMotif::<
            u16,
            4,
            { (1 << 4) - 1 },
            { ((1usize << 4) - 1).div_ceil(u16::BITS as usize) },
            { $crate::util::const_operations::factorial(4) },
        >
    };
    (5) => {
        $crate::bin_store::CompactMotif::<
            u32,
            5,
            { (1 << 5) - 1 },
            { ((1usize << 5) - 1).div_ceil(u32::BITS as usize) },
            { $crate::util::const_operations::factorial(5) },
        >
    };
    (6) => {
        $crate::bin_store::CompactMotif::<
            u64,
            6,
            { (1 << 6) - 1 },
            { ((1usize << 6) - 1).div_ceil(u64::BITS as usize) },
            { $crate::util::const_operations::factorial(6) },
        >
    };
    (7) => {
        $crate::bin_store::CompactMotif::<
            u128,
            7,
            { (1 << 7) - 1 },
            { ((1usize << 7) - 1).div_ceil(u128::BITS as usize) },
            { $crate::util::const_operations::factorial(7) },
        >
    };
    (8) => {
        $crate::bin_store::CompactMotif::<
            u128,
            8,
            { (1 << 8) - 1 },
            { ((1usize << 8) - 1).div_ceil(u128::BITS as usize) },
            { $crate::util::const_operations::factorial(8) },
        >
    };

    ($n:expr) => {
        compile_error!("`CompactMotif!` only supports order sizes N in the range [2, 8].")
    };
}

/// Instantiate an empty `CompactMotif` for N nodes (2..=8).
///
/// # Example
/// ```ignore
/// let m = compact_motif!(3);  // CompactMotif::<u8, 3, 7, 1, 6>::EMPTY
/// ```
#[macro_export]
macro_rules! compact_motif {
    (2) => {
        <CompactMotif!(2)>::EMPTY
    };
    (3) => {
        <CompactMotif!(3)>::EMPTY
    };
    (4) => {
        <CompactMotif!(4)>::EMPTY
    };
    (5) => {
        <CompactMotif!(5)>::EMPTY
    };
    (6) => {
        <CompactMotif!(6)>::EMPTY
    };
    (7) => {
        <CompactMotif!(7)>::EMPTY
    };
    (8) => {
        <CompactMotif!(8)>::EMPTY
    };
    ($n:expr) => {
        compile_error!("`compact_motif!` only supports order sizes N in the range [2, 8].")
    };
}
