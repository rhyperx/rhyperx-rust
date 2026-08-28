use crate::CompactMotif;
type CompactMotif3 = CompactMotif!(3);
type CompactMotif4 = CompactMotif!(4);
type CompactMotif5 = CompactMotif!(5);

pub const TRIANGLE: CompactMotif3 = {
    let mut rv = CompactMotif3::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([0, 2]);
    rv.const_add_edge_with_nodes([1, 2]);
    rv
};

pub const STRAIGHT_PATH: CompactMotif3 = {
    let mut rv = CompactMotif3::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([1, 2]);
    rv
};

// 4-node connected graphlets (6 types)
pub const FOUR_CLIQUE: CompactMotif4 = {
    let mut rv = CompactMotif4::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([0, 2]);
    rv.const_add_edge_with_nodes([0, 3]);
    rv.const_add_edge_with_nodes([1, 2]);
    rv.const_add_edge_with_nodes([1, 3]);
    rv.const_add_edge_with_nodes([2, 3]);
    rv
};

pub const DIAMOND: CompactMotif4 = {
    let mut rv = CompactMotif4::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([1, 2]);
    rv.const_add_edge_with_nodes([2, 3]);
    rv.const_add_edge_with_nodes([0, 3]);
    rv.const_add_edge_with_nodes([1, 3]);
    rv
};

pub const FOUR_CYCLE: CompactMotif4 = {
    let mut rv = CompactMotif4::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([1, 2]);
    rv.const_add_edge_with_nodes([2, 3]);
    rv.const_add_edge_with_nodes([3, 0]);
    rv
};

pub const PAW: CompactMotif4 = {
    let mut rv = CompactMotif4::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([1, 2]);
    rv.const_add_edge_with_nodes([2, 0]); // Triangle
    rv.const_add_edge_with_nodes([2, 3]); // Tail
    rv
};

pub const PATH_4: CompactMotif4 = {
    let mut rv = CompactMotif4::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([1, 2]);
    rv.const_add_edge_with_nodes([2, 3]);
    rv
};

pub const STAR_4: CompactMotif4 = {
    let mut rv = CompactMotif4::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([0, 2]);
    rv.const_add_edge_with_nodes([0, 3]);
    rv
};

// Disconnected 4-node motifs
pub const TWO_EDGES_DISCONNECTED: CompactMotif4 = {
    let mut rv = CompactMotif4::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([2, 3]);
    rv
};

pub const TAILED_TRIANGLE: CompactMotif4 = {
    let mut rv = CompactMotif4::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([0, 2]);
    rv.const_add_edge_with_nodes([1, 2]);
    rv.const_add_edge_with_nodes([2, 3]);
    rv
};

pub const PATH_3_PLUS_ISOLATED: CompactMotif4 = {
    let mut rv = CompactMotif4::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv.const_add_edge_with_nodes([1, 2]);
    rv
};

pub const EDGE_PLUS_TWO_ISOLATED: CompactMotif4 = {
    let mut rv = CompactMotif4::zero();
    rv.const_add_edge_with_nodes([0, 1]);
    rv
};

pub const FOUR_ISOLATED: CompactMotif4 = CompactMotif4::zero();
