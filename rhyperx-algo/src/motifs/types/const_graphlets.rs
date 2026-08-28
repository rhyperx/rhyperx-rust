use crate::motifs::{compressed_motif::CompactMotif, compressed_node_set::CompressedNodeSet};

pub const TRIANGLE: CompactMotif<3> = {
    let mut rv = CompactMotif::<3>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 2]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 2]));
    rv
};

pub const STRAIGHT_PATH: CompactMotif<3> = {
    let mut rv = CompactMotif::<3>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 2]));
    rv
};

// 4-node connected graphlets (6 types)
pub const FOUR_CLIQUE: CompactMotif<4> = {
    let mut rv = CompactMotif::<4>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 2]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 3]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 2]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 3]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([2, 3]));
    rv
};

pub const DIAMOND: CompactMotif<4> = {
    let mut rv = CompactMotif::<4>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 2]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([2, 3]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 3]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 3]));
    rv
};

pub const FOUR_CYCLE: CompactMotif<4> = {
    let mut rv = CompactMotif::<4>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 2]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([2, 3]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([3, 0]));
    rv
};

pub const PAW: CompactMotif<4> = {
    let mut rv = CompactMotif::<4>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 2]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([2, 0])); // Triangle
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([2, 3])); // Tail
    rv
};

pub const PATH_4: CompactMotif<4> = {
    let mut rv = CompactMotif::<4>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 2]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([2, 3]));
    rv
};

pub const STAR_4: CompactMotif<4> = {
    let mut rv = CompactMotif::<4>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 2]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 3]));
    rv
};

// Disconnected 4-node motifs
pub const TWO_EDGES_DISCONNECTED: CompactMotif<4> = {
    let mut rv = CompactMotif::<4>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([2, 3]));
    rv
};

pub const TAILED_TRIANGLE: CompactMotif<4> = {
    let mut rv = CompactMotif::<4>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 2]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 2]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([2, 3]));
    rv
};

pub const PATH_3_PLUS_ISOLATED: CompactMotif<4> = {
    let mut rv = CompactMotif::<4>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([1, 2]));
    rv
};

pub const EDGE_PLUS_TWO_ISOLATED: CompactMotif<4> = {
    let mut rv = CompactMotif::<4>::zero();
    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1]));
    rv
};

pub const FOUR_ISOLATED: CompactMotif<4> = { CompactMotif::<4>::zero() };
