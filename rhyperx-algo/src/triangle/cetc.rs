use rhyperx_core::graph::{AdjList, IndexedNeighborEntry, IndexedNeighbors, Undirected};
use rhyperx_core::types::NodeId;

use crate::misc::common_neighbors::common_neighbors_sorted_list;
use crate::misc::sorting::degeneracy_ordering;
use crate::misc::traversal::bfs;
use crate::triangle::forward::forward_hashed;

/// Computes intersection of two sorted vectors and returns the common elements.
pub fn cetc<G>(adj: &G) -> usize
where
    G: IndexedNeighbors<Dir = Undirected>,
    G::Neighbor: Ord,
{
    let n = adj.n();
    let mut count = 0;

    // adj.sort_neighbors();
    let levels = bfs(adj);

    for u in 0..n {
        let u_id = G::NodeIdType::from_usize(u);
        for v in adj.neighbors(u_id) {
            let v = v.node();
            let v_idx = v.as_usize();
            // Check levels and use u < v to avoid double counting
            if levels[v_idx] == levels[u] && u < v_idx {
                let common = common_neighbors_sorted_list(adj.neighbors(u_id), adj.neighbors(v));
                for w in common {
                    // Triangle (u, v, w) logic
                    if levels[w.node().as_usize()] != levels[u] || v_idx < w.node().as_usize() {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

pub fn cetc_s<G>(adj: &G) -> usize
where
    G: IndexedNeighbors<Dir = Undirected>,
{
    let n = adj.n();
    let mut adj0 = vec![vec![]; n];
    let mut adj1 = vec![vec![]; n];
    let mut hash = vec![false; n];
    let mut count = 0;

    let levels = bfs(adj);

    // Partition edges based on levels
    for u in 0..n {
        let u_id = G::NodeIdType::from_usize(u);
        for v in adj.neighbors(u_id) {
            let v = v.node();
            if levels[u] == levels[v.as_usize()] {
                adj0[u].push(v);
            } else {
                adj1[u].push(v);
            }
        }
    }

    // Reuse the compact_forward from previous implementation
    // Build an AdjList from adj0 to call forward_hashed
    let mut al0: AdjList<G::NodeIdType, (), Undirected> = AdjList::with_nodes(n);
    for (u, neighbors) in adj0.iter().enumerate() {
        for &v in neighbors {
            al0.insert_edge(G::NodeIdType::from_usize(u), v, ());
        }
    }
    let (order_pos, _degeneracy) = degeneracy_ordering(&al0);
    count += forward_hashed(&al0, Some((&order_pos.order, &order_pos.pos)));

    for u in 0..n {
        if adj1[u].is_empty() {
            continue;
        }

        // Standard hash-based intersection logic
        for &v in &adj1[u] {
            hash[v.as_usize()] = true;
        }

        for &v in &adj0[u] {
            let v = v.as_usize();
            if u < v {
                for &w in &adj1[v] {
                    let w = w.as_usize();
                    if hash[w] {
                        count += 1;
                    }
                }
            }
        }

        // Clean up hash for the next iteration
        for &v in &adj1[u] {
            hash[v.as_usize()] = false;
        }
    }

    count
}
