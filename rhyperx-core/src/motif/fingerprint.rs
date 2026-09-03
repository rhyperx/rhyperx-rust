use crate::CompactMotif;
use crate::collections::BinStore;
use hashbrown::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::Hasher;

type CompactMotif3 = CompactMotif!(3);
type CompactMotif4 = CompactMotif!(4);
type CompactMotif5 = CompactMotif!(5);

pub trait Fingerprint: Sized + Hash + Eq + PartialEq + Copy + Clone {
    type MotifType;

    fn get_canonical_rep(&self) -> Self::MotifType;
}

pub trait Fingerprintable {
    type FingerprintType: Fingerprint;

    fn fingerprint(&self) -> Self::FingerprintType;
}

/// Fingerprint for 3-node hypergraphs.
///
/// Encodes the number of 2-edges and 3-edges in a single byte:
/// ```text
/// bits [7..4]: 3-edge count (0-2)
/// bits [3..0]: 2-edge count (0-3)
/// ```
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Fingerprint3 {
    edge_counts: u8,
}

impl Fingerprint3 {
    pub const SIZE: usize = 3;
    pub const MAX_EDGE_COUNT: usize = 2 << Self::SIZE;

    pub fn get_canonical_rep(&self) -> CompactMotif3 {
        let count_2 = self.edge_counts & ((1 << 4) - 1);
        let count_3 = (self.edge_counts >> 4) & ((1 << 4) - 1);
        let mut rv = CompactMotif3::EMPTY;

        for i in 0..count_2 {
            let nodes = {
                let mut rv = [i, (i + 1) % 3];
                rv.sort_unstable();
                rv
            };

            rv.add_edge_with_nodes(nodes);
        }
        if count_3 != 0 {
            rv.add_edge_with_nodes([0, 1, 2]);
        }
        rv.into()
    }

    pub fn enum_all(min_hx_size: usize, max_hx_size: usize) -> HashSet<Self> {
        let mut rv = HashSet::with_capacity(6);
        for motif in CompactMotif3::enum_connected_motifs(min_hx_size..=max_hx_size) {
            rv.insert(motif.fingerprint());
        }
        rv
    }
}

impl Debug for Fingerprint3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let order2 = (self.edge_counts >> 0) & ((1 << 4) - 1);
        let order3 = (self.edge_counts >> 4) & ((1 << 4) - 1);
        write!(f, "{:?}", [order2, order3])
    }
}

impl From<CompactMotif3> for Fingerprint3 {
    fn from(cm: CompactMotif3) -> Self {
        let mut edge_counts = 0u8;
        for nodes in cm.iter_edges() {
            edge_counts += 1 << (4 * (nodes.count() as usize - 2));
        }

        Fingerprint3 { edge_counts }
    }
}

impl Into<CompactMotif3> for Fingerprint3 {
    fn into(self) -> CompactMotif3 {
        self.get_canonical_rep()
    }
}

impl Fingerprint for Fingerprint3 {
    type MotifType = CompactMotif3;

    fn get_canonical_rep(&self) -> Self::MotifType {
        self.get_canonical_rep()
    }
}

impl Fingerprintable for CompactMotif3 {
    type FingerprintType = Fingerprint3;

    fn fingerprint(&self) -> Self::FingerprintType {
        Fingerprint3::from(*self)
    }
}

/// Fingerprint for 4-node hypergraphs.
///
/// ## Order map
/// For each node, a histogram of the sizes of the edges it participates in.
/// Sorted by node degree, then lexicographically by histogram.
/// Each entry is a u8 encoding:
/// ```text
/// bits [1..0]: 2-edge count (0-3)
/// bits [3..2]: 3-edge count (0-2)
/// bits [5..4]: 4-edge count (0-1)
/// ```
///
/// ## Inclusions
/// For each edge, the number of edges that fully contain it.
/// Each edge is stored in 3 bits (max 3 containing edges per edge, using 2 bits),
/// packed into a u32. The first `count_4` 3-bit slots are for 4-edges,
/// followed by slots for 3-edges.
#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct Fingerprint4 {
    order_map: [u8; 4],
    inclusions: u32,
}

/// Build a `[u8; 3]` array containing all nodes 0..3 except `removed`.
fn exclude_4(removed: u8) -> [u8; 3] {
    match removed {
        0 => [1, 2, 3],
        1 => [0, 2, 3],
        2 => [0, 1, 3],
        3 => [0, 1, 2],
        _ => unreachable!(),
    }
}

impl Fingerprint4 {
    const SIZE: usize = 4;

    pub fn get_canonical_rep(&self) -> CompactMotif4 {
        let mut rv = CompactMotif4::EMPTY;

        let mut out_2 = [(0u8, 0u8); Self::SIZE];
        for i in 0..Self::SIZE {
            out_2[i] = (i as u8, self.order_map[i] & 3);
        }

        loop {
            out_2.sort_unstable_by(|a, b| b.1.cmp(&a.1));

            let (current_node, degree) = out_2[0];
            if degree == 0 {
                break;
            }

            out_2[0].1 = 0;

            let mut to_remove = degree;
            for j in 1..Self::SIZE {
                if to_remove > 0 && out_2[j].1 > 0 {
                    rv.add_edge_with_nodes([current_node, out_2[j].0]);
                    out_2[j].1 -= 1;
                    to_remove -= 1;
                }
            }
        }

        let count_4 = (self.order_map[0] >> 4) & 3;

        let total_3_deg: usize = self
            .order_map
            .iter()
            .map(|&x| ((x >> 2) & 3) as usize)
            .sum();
        let count_3 = total_3_deg / 3;

        let mut check_3_edges = || match count_3 {
            0 => {}
            1 => {
                let expected = ((self.inclusions >> (3 * count_4)) & ((1 << 3) - 1)) as usize - 1;
                for i in 0..Self::SIZE {
                    if rv.neighbors(i).edge_count() == expected {
                        rv.add_edge_with_nodes(exclude_4(i as u8));
                        break;
                    }
                }
            }
            2 => {
                for i in 0..Self::SIZE {
                    for j in (i + 1)..Self::SIZE {
                        let incl_0 = (rv.neighbors(i) & rv.neighbors(j)).edge_count();
                        let incl_1 = (rv.neighbors(i) | rv.neighbors(j)).edge_count() - incl_0;
                        let incl_2 = rv.edge_count() - incl_0 - incl_1;

                        let expected_incl_0 =
                            ((self.inclusions >> (3 * count_4)) & ((1 << 3) - 1)) as usize - 2;
                        let expected_incl_1 =
                            ((self.inclusions >> (3 * (1 + count_4))) & ((1 << 3) - 1)) as usize;
                        let expected_incl_2 =
                            ((self.inclusions >> (3 * (2 + count_4))) & ((1 << 3) - 1)) as usize;

                        if incl_0 == expected_incl_0
                            && incl_1 == expected_incl_1
                            && incl_2 == expected_incl_2
                        {
                            rv.add_edge_with_nodes(exclude_4(i as u8));
                            rv.add_edge_with_nodes(exclude_4(j as u8));
                            return;
                        }
                    }
                }
            }
            3 => {
                for i in 0..Self::SIZE {
                    for j in 0..Self::SIZE {
                        if i == j {
                            continue;
                        }

                        let incl_2 = rv.neighbors(i).edge_count();
                        let incl_1 = rv.edge_count() - incl_2;

                        let expected_incl_1 =
                            ((self.inclusions >> (3 * (1 + count_4))) & ((1 << 3) - 1)) as usize;
                        let expected_incl_2 =
                            ((self.inclusions >> (3 * (2 + count_4))) & ((1 << 3) - 1)) as usize;

                        if incl_1 == expected_incl_1 && incl_2 == expected_incl_2 {
                            let other_nodes: Vec<u8> = [0, 1, 2, 3]
                                .into_iter()
                                .filter(|e| *e != i as u8 && *e != j as u8)
                                .collect();
                            let (a, b) = (other_nodes[0], other_nodes[1]);

                            rv.add_edge_with_nodes(exclude_4(a));
                            rv.add_edge_with_nodes(exclude_4(b));
                            rv.add_edge_with_nodes(exclude_4(j as u8));

                            return;
                        }
                    }
                }
            }
            4 => {
                for i in 0..Self::SIZE {
                    rv.add_edge_with_nodes(exclude_4(i as u8));
                }
            }
            _ => {}
        };
        check_3_edges();

        if count_4 == 1 {
            rv.add_edge_with_nodes([0, 1, 2, 3]);
        }

        rv
    }

    pub fn enum_all(min_hx_size: usize, max_hx_size: usize) -> HashSet<Self> {
        let mut rv = HashSet::with_capacity(12);
        for motif in CompactMotif4::enum_connected_motifs(min_hx_size..=max_hx_size) {
            rv.insert(motif.fingerprint());
        }
        rv
    }
}

impl Debug for Fingerprint4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rv = String::new();
        rv += format!("Order map: \n").as_str();
        for i in 0..Self::SIZE {
            let order2 = (self.order_map[i] >> 0) & ((1 << 2) - 1);
            let order3 = (self.order_map[i] >> 2) & ((1 << 2) - 1);
            let order4 = (self.order_map[i] >> 4) & ((1 << 2) - 1);
            rv += format!("\t {:?}\n", [order2, order3, order4]).as_str();
        }
        rv += format!("Inclusions: {:?}\n", self.inclusions).as_str();
        f.write_str(rv.as_str())
    }
}

impl From<CompactMotif4> for Fingerprint4 {
    fn from(cm: CompactMotif4) -> Self {
        let mut order_map = [0u8; Self::SIZE];
        for e in cm.iter_edges() {
            let size = e.count() as usize;
            let mut bits = e;
            while !bits.is_empty() {
                let n = bits.trailing_zeros();
                bits.clear_bit(n);
                order_map[n as usize] += 1 << (2 * (size - 2));
            }
        }
        order_map.sort_unstable();

        let mut inclusions = 0u32;
        for e in cm.iter_edges_ids() {
            let inclusions_count = cm.inclusions(e).edge_count() - 1;
            inclusions += 1 << (3 * inclusions_count as u32);
        }

        Fingerprint4 {
            order_map,
            inclusions,
        }
    }
}

impl Into<CompactMotif4> for Fingerprint4 {
    fn into(self) -> CompactMotif4 {
        self.get_canonical_rep()
    }
}

impl Fingerprint for Fingerprint4 {
    type MotifType = CompactMotif4;

    fn get_canonical_rep(&self) -> Self::MotifType {
        self.get_canonical_rep()
    }
}

impl Fingerprintable for CompactMotif4 {
    type FingerprintType = Fingerprint4;

    fn fingerprint(&self) -> Self::FingerprintType {
        Fingerprint4::from(*self)
    }
}

/// Fingerprint for 5-node hypergraphs.
///
/// ## Order map
/// For each node, a histogram of edge sizes it participates in.
/// Sorted by node degree, then lexicographically by histogram.
/// Each entry is a u16 encoding:
/// ```text
/// bits [2..0]: 2-edge count
/// bits [5..3]: 3-edge count
/// bits [8..6]: 4-edge count
/// ```
///
/// ## Edge connection map
/// For each edge of a given order, a packed descriptor of how it connects
/// to other edges. The descriptor records counts of edges grouped by
/// overlap size and group ID (based on which subset of the edge's nodes
/// they share). The groups are sorted to canonicalize the descriptor.
///
/// The map has two components:
/// - `edge_connection_map.0`: entries for order-2 edges (`u16` each).
/// - `edge_connection_map.1`: entries for order-3 edges (`u32` each).
///
/// ## Reconstruction
/// Not currently implemented (original code uses `todo!()`).
#[derive(Copy, Clone)]
pub struct Fingerprint5 {
    /// For each node, a histogram of the sizes of the edges it participates in.
    /// Sorted by node degree, then lexicographically by histogram.
    order_map: [u16; 5],

    /// For each edge it stores information about its connectivity.
    /// cover_tree[overlapping node set size][group id of the overlapping node set]
    ///     [overlapping edge size] = number of edges with this configuration
    edge_connection_map: (
        [u16; CompactMotif5::max_edge_count(2)],
        [u32; CompactMotif5::max_edge_count(3)],
    ),

    edge_connection_map_sizes: (usize, usize),
}

/// Extract the raw integer value of a `BinStore<u8, 1>` bitset.
fn binstore_u8_value(bs: BinStore<u8, 1>) -> usize {
    let mut result = 0usize;
    let mut bits = bs;
    while !bits.is_empty() {
        let n = bits.trailing_zeros();
        bits.clear_bit(n);
        result |= 1 << n;
    }
    result
}

impl Fingerprint5 {
    const SIZE: usize = 5;
    const MAX_EDGE_COUNT: usize = CompactMotif5::max_edge_count_tot();

    pub fn new() -> Self {
        Self {
            order_map: [0u16; Self::SIZE],
            edge_connection_map: (
                [0u16; CompactMotif5::max_edge_count(2)],
                [0u32; CompactMotif5::max_edge_count(3)],
            ),
            edge_connection_map_sizes: (0, 0),
        }
    }

    pub fn build_order_map(&mut self, cm: &CompactMotif5) {
        let mut order_map = [0u16; Self::SIZE];

        for e in cm.iter_edges_ids() {
            let nodes = CompactMotif5::NODE_MAP[e];
            let edge_size = nodes.count() as usize;
            let mut bits = nodes;
            while !bits.is_empty() {
                let n = bits.trailing_zeros();
                bits.clear_bit(n);
                order_map[n as usize] += 1 << (3 * (edge_size - 2));
            }
        }
        order_map.sort_unstable();

        self.order_map = order_map;
    }

    fn compute_group_id_adj() -> Vec<Vec<Vec<CompactMotif5>>> {
        let max_edge_count = Self::MAX_EDGE_COUNT;
        let mut group_id_adj = vec![vec![vec![CompactMotif5::EMPTY; 6]; 4]; max_edge_count];

        for outer in 0..max_edge_count {
            let mut cross_edges =
                CompactMotif5::PART_OVERLAPS[outer] & !CompactMotif5::FULL_OVERLAPS[outer];
            cross_edges.remove_edge(outer);
            cross_edges.remove_order(5);

            for inner in cross_edges.iter_edges_ids() {
                let overlapping_nodes =
                    CompactMotif5::NODE_MAP[outer].bitand(&CompactMotif5::NODE_MAP[inner]);
                let overlapping_size = overlapping_nodes.count() as usize;

                let overlapping_group_idx = {
                    let node_induced_edge =
                        CompactMotif5::EDGE_MAP[binstore_u8_value(overlapping_nodes)];
                    let filtered =
                        CompactMotif5::FULL_OVERLAPS[outer].filtered_by_order(overlapping_size);
                    filtered
                        .iter_edges_ids()
                        .take_while(|&e| e < node_induced_edge)
                        .count()
                };

                group_id_adj[outer][overlapping_size - 1][overlapping_group_idx].add_edge(inner);
            }
        }

        group_id_adj
    }

    pub fn build_edge_connection_map(&mut self, cm: &CompactMotif5) {
        let mut edge_connection_map_sizes = (0, 0);
        let mut edge_connection_map = (
            [0u16; CompactMotif5::max_edge_count(2)],
            [0u32; CompactMotif5::max_edge_count(3)],
        );

        let group_id_adj = Self::compute_group_id_adj();

        for e in cm.filtered_by_order(2).iter_edges_ids() {
            let out_10 = *cm & group_id_adj[e][0][0];
            let out_11 = *cm & group_id_adj[e][0][1];

            let out_20 = *cm & group_id_adj[e][1][0];

            let packed_out_20 = (out_10.filtered_by_order(2).edge_count() << 0)
                | (out_10.filtered_by_order(3).edge_count() << 2)
                | (out_10.filtered_by_order(4).edge_count() << 4);
            let packed_out_21 = (out_11.filtered_by_order(2).edge_count() << 0)
                | (out_11.filtered_by_order(3).edge_count() << 2)
                | (out_11.filtered_by_order(4).edge_count() << 4);

            let packed_out_30 = (out_20.filtered_by_order(3).edge_count() << 0)
                | (out_20.filtered_by_order(4).edge_count() << 2);

            let mut edge_infos = [packed_out_20, packed_out_21, packed_out_30];
            edge_infos[0..2].sort_unstable();

            let entry = ((edge_infos[0] as u16) << 0)
                | ((edge_infos[1] as u16) << 5)
                | ((edge_infos[2] as u16) << 10);
            edge_connection_map.0[edge_connection_map_sizes.0] = entry;
            edge_connection_map_sizes.0 += 1;
        }

        for e in cm.filtered_by_order(3).iter_edges_ids() {
            let out_10 = *cm & group_id_adj[e][0][0];
            let out_11 = *cm & group_id_adj[e][0][1];
            let out_12 = *cm & group_id_adj[e][0][2];

            let out_20 = *cm & group_id_adj[e][1][0];
            let out_21 = *cm & group_id_adj[e][1][1];
            let out_22 = *cm & group_id_adj[e][1][2];

            let out_30 = *cm & group_id_adj[e][2][0];

            let packed_out_10 = (out_10.filtered_by_order(2).edge_count() << 0)
                | (out_10.filtered_by_order(3).edge_count() << 2);
            let packed_out_11 = (out_11.filtered_by_order(2).edge_count() << 0)
                | (out_11.filtered_by_order(3).edge_count() << 2);
            let packed_out_12 = (out_12.filtered_by_order(2).edge_count() << 0)
                | (out_12.filtered_by_order(3).edge_count() << 2);

            let packed_out_20 = (out_20.filtered_by_order(3).edge_count() << 0)
                | (out_20.filtered_by_order(4).edge_count() << 2);
            let packed_out_21 = (out_21.filtered_by_order(3).edge_count() << 0)
                | (out_21.filtered_by_order(4).edge_count() << 2);
            let packed_out_22 = (out_22.filtered_by_order(3).edge_count() << 0)
                | (out_22.filtered_by_order(4).edge_count() << 2);

            let packed_out_30 = out_30.filtered_by_order(4).edge_count() << 0;

            let mut edge_infos = [
                packed_out_10,
                packed_out_11,
                packed_out_12,
                packed_out_20,
                packed_out_21,
                packed_out_22,
                packed_out_30,
            ];

            edge_infos[0..3].sort_unstable();
            edge_infos[3..6].sort_unstable();

            let entry = (edge_infos[0] as u32) << 0
                | (edge_infos[1] as u32) << 3
                | (edge_infos[2] as u32) << 6
                | (edge_infos[3] as u32) << 9
                | (edge_infos[4] as u32) << 12
                | (edge_infos[5] as u32) << 15
                | (edge_infos[6] as u32) << 18;
            edge_connection_map.1[edge_connection_map_sizes.1] = entry;
            edge_connection_map_sizes.1 += 1;
        }

        edge_connection_map.0[0..edge_connection_map_sizes.0].sort_unstable();
        edge_connection_map.1[0..edge_connection_map_sizes.1].sort_unstable();

        self.edge_connection_map = edge_connection_map;
        self.edge_connection_map_sizes = edge_connection_map_sizes;
    }
}

impl Debug for Fingerprint5 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rv = String::new();
        rv += format!("Order map: \n").as_str();
        for i in 0..Self::SIZE {
            let order2 = (self.order_map[i] >> 0) & ((1 << 3) - 1);
            let order3 = (self.order_map[i] >> 3) & ((1 << 3) - 1);
            let order4 = (self.order_map[i] >> 6) & ((1 << 3) - 1);
            rv += format!("\t {:?}\n", [order2, order3, order4]).as_str();
        }

        f.write_str(rv.as_str())
    }
}

impl Hash for Fingerprint5 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.order_map.hash(state);
        self.edge_connection_map.0[0..self.edge_connection_map_sizes.0].hash(state);
        self.edge_connection_map.1[0..self.edge_connection_map_sizes.1].hash(state);
    }
}

impl PartialEq for Fingerprint5 {
    fn eq(&self, other: &Self) -> bool {
        self.order_map == other.order_map
            && self.edge_connection_map.0[0..self.edge_connection_map_sizes.0]
                == other.edge_connection_map.0[0..other.edge_connection_map_sizes.0]
            && self.edge_connection_map.1[0..self.edge_connection_map_sizes.1]
                == other.edge_connection_map.1[0..other.edge_connection_map_sizes.1]
    }
}

impl Eq for Fingerprint5 {}

impl From<CompactMotif5> for Fingerprint5 {
    fn from(cm: CompactMotif5) -> Self {
        let mut rv = Fingerprint5::new();
        rv.build_order_map(&cm);
        rv.build_edge_connection_map(&cm);
        rv
    }
}

impl Into<CompactMotif5> for Fingerprint5 {
    fn into(self) -> CompactMotif5 {
        todo!()
    }
}

impl Fingerprint for Fingerprint5 {
    type MotifType = CompactMotif5;

    fn get_canonical_rep(&self) -> Self::MotifType {
        todo!()
    }
}

impl Fingerprintable for CompactMotif5 {
    type FingerprintType = Fingerprint5;

    fn fingerprint(&self) -> Self::FingerprintType {
        Fingerprint5::from(*self)
    }
}
