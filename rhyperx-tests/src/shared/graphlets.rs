use std::sync::LazyLock;

use rust_core::types::{Hx, Hypergraph, NodeId, NodeWeight, hyperadj_list::HyperAdjList};

type RvPairW = (Hypergraph<NodeId, NodeWeight>, HyperAdjList<NodeWeight>);
type RvPairUW = (Hypergraph<NodeId, ()>, HyperAdjList<()>);

/// 1. Path Graph (P_4): A linear chain of 4 nodes.
/// Edges: 0-1, 1-2, 2-3
pub const PATH4_W: LazyLock<RvPairW> = LazyLock::new(|| {
    let mut hg = Hypergraph::new();
    hg.extend_with_edges(vec![
        Hx::new_unchecked([0, 1], 1.0),
        Hx::new_unchecked([1, 2], 1.0),
        Hx::new_unchecked([2, 3], 1.0),
    ]);
    let adj = HyperAdjList::from_hypergraph_unmapped(hg.clone());
    (hg, adj)
});

/// 2. Star 4: One central hub node connected to 3 peripheral leaves.
/// Edges: 0-1, 0-2, 0-3
pub const STAR4_W: LazyLock<RvPairW> = LazyLock::new(|| {
    let mut hg = Hypergraph::new();
    hg.extend_with_edges(vec![
        Hx::new_unchecked([0, 1], 1.0),
        Hx::new_unchecked([0, 2], 1.0),
        Hx::new_unchecked([0, 3], 1.0),
    ]);
    let adj = HyperAdjList::from_hypergraph_unmapped(hg.clone());
    (hg, adj)
});

/// 3. Cycle Graph (C_4): A single closed loop of 4 nodes.
/// Edges: 0-1, 1-2, 2-3, 3-0
pub const C4_W: LazyLock<RvPairW> = LazyLock::new(|| {
    let mut hg = Hypergraph::new();
    hg.extend_with_edges(vec![
        Hx::new_unchecked([0, 1], 1.0),
        Hx::new_unchecked([1, 2], 1.0),
        Hx::new_unchecked([2, 3], 1.0),
        Hx::new_unchecked([3, 0], 1.0),
    ]);
    let adj = HyperAdjList::from_hypergraph_unmapped(hg.clone());
    (hg, adj)
});

/// 4. Paw: A triangle (0, 1, 2) with a tail attached to one vertex.
/// Edges: 0-1, 1-2, 2-0, 2-3
pub const PAW_W: LazyLock<RvPairW> = LazyLock::new(|| {
    let mut hg = Hypergraph::new();
    hg.extend_with_edges(vec![
        Hx::new_unchecked([0, 1], 1.0),
        Hx::new_unchecked([1, 2], 1.0),
        Hx::new_unchecked([2, 0], 1.0),
        Hx::new_unchecked([2, 3], 1.0),
    ]);
    let adj = HyperAdjList::from_hypergraph_unmapped(hg.clone());
    (hg, adj)
});

/// 5. Diamond Graph: A C_4 cycle with one extra internal diagonal chord.
/// Edges: 0-1, 0-2, 1-2, 1-3, 2-3 (Missing only 0-3)
pub const DIAMOND_W: LazyLock<RvPairW> = LazyLock::new(|| {
    let mut hg = Hypergraph::new();
    hg.extend_with_edges(vec![
        Hx::new_unchecked([0, 1], 1.0),
        Hx::new_unchecked([0, 2], 1.0),
        Hx::new_unchecked([1, 2], 1.0),
        Hx::new_unchecked([1, 3], 1.0),
        Hx::new_unchecked([2, 3], 1.0),
    ]);
    let adj = HyperAdjList::from_hypergraph_unmapped(hg.clone());
    (hg, adj)
});

/// 6. Complete Graph (K_4): Fully connected structure.
/// Edges: All 6 possible node pairs
pub const K4_W: LazyLock<RvPairW> = LazyLock::new(|| {
    let mut hg = Hypergraph::new();
    hg.extend_with_edges(vec![
        Hx::new_unchecked([0, 1], 1.0),
        Hx::new_unchecked([0, 2], 1.0),
        Hx::new_unchecked([0, 3], 1.0),
        Hx::new_unchecked([1, 2], 1.0),
        Hx::new_unchecked([1, 3], 1.0),
        Hx::new_unchecked([2, 3], 1.0),
    ]);
    let adj = HyperAdjList::from_hypergraph_unmapped(hg.clone());
    (hg, adj)
});

pub const STD_HG: LazyLock<RvPairW> = LazyLock::new(|| {
    let mut hg = Hypergraph::new();
    hg.extend_with_edges(vec![
        Hx::new_unchecked([0, 1], 1.0),
        Hx::new_unchecked([0, 2], 1.0),
        // Hx::new_unchecked([0, 3], 1.0),
        // Hx::new_unchecked([1, 2], 1.0),
        // Hx::new_unchecked([1, 3], 1.0),
        // Hx::new_unchecked([2, 3], 1.0),
    ]);

    hg.extend_with_edges(vec![
        Hx::new_unchecked([0, 1, 2], 1.0),
        Hx::new_unchecked([1, 2, 3], 1.0),
        Hx::new_unchecked([0, 1, 3], 1.0),
        Hx::new_unchecked([0, 2, 3], 1.0),
        Hx::new_unchecked([2, 3, 4], 1.0),
    ]);
    let adj = HyperAdjList::from_hypergraph_unmapped(hg.clone());
    (hg, adj)
});
