use crate::CompactMotif;
use crate::bin_store::BinStore;
// use crate::util::sorting_network::TryNetSort;
use hashbrown::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

type CompactMotif3 = CompactMotif!(3);
type CompactMotif4 = CompactMotif!(4);
type CompactMotif5 = CompactMotif!(5);

pub trait Fingerprint: Sized + Hash + Eq + PartialEq + Copy + Clone {
    type CMType: CompactMotifConfigurator;

    fn get_canonical_rep(&self) -> Self::CMType;
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Fingerprint3 {
    /// The number of edges of size 2 and 3. layed out as follows:
    /// [3-edge count (4 bits)][2-edge count (4 bits)]
    edge_counts: u8,
}

// impl Fingerprint3 {
//     const SIZE: usize = 3;
//     const MAX_EDGE_COUNT: usize = 2 << Self::SIZE;
//
//     pub fn get_canonical_rep(&self) -> CompactMotif3 {
//         let count_2 = self.edge_counts & ((1 << 4) - 1);
//         let count_3 = (self.edge_counts >> 4) & ((1 << 4) - 1);
//         let mut rv = CompactMotif3::zero();
//
//         for i in 0..count_2 {
//             let nodes = {
//                 let mut rv = [i, (i + 1) % 3];
//                 rv.sort_unstable();
//                 rv
//             };
//
//             rv.add_edge_with_nodes(BinStore::<u8, 1>::with_elements(nodes).clone);
//         }
//         if count_3 != 0 {
//             rv.add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2]));
//         }
//         rv.into()
//     }
//
//     pub fn __str__(&self) -> String {
//         let order2 = (self.edge_counts >> 0) & ((1 << 4) - 1);
//         let order3 = (self.edge_counts >> 4) & ((1 << 4) - 1);
//         format!("edge_counts {:?}", [order2, order3])
//     }
//
//     #[staticmethod]
//     pub fn enum_all(min_hx_size: usize, max_hx_size: usize) -> HashSet<Self> {
//         let mut rv = HashSet::with_capacity(6);
//         for motif in CompactMotif::<3>::enum_connected_motifs(min_hx_size..=max_hx_size) {
//             rv.insert(motif.fingerprint());
//         }
//         rv
//     }
// }
//
// impl Debug for Fingerprint3 {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let order2 = (self.edge_counts >> 0) & ((1 << 4) - 1);
//         let order3 = (self.edge_counts >> 4) & ((1 << 4) - 1);
//         write!(f, "{:?}", [order2, order3])
//     }
// }
//
// impl From<CompactMotif3> for Fingerprint3 {
//     fn from(cm: CompactMotif3) -> Self {
//         let mut edge_counts = 0u8;
//         for nodes in cm.iter_nodes() {
//             edge_counts += 1 << (4 * (nodes.len() as usize - 2));
//         }
//
//         Fingerprint3 { edge_counts }
//     }
// }
//
// impl Into<CompactMotif3> for Fingerprint3 {
//     fn into(self) -> CompactMotif3 {
//         self.get_canonical_rep().into()
//     }
// }

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct Fingerprint4 {
    /// For each node, a histogram of the sizes of the edges it participates in. Sorted by node
    /// degree, then lexicographically by histogram
    order_map: [u8; 4],
    /// For each edge, the number of edges in which its fully contained. For performance reasons a
    /// spectrogram is saved, so that full sorting can be avoided.
    ///
    /// Each edge could be fully contained in at most 3 other edges, so 2 bits are enough to store
    /// this information for each edge.
    inclusions: u32,
}

// impl Fingerprint4 {
//     const SIZE: usize = <Self as CMAssociated>::CMType::SIZE;
//
//     #[staticmethod]
//     pub fn new() -> Self {
//         Self {
//             order_map: [0u8; Self::SIZE],
//             inclusions: 0,
//         }
//     }
//
//     pub fn get_canonical_rep(&self) -> CompactMotif4 {
//         let mut rv = CompactMotif::<4>::zero();
//
//         let mut out_2 = [(0u8, 0u8); Self::SIZE];
//         for i in 0..Self::SIZE {
//             out_2[i] = (i as u8, self.order_map[i] & 3);
//         }
//
//         // Loop until all degrees are satisfied
//         loop {
//             // Sort descending by degree
//             out_2.sort_unstable_by(|a, b| b.1.cmp(&a.1));
//
//             let (current_node, degree) = out_2[0];
//             if degree == 0 {
//                 break;
//             } // All done
//
//             out_2[0].1 = 0; // Consume this node's degree
//
//             let mut to_remove = degree;
//             for j in 1..Self::SIZE {
//                 if to_remove > 0 && out_2[j].1 > 0 {
//                     rv.add_edge_with_nodes(CompressedNodeSet::from_array([
//                         current_node,
//                         out_2[j].0,
//                     ]));
//                     out_2[j].1 -= 1;
//                     to_remove -= 1;
//                 }
//             }
//         }
//
//         let count_4 = (self.order_map[0] >> 4) & 3;
//
//         let total_3_deg: usize = self
//             .order_map
//             .iter()
//             .map(|&x| ((x >> 2) & 3) as usize)
//             .sum();
//         let count_3 = total_3_deg / 3;
//
//         let mut check_3_edges = || {
//             match count_3 {
//                 0 => {}
//                 1 => {
//                     // One 3-edge: excludes the unique node with a 3-degree of 0
//                     let expected = ((self.inclusions >> (3 * count_4)) & ((1 << 3) - 1)) - 1;
//                     for i in 0..Self::SIZE {
//                         if rv.neighbors(i).edge_count() == expected {
//                             let mut nodes = CompressedNodeSet::from_array([0, 1, 2, 3]);
//                             nodes.remove(i);
//                             rv.add_edge_with_nodes(nodes);
//                             break;
//                         }
//                     }
//                 }
//                 2 => {
//                     for i in 0..Self::SIZE {
//                         for j in (i + 1)..Self::SIZE {
//                             let incl_0 = (rv.neighbors(i) & rv.neighbors(j)).edge_count();
//                             let incl_1 = (rv.neighbors(i) | rv.neighbors(j)).edge_count() - incl_0;
//                             let incl_2 = rv.edge_count() - incl_0 - incl_1;
//
//                             let expected_incl_0 =
//                                 ((self.inclusions >> (3 * count_4)) & ((1 << 3) - 1)) - 2; // removing
//                             // 2 ttiangles
//                             let expected_incl_1 =
//                                 (self.inclusions >> (3 * (1 + count_4))) & ((1 << 3) - 1);
//                             let expected_incl_2 =
//                                 (self.inclusions >> (3 * (2 + count_4))) & ((1 << 3) - 1);
//
//                             if incl_0 == expected_incl_0
//                                 && incl_1 == expected_incl_1
//                                 && incl_2 == expected_incl_2
//                             {
//                                 let mut e1 = CompressedNodeSet::from_array([0, 1, 2, 3]);
//                                 let mut e2 = CompressedNodeSet::from_array([0, 1, 2, 3]);
//                                 e1.remove(i);
//                                 e2.remove(j);
//                                 rv.add_edge_with_nodes(e1);
//                                 rv.add_edge_with_nodes(e2);
//                                 return;
//                             }
//                         }
//                     }
//                 }
//                 3 => {
//                     for i in 0..Self::SIZE {
//                         for j in 0..Self::SIZE {
//                             if i == j {
//                                 continue;
//                             }
//
//                             let incl_2 = (rv.neighbors(i)).edge_count();
//                             let incl_1 = rv.edge_count() - incl_2;
//
//                             let expected_incl_1 =
//                                 (self.inclusions >> (3 * (1 + count_4))) & ((1 << 3) - 1);
//                             let expected_incl_2 =
//                                 (self.inclusions >> (3 * (2 + count_4))) & ((1 << 3) - 1);
//
//                             if incl_1 == expected_incl_1 && incl_2 == expected_incl_2 {
//                                 let other_nodes = [0, 1, 2, 3]
//                                     .into_iter()
//                                     .filter(|e| *e != i && *e != j)
//                                     .collect::<Vec<_>>();
//                                 let (a, b) = (other_nodes[0], other_nodes[1]);
//
//                                 let mut e1 = CompressedNodeSet::from_array([0, 1, 2, 3]);
//                                 let mut e2 = CompressedNodeSet::from_array([0, 1, 2, 3]);
//                                 let mut e3 = CompressedNodeSet::from_array([0, 1, 2, 3]);
//                                 e1.remove(a);
//                                 e2.remove(b);
//                                 e3.remove(j);
//                                 rv.add_edge_with_nodes(e1);
//                                 rv.add_edge_with_nodes(e2);
//                                 rv.add_edge_with_nodes(e3);
//
//                                 return;
//                             }
//                         }
//                     }
//                 }
//                 4 => {
//                     for i in 0..Self::SIZE {
//                         let mut e = CompressedNodeSet::from_array([0, 1, 2, 3]);
//                         e.remove(i);
//                         rv.add_edge_with_nodes(e);
//                     }
//                 }
//                 _ => {}
//             }
//         };
//         check_3_edges();
//
//         if count_4 == 1 {
//             rv.add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2, 3]));
//         }
//
//         rv.into()
//     }
// }
//
// impl Debug for Fingerprint4 {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut rv = String::new();
//         rv += format!("Order map: \n").as_str();
//         for i in 0..<Self as CMAssociated>::CMType::SIZE {
//             let order2 = (self.order_map[i] >> 0) & ((1 << 2) - 1);
//             let order3 = (self.order_map[i] >> 2) & ((1 << 2) - 1);
//             let order4 = (self.order_map[i] >> 4) & ((1 << 2) - 1);
//             rv += format!("\t {:?}\n", [order2, order3, order4]).as_str();
//         }
//         rv += format!("Inclusions: {:?}\n", self.inclusions).as_str();
//         f.write_str(rv.as_str())
//     }
// }
//
// impl From<CompactMotif4> for Fingerprint4 {
//     fn from(cm: CompactMotif4) -> Self {
//         let mut order_map = [0u8; Self::SIZE];
//         for nodes in cm.iter_nodes() {
//             for n in nodes {
//                 order_map[n] += 1 << (2 * (nodes.len() as usize - 2));
//             }
//         }
//         order_map.sort_unstable();
//
//         let mut inclusions = 0;
//         for e in cm {
//             let inclusions_count = cm.inclusions(e).edge_count() as u8 - 1;
//             inclusions += 1 << (3 * (inclusions_count as u32));
//         }
//
//         Fingerprint4 {
//             order_map,
//             inclusions,
//         }
//     }
// }
//
// impl Into<CompactMotif4> for Fingerprint4 {
//     fn into(self) -> CompactMotif4 {
//         self.get_canonical_rep().into()
//     }
// }

#[derive(Copy, Clone)]
pub struct Fingerprint5 {
    /// for each node, a histogram of the sizes of the edges it participates in. Sorted by node
    /// degree, then lexicographically by histogram
    order_map: [u16; 5],

    /// For each edge it contains the following information:
    /// bit 0-5:    the number of edges that are fully contained within it.
    /// bit 5-10:   the number of edges that overlap with it in at least one node (including itself)
    /// bit 10-15:  the number of edges in which its fully contained
    // edges_props: [u16; <Self as CMAssociated>::CMType::MAX_EDGE_COUNT],

    /// For each edge it stores information about its connectivity.
    /// One could think of it as a small tensor with informations stored as follows:
    /// cover_tree[overlapping node set size][group id of the overlapping node set][overlapping edge size] = number of edges with this configuration
    edge_connection_map: (
        // [2x5b][1x4b][2 empty bits]
        [u16; CompactMotif5::max_edge_count(2)],
        // [3x3b][3x3b][1x2b][12 empty bits]
        [u32; CompactMotif5::max_edge_count(3)],
        // [4x1b][4x1b][4x1b][4 empty bits] UNUSED, perhaps could prove useful for furute extension
        // [u16; CompactMotif::<5>::max_edge_count(4)],
    ),

    edge_connection_map_sizes: (usize, usize),
}

// impl Fingerprint5 {
//     pub const SIZE: usize = <Self as CMAssociated>::CMType::SIZE;
//     pub const MAX_EDGE_COUNT: usize = <Self as CMAssociated>::CMType::MAX_EDGE_COUNT;
//
//     pub const FULL_OVERLAPS:
//         <<Self as CMAssociated>::CMType as CompactMotifConfigurator>::FullOverlapsType =
//         <Self as CMAssociated>::CMType::FULL_OVERLAPS;
//
//     pub const PART_OVERLAPS:
//         <<Self as CMAssociated>::CMType as CompactMotifConfigurator>::PartOverlapsType =
//         <Self as CMAssociated>::CMType::PART_OVERLAPS;
//
//     pub const NODE_MAP: <<Self as CMAssociated>::CMType as CompactMotifConfigurator>::NodeMapType =
//         <Self as CMAssociated>::CMType::NODE_MAP;
//
//     pub const EDGE_MAP: <<Self as CMAssociated>::CMType as CompactMotifConfigurator>::EdgeMapType =
//         <Self as CMAssociated>::CMType::EDGE_MAP;
//
//     pub const ADJ: <<Self as CMAssociated>::CMType as CompactMotifConfigurator>::AdjType =
//         <Self as CMAssociated>::CMType::ADJ;
// }
//
// impl Fingerprint5 {
//     // [outer_edge][overlap_size - 1][group_id]
//     pub const GROUP_ID_ADJ: [[[<Self as CMAssociated>::CMType; 6]; 4];
//         <Self as CMAssociated>::CMType::MAX_EDGE_COUNT] = const {
//         type MotifType = <Fingerprint5 as CMAssociated>::CMType;
//
//         let mut group_id_adj = [[[<Self as CMAssociated>::CMType::ZERO; 6]; 4];
//             <Self as CMAssociated>::CMType::MAX_EDGE_COUNT];
//         let mut outer = 0;
//         while outer < Self::MAX_EDGE_COUNT {
//             let mut cross_edges = MotifType::PART_OVERLAPS[outer]
//                 .const_bitand(MotifType::FULL_OVERLAPS[outer].const_not());
//             cross_edges.const_remove_edge(outer);
//             cross_edges.const_remove_order(5);
//
//             let mut iter = cross_edges.container;
//             while iter != 0 {
//                 let inner = iter.trailing_zeros() as usize;
//                 iter &= iter - 1;
//
//                 let overlapping_nodes = Self::NODE_MAP[inner].bitand(Self::NODE_MAP[outer]);
//                 let overlapping_size = overlapping_nodes.len() as usize;
//
//                 let overlapping_group_idx = {
//                     let node_induced_edge = Self::EDGE_MAP[overlapping_nodes.nodes as usize];
//                     (Self::FULL_OVERLAPS[outer]
//                         .filtered_by_order(overlapping_size)
//                         .container
//                         & ((1 << node_induced_edge) - 1))
//                         .count_ones() as u32
//                 } as usize;
//
//                 group_id_adj[outer][overlapping_size - 1][overlapping_group_idx]
//                     .const_add_edge(inner);
//             }
//
//             outer += 1;
//         }
//
//         group_id_adj
//     };
//
//     pub fn new() -> Self {
//         Self {
//             order_map: [0u16; <Self as CMAssociated>::CMType::SIZE],
//             edge_connection_map: (
//                 [0u16; CompactMotif::<5>::max_edge_count(2)],
//                 [0u32; CompactMotif::<5>::max_edge_count(3)],
//             ),
//             edge_connection_map_sizes: (0, 0),
//         }
//     }
//
//     pub fn build_order_map(&mut self, cm: &CompactMotif5) {
//         let mut order_map = [0u16; <Self as CMAssociated>::CMType::SIZE];
//
//         for e in cm {
//             let nodes = <Self as CMAssociated>::CMType::NODE_MAP[e];
//             for n in nodes {
//                 order_map[n] += 1 << (3 * (nodes.len() as usize - 2));
//             }
//         }
//         order_map.sort_unstable();
//
//         self.order_map = order_map;
//     }
//
//     pub fn build_edge_connection_map(&mut self, cm: &CompactMotif5) {
//         let mut edge_connection_map_sizes = (0, 0);
//         let mut edge_connection_map = (
//             [0u16; CompactMotif::<5>::max_edge_count(2)],
//             [0u32; CompactMotif::<5>::max_edge_count(3)],
//         );
//
//         // order 2 edges
//         for e in cm.filtered_by_order(2) {
//             // let mut edge_infos = insert_cross_edge!(e, 2, [0u8; 3]);
//             let out_10 = *cm & Self::GROUP_ID_ADJ[e][0][0];
//             let out_11 = *cm & Self::GROUP_ID_ADJ[e][0][1];
//
//             let out_20 = *cm & Self::GROUP_ID_ADJ[e][1][0];
//
//             let packed_out_20 = (out_10.filtered_by_order(2).edge_count() << 0)
//                 | (out_10.filtered_by_order(3).edge_count() << 2)
//                 | (out_10.filtered_by_order(4).edge_count() << 4);
//             let packed_out_21 = (out_11.filtered_by_order(2).edge_count() << 0)
//                 | (out_11.filtered_by_order(3).edge_count() << 2)
//                 | (out_11.filtered_by_order(4).edge_count() << 4);
//
//             let packed_out_30 = (out_20.filtered_by_order(3).edge_count() << 0)
//                 | (out_20.filtered_by_order(4).edge_count() << 2);
//
//             let mut edge_infos = [packed_out_20, packed_out_21, packed_out_30];
//             edge_infos[0..2].try_network_sort();
//             // edge_infos[0..2].sort_unstable();
//
//             let entry = ((edge_infos[0] as u16) << 0)
//                 | ((edge_infos[1] as u16) << 5)
//                 | ((edge_infos[2] as u16) << 10);
//             edge_connection_map.0[edge_connection_map_sizes.0] = entry;
//             edge_connection_map_sizes.0 += 1;
//         }
//
//         // order 3 edges
//         for e in cm.filtered_by_order(3) {
//             // let mut edge_infos = insert_cross_edge!(e, 3, [0u8; 7]);
//
//             let out_10 = *cm & Self::GROUP_ID_ADJ[e][0][0];
//             let out_11 = *cm & Self::GROUP_ID_ADJ[e][0][1];
//             let out_12 = *cm & Self::GROUP_ID_ADJ[e][0][2];
//
//             let out_20 = *cm & Self::GROUP_ID_ADJ[e][1][0];
//             let out_21 = *cm & Self::GROUP_ID_ADJ[e][1][1];
//             let out_22 = *cm & Self::GROUP_ID_ADJ[e][1][2];
//
//             let out_30 = *cm & Self::GROUP_ID_ADJ[e][2][0];
//
//             let packed_out_10 = (out_10.filtered_by_order(2).edge_count() << 0)
//                 | (out_10.filtered_by_order(3).edge_count() << 2);
//             let packed_out_11 = (out_11.filtered_by_order(2).edge_count() << 0)
//                 | (out_11.filtered_by_order(3).edge_count() << 2);
//             let packed_out_12 = (out_12.filtered_by_order(2).edge_count() << 0)
//                 | (out_12.filtered_by_order(3).edge_count() << 2);
//
//             let packed_out_20 = (out_20.filtered_by_order(3).edge_count() << 0)
//                 | (out_20.filtered_by_order(4).edge_count() << 2);
//             let packed_out_21 = (out_21.filtered_by_order(3).edge_count() << 0)
//                 | (out_21.filtered_by_order(4).edge_count() << 2);
//             let packed_out_22 = (out_22.filtered_by_order(3).edge_count() << 0)
//                 | (out_22.filtered_by_order(4).edge_count() << 2);
//
//             let packed_out_30 = out_30.filtered_by_order(4).edge_count() << 0;
//
//             let mut edge_infos = [
//                 packed_out_10,
//                 packed_out_11,
//                 packed_out_12,
//                 packed_out_20,
//                 packed_out_21,
//                 packed_out_22,
//                 packed_out_30,
//             ];
//
//             // edge_infos[0..3].try_network_sort();
//             // edge_infos[3..6].try_network_sort();
//             edge_infos[0..3].sort_unstable();
//             edge_infos[3..6].sort_unstable();
//
//             let entry = (edge_infos[0] as u32) << 0
//                 | (edge_infos[1] as u32) << 3
//                 | (edge_infos[2] as u32) << 6
//                 | (edge_infos[3] as u32) << 9
//                 | (edge_infos[4] as u32) << 12
//                 | (edge_infos[5] as u32) << 15
//                 | (edge_infos[6] as u32) << 18;
//             edge_connection_map.1[edge_connection_map_sizes.1] = entry;
//             edge_connection_map_sizes.1 += 1;
//         }
//
//         // edge_connection_map.0[0..edge_connection_map_sizes.0].try_network_sort();
//         // edge_connection_map.1[0..edge_connection_map_sizes.1].try_network_sort();
//         edge_connection_map.0[0..edge_connection_map_sizes.0].sort_unstable();
//         edge_connection_map.1[0..edge_connection_map_sizes.1].sort_unstable();
//
//         self.edge_connection_map = edge_connection_map;
//         self.edge_connection_map_sizes = edge_connection_map_sizes;
//     }
// }
//
// impl Debug for Fingerprint5 {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut rv = String::new();
//         rv += format!("Order map: \n").as_str();
//         for i in 0..<Self as CMAssociated>::CMType::SIZE {
//             let order2 = (self.order_map[i] >> 0) & ((1 << 3) - 1);
//             let order3 = (self.order_map[i] >> 3) & ((1 << 3) - 1);
//             let order4 = (self.order_map[i] >> 6) & ((1 << 3) - 1);
//             rv += format!("\t {:?}\n", [order2, order3, order4]).as_str();
//         }
//
//         f.write_str(rv.as_str())
//     }
// }
//
// impl Hash for Fingerprint5 {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.order_map.hash(state);
//         self.edge_connection_map.0[0..self.edge_connection_map_sizes.0].hash(state);
//         self.edge_connection_map.0[0..self.edge_connection_map_sizes.0].hash(state);
//     }
// }
//
// impl PartialEq for Fingerprint5 {
//     fn eq(&self, other: &Self) -> bool {
//         self.order_map == other.order_map
//             && self.edge_connection_map.0[0..self.edge_connection_map_sizes.0]
//                 == other.edge_connection_map.0[0..self.edge_connection_map_sizes.0]
//             && self.edge_connection_map.1[0..self.edge_connection_map_sizes.1]
//                 == other.edge_connection_map.1[0..self.edge_connection_map_sizes.1]
//     }
// }
//
// impl Eq for Fingerprint5 {}
//
// impl From<CompactMotif5> for Fingerprint5 {
//     fn from(cm: CompactMotif5) -> Self {
//         let mut rv = Fingerprint5::new();
//         rv.build_order_map(&cm);
//         rv.build_edge_connection_map(&cm);
//         rv
//     }
// }
//
// impl Into<CompactMotif5> for Fingerprint5 {
//     fn into(self) -> CompactMotif5 {
//         todo!()
//     }
// }
//
// Unused parts for furutre improvements
// 4 => {
//     const OVERLAP_GROUP_OFFSET: [usize; 4] = [0, 4, 10, 14];
//
//     // const OUTER_OFFSETS: [usize; 4] = [0, 4, 10, 14];
//     // const BLOCK_SIZE: [usize; 4] = [1, 1, 1, 0];
//     const INNER_OFFSET: [[usize; 3]; 4] =
//         [[0, 1, 1], [0, 0, 1], [0, 0, 0], [0, 0, 0]];
//
//     let mut edge_infos = insert_cross_edge!(e, 4, [0u8; 15]);
//     // sort_slice4(&mut edge_infos[0..4]);
//     // sort_slice6(&mut edge_infos[4..10]);
//     // sort_slice4(&mut edge_infos[10..14]);
//     edge_infos[0..4].sort_unstable();
//     edge_infos[4..10].sort_unstable();
//     edge_infos[10..14].sort_unstable();
//     let entry = (edge_infos[0] as u16) << 0
//         | (edge_infos[1] as u16) << 1
//         | (edge_infos[2] as u16) << 2
//         | (edge_infos[3] as u16) << 3
//         | (edge_infos[4] as u16) << 4
//         | (edge_infos[5] as u16) << 5
//         | (edge_infos[6] as u16) << 6
//         | (edge_infos[7] as u16) << 7
//         | (edge_infos[8] as u16) << 8
//         | (edge_infos[9] as u16) << 9
//         | (edge_infos[10] as u16) << 10
//         | (edge_infos[11] as u16) << 11
//         | (edge_infos[12] as u16) << 12
//         | (edge_infos[13] as u16) << 13;
//     edge_connection_map.2[edge_connection_map_sizes.2] = entry;
//     edge_connection_map_sizes.2 += 1;
// }
