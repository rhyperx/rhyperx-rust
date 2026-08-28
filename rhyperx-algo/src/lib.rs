// use duplicate::duplicate_item;

// pub mod motifs;
pub mod bin_store;
pub mod util;

// pub trait ComptactMotifConfig {
//     pub type Container
//     // const SIZE: usize = 5;
//     // type AdjType = [usize; Self::SIZE];
//     // const M: [usize; <Self as X>::size];
// }

// impl<const N: usize> Trait for X<N> {
//     const SIZE: usize = N;
// }
//
// const fn compute_flat_size(n: usize) -> usize {
//     // Computes the required size of the flat array to store the motif data.
//     42
// }
//
//
// pub fn test() -> compact_motif!(4) {}

// pub trait BitContainer {
//     const ORDER: usize;
//     type ContainerType;
//     // fn get(&self, index: usize) -> bool;
//     // fn set(&mut self, index: usize, value: bool);
// }
//
// pub struct CompactMotif<const N: usize, T: BitContainer> {
//     bits: [u8; N], // container: <Self as BitContainer>::ContainerType,
//                    // x: [usize; const { something(N) }],
//                    // _phantom: std::marker::PhantomData<[(); N]>,
// }
//
// // pub trait AsPrimitive {
// //     type PrimitiveType;
// //     const BITS_COUNT: usize;
// //     fn as_primitive(&self) -> Self::PrimitiveType;
// // }
//
// pub trait Config {
//     const ORDER: usize;
//     type ContainerType: AsPrimitive<PrimitiveType = u64>;
// }
//
// pub struct CompactMotif<const N: usize, T: AsPrimitive>
// // where
// //     Self: BitContainer,
// {
//     bits: [u8; N], // container: <Self as BitContainer>::ContainerType,
//                    // x: [usize; const { something(N) }],
//                    // _phantom: std::marker::PhantomData<[(); N]>,
// }

// impl<const N: usize, T: AsPrimitive> CompactMotif<N, T>
// {
//
//
//     const ADJ:  = const {
//         let mut raw_adj = [0; N];
//         iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
//             let mut i = 0;
//             while i < edge_size {
//                 raw_adj[edge[i]] |= 1 << edge_idx;
//                 i = i + 1;
//             }
//         });
//
//         let mut adj = [Self::new(0); N];
//         let mut i = 0;
//         while i < N {
//             adj[i] = Self::new(raw_adj[i]);
//             i += 1;
//         }
//         adj
//     };
//
//     // const NODE_MAP: Self::NodeMapType = const {
//     //     let mut rv = [CompressedNodeSet::new(0); Self::MAX_EDGE_COUNT];
//     //     iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
//     //         let mut i = 0;
//     //         let mut bitset = 0;
//     //         while i < edge_size {
//     //             bitset |= 1 << edge[i];
//     //             i = i + 1;
//     //         }
//     //         rv[edge_idx] = CompressedNodeSet::new(bitset);
//     //     });
//     //     rv
//     // };
//     //
//     // const EDGE_MAP: Self::EdgeMapType = const {
//     //     let mut rv = [0; 1 << $order];
//     //     iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
//     //         let mut i = 0;
//     //         let mut bitset = 0;
//     //         while i < edge_size {
//     //             bitset |= 1 << edge[i];
//     //             i = i + 1;
//     //         }
//     //         rv[bitset] = edge_idx as u8;
//     //     });
//     //     rv
//     // };
//     //
//     // const FULL_OVERLAPS: Self::FullOverlapsType = const {
//     //     let mut rv_raw = [!0; Self::MAX_EDGE_COUNT];
//     //     iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
//     //         let mut i = 0;
//     //         let mut bitset = 0u32;
//     //         while i < edge_size {
//     //             bitset |= 1 << edge[i];
//     //             i = i + 1;
//     //         }
//     //         bitset = !bitset & ((1 << $order) - 1);
//     //         while bitset != 0 {
//     //             let node = bitset.trailing_zeros() as usize;
//     //             bitset &= !(1 << node);
//     //             rv_raw[edge_idx] &= !Self::ADJ[node].container;
//     //         }
//     //     });
//     //
//     //     let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
//     //     let mut i = 0;
//     //     while i < Self::MAX_EDGE_COUNT {
//     //         rv[i] = Self::new(rv_raw[i] & ((1 << Self::MAX_EDGE_COUNT) - 1));
//     //         i += 1;
//     //     }
//     //     rv
//     // };
//     //
//     // const PART_OVERLAPS: Self::PartOverlapsType = const {
//     //     let adj = Self::ADJ;
//     //     let mut rv_raw = [0; Self::MAX_EDGE_COUNT];
//     //     iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
//     //         let mut i = 0;
//     //         while i < edge_size {
//     //             rv_raw[edge_idx] |= adj[edge[i]].container;
//     //             i = i + 1;
//     //         }
//     //     });
//     //
//     //     let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
//     //     let mut i = 0;
//     //     while i < Self::MAX_EDGE_COUNT {
//     //         rv[i] = Self::new(rv_raw[i] & ((1 << Self::MAX_EDGE_COUNT) - 1));
//     //         i += 1;
//     //     }
//     //     rv
//     // };
//     //
//     // const INCLUSION_MAP: Self::InclusionMapType = const {
//     //     let mut rv_raw = [0; Self::MAX_EDGE_COUNT];
//     //     iter_hyperedges!($order, 1..=$order, |_edge, _edge_size, edge_idx| {
//     //         let mut iter = Self::FULL_OVERLAPS[edge_idx].container;
//     //         while iter != 0 {
//     //             let inner = iter.trailing_zeros() as usize;
//     //             iter &= !(1 << inner);
//     //             rv_raw[inner] |= 1 << edge_idx;
//     //         }
//     //     });
//     //
//     //     let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
//     //     let mut i = 0;
//     //     while i < Self::MAX_EDGE_COUNT {
//     //         rv[i] = Self::new(rv_raw[i]);
//     //         i += 1;
//     //     }
//     //     rv
//     // };
//     //
//     // const EDGE_FILTER_BITMASK: Self::EdgeFilterBitmaskType = const {
//     //     let mut rv_raw = [0; Self::MAX_EDGE_COUNT];
//     //
//     //     let mut shift_offset = 0;
//     //     let mut i = 1;
//     //
//     //     while i <= $order {
//     //         let curr_count = max_hyperedge_count($order, i, i);
//     //
//     //         rv_raw[i] = ((1 << curr_count) - 1) << shift_offset;
//     //
//     //         shift_offset += curr_count;
//     //         i += 1;
//     //     }
//     //
//     //     let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
//     //     let mut i = 0;
//     //     while i < Self::MAX_EDGE_COUNT {
//     //         rv[i] = Self::new(rv_raw[i]);
//     //         i += 1;
//     //     }
//     //     rv
//     // };
//     //
//     // const RELABELING_MAP: Self::RelabelingMap = const {
//     //     let node_map = Self::NODE_MAP;
//     //     let edge_map = Self::EDGE_MAP;
//     //
//     //     let mut relabeling_map = [[0u8; Self::MAX_EDGE_COUNT]; factorial($order)];
//     //
//     //     let mut i = 0;
//     //     while i < factorial($order) {
//     //         let perm = BinPerm::from_usize(i).decode::<$order>();
//     //         let mut j = 0;
//     //
//     //         while j < Self::MAX_EDGE_COUNT {
//     //             let mut old_nodes = node_map[j].nodes;
//     //             let mut new_nodes = 0u8;
//     //
//     //             while old_nodes != 0 {
//     //                 let old_node = old_nodes.trailing_zeros() as usize;
//     //                 old_nodes &= !(1 << old_node);
//     //
//     //                 let new_node = perm[old_node];
//     //                 new_nodes |= 1 << new_node;
//     //             }
//     //
//     //             relabeling_map[i][j] = edge_map[new_nodes as usize];
//     //
//     //             j += 1;
//     //         }
//     //         i += 1;
//     //     }
//     //
//     //     relabeling_map
//     // };
//     // const MAX: usize = Self::SIZE * Self::SIZE * N;
//     //
//     // pub const M: [usize; <Self as Trait>::SIZE] = [1, 2, 3, 4, 5];
// }
//
//
//
// macro_rules! compact_motif {
//     (2) => {
//         CompactMotif<u8>
//     };
//
//     (3) => {
//         CompactMotif<u8>
//     };
//
//     (4) => {
//         CompactMotif<u16>
//     };
//
//     (5) => {
//         CompactMotif<u32>
//     };
//
//     (6) => {
//         CompactMotif<u64>
//     };
//
//     (6) => {
//         CompactMotif<u128>
//     };
//
//     ($n: literal) => {
//         [u64; compute_flat_size($n)]
//     };
// }
//
// // pub trait Config {
// //     const ORDER: usize;
// //     const BIT_ARRAY_SIZE: usize;
// //     type T: BitStore;
// // }
//
// pub struct CompactMotif<T: BitStore, const N: usize, const M: usize> {
//     bits: BitArray<[T; M], Lsb0>,
// }
//
// impl<T: BitStore, const N: usize, const M: usize> CompactMotif<T, N, M> {
//     const ZERO: Self = Self {
//         bits: BitArray::ZERO,
//     };
//
//     const ADJ: [Self; N] = const {
//         let mut raw_adj = [0; N];
//
//         iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
//             let mut i = 0;
//             while i < edge_size {
//                 raw_adj[edge[i]] |= 1 << edge_idx;
//                 i = i + 1;
//             }
//         });
//
//         let mut adj = [Self::ZERO; N];
//         let mut i = 0;
//         while i < N {
//             adj[i] = Self::new(raw_adj[i]);
//             i += 1;
//         }
//         adj
//     };
//
//     pub fn test(&self){
//         self.bits.shift_start(by);
//     }
//
//     // pub fn new(bits: [T; N]) -> Self {
//     //
//     //     Self {
//     //         bits: BitArray::ZERO,
//     //     }
//     //
//     // }
//     // //
//     // const ADJ:  = const {
//     //     let mut raw_adj = [0; $order];
//     //     iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
//     //         let mut i = 0;
//     //         while i < edge_size {
//     //             raw_adj[edge[i]] |= 1 << edge_idx;
//     //             i = i + 1;
//     //         }
//     //     });
//     //
//     //     let mut adj = [Self::new(0); $order];
//     //     let mut i = 0;
//     //     while i < $order {
//     //         adj[i] = Self::new(raw_adj[i]);
//     //         i += 1;
//     //     }
//     //     adj
//     // };
// }
//
// pub fn test() {
//     let example = bitarr![0; 65];
//     example.shift_end
//     // example.count_ones
//     // let x: BitArr!(for 64, in u64, Lsb0) = bitarr!(u64, Lsb0);
//     // let y = x;
// }
//
//
// pub trait AtomicBinContainer {}
// pub struct BinContainer<T: AtomicBinContainer> {
//
// }
//

// Compact bit representation of a motif with N nodes
// N: number of nodes in the motif
// A: number of storage elements required to store the motif
// T: storage type (u8, u16, u32, u64, u128)
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct CompactMotif<T, const N: usize, const A: usize> {
//     pub(self) bits: [T; A],
// }
//
// #[duplicate_item( raw_type; [u8]; [u16]; [u32]; [u64]; [u128]; )]
// impl<const N: usize, const M: usize> CompactMotif<raw_type, N, M> {
//     const ZERO: Self = Self { bits: [0; M] };
//
//     pub fn new(bits: [raw_type; M]) -> Self {
//         Self { bits }
//     }
//
//     const fn set_bit(&mut self, index: usize) {
//         let word_index = index / (raw_type::BITS as usize);
//         let bit_index = index % (raw_type::BITS as usize);
//         self.bits[word_index] |= 1 << bit_index;
//     }
//
//     const fn clear_bit(&mut self, index: usize) {
//         let word_index = index / (raw_type::BITS as usize);
//         let bit_index = index % (raw_type::BITS as usize);
//         self.bits[word_index] &= !(1 << bit_index);
//     }
//
//     const ADJ: [Self; N] = const {
//         let mut adj = [Self::ZERO; N];
//         iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
//             let mut i = 0;
//             while i < edge_size {
//                 adj[edge[i]].set_bit(edge_idx);
//                 i = i + 1;
//             }
//         });
//
//         adj
//     };
//
//     const NODE_MAP: Self::NodeMapType = const {
//         let mut rv = [CompressedNodeSet::new(0); Self::MAX_EDGE_COUNT];
//         iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
//             let mut i = 0;
//             let mut bitset = 0;
//             while i < edge_size {
//                 bitset |= 1 << edge[i];
//                 i = i + 1;
//             }
//             rv[edge_idx] = CompressedNodeSet::new(bitset);
//         });
//         rv
//     };
//
//     //     const EDGE_MAP: Self::EdgeMapType = const {
//     //         let mut rv = [0; 1 << $order];
//     //         iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
//     //             let mut i = 0;
//     //             let mut bitset = 0;
//     //             while i < edge_size {
//     //                 bitset |= 1 << edge[i];
//     //                 i = i + 1;
//     //             }
//     //             rv[bitset] = edge_idx as u8;
//     //         });
//     //         rv
//     //     };
//     //
//     //     const FULL_OVERLAPS: Self::FullOverlapsType = const {
//     //         let mut rv_raw = [!0; Self::MAX_EDGE_COUNT];
//     //         iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
//     //             let mut i = 0;
//     //             let mut bitset = 0u32;
//     //             while i < edge_size {
//     //                 bitset |= 1 << edge[i];
//     //                 i = i + 1;
//     //             }
//     //             bitset = !bitset & ((1 << $order) - 1);
//     //             while bitset != 0 {
//     //                 let node = bitset.trailing_zeros() as usize;
//     //                 bitset &= !(1 << node);
//     //                 rv_raw[edge_idx] &= !Self::ADJ[node].container;
//     //             }
//     //         });
//     //
//     //         let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
//     //         let mut i = 0;
//     //         while i < Self::MAX_EDGE_COUNT {
//     //             rv[i] = Self::new(rv_raw[i] & ((1 << Self::MAX_EDGE_COUNT) - 1));
//     //             i += 1;
//     //         }
//     //         rv
//     //     };
//     //
//     //     const PART_OVERLAPS: Self::PartOverlapsType = const {
//     //         let adj = Self::ADJ;
//     //         let mut rv_raw = [0; Self::MAX_EDGE_COUNT];
//     //         iter_hyperedges!($order, 1..=$order, |edge, edge_size, edge_idx| {
//     //             let mut i = 0;
//     //             while i < edge_size {
//     //                 rv_raw[edge_idx] |= adj[edge[i]].container;
//     //                 i = i + 1;
//     //             }
//     //         });
//     //
//     //         let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
//     //         let mut i = 0;
//     //         while i < Self::MAX_EDGE_COUNT {
//     //             rv[i] = Self::new(rv_raw[i] & ((1 << Self::MAX_EDGE_COUNT) - 1));
//     //             i += 1;
//     //         }
//     //         rv
//     //     };
//     //
//     //     const INCLUSION_MAP: Self::InclusionMapType = const {
//     //         let mut rv_raw = [0; Self::MAX_EDGE_COUNT];
//     //         iter_hyperedges!($order, 1..=$order, |_edge, _edge_size, edge_idx| {
//     //             let mut iter = Self::FULL_OVERLAPS[edge_idx].container;
//     //             while iter != 0 {
//     //                 let inner = iter.trailing_zeros() as usize;
//     //                 iter &= !(1 << inner);
//     //                 rv_raw[inner] |= 1 << edge_idx;
//     //             }
//     //         });
//     //
//     //         let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
//     //         let mut i = 0;
//     //         while i < Self::MAX_EDGE_COUNT {
//     //             rv[i] = Self::new(rv_raw[i]);
//     //             i += 1;
//     //         }
//     //         rv
//     //     };
//     //
//     //     const EDGE_FILTER_BITMASK: Self::EdgeFilterBitmaskType = const {
//     //         let mut rv_raw = [0; Self::MAX_EDGE_COUNT];
//     //
//     //         let mut shift_offset = 0;
//     //         let mut i = 1;
//     //
//     //         while i <= $order {
//     //             let curr_count = max_hyperedge_count($order, i, i);
//     //
//     //             rv_raw[i] = ((1 << curr_count) - 1) << shift_offset;
//     //
//     //             shift_offset += curr_count;
//     //             i += 1;
//     //         }
//     //
//     //         let mut rv = [Self::new(0); Self::MAX_EDGE_COUNT];
//     //         let mut i = 0;
//     //         while i < Self::MAX_EDGE_COUNT {
//     //             rv[i] = Self::new(rv_raw[i]);
//     //             i += 1;
//     //         }
//     //         rv
//     //     };
//     //
//     //     const RELABELING_MAP: Self::RelabelingMap = const {
//     //         let node_map = Self::NODE_MAP;
//     //         let edge_map = Self::EDGE_MAP;
//     //
//     //         let mut relabeling_map = [[0u8; Self::MAX_EDGE_COUNT]; factorial($order)];
//     //
//     //         let mut i = 0;
//     //         while i < factorial($order) {
//     //             let perm = BinPerm::from_usize(i).decode::<$order>();
//     //             let mut j = 0;
//     //
//     //             while j < Self::MAX_EDGE_COUNT {
//     //                 let mut old_nodes = node_map[j].nodes;
//     //                 let mut new_nodes = 0u8;
//     //
//     //                 while old_nodes != 0 {
//     //                     let old_node = old_nodes.trailing_zeros() as usize;
//     //                     old_nodes &= !(1 << old_node);
//     //
//     //                     let new_node = perm[old_node];
//     //                     new_nodes |= 1 << new_node;
//     //                 }
//     //
//     //                 relabeling_map[i][j] = edge_map[new_nodes as usize];
//     //
//     //                 j += 1;
//     //             }
//     //             i += 1;
//     //         }
//     //
//     //         relabeling_map
//     //     };
//     // }
// }
