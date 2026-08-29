use rhyperx_core::graph::NeighborRetrieval;
use rhyperx_core::types::NodeId;

use crate::misc::sorting::degeneracy_ordering;

/// Counts the number of triangles in the graph using the Chiba-Nishizeki algorithm.
///
/// Complexity: O(m * k) where k is the degeneracy.
pub fn kclist<G: NeighborRetrieval>(adj: &G) -> usize {
    let n = adj.n();
    if n < 3 {
        return 0;
    }

    // 1. Get the degeneracy ordering
    let (order_pos, _) = degeneracy_ordering(adj);

    // 2. Re-orient edges: only keep edges u -> v where pos[u] < pos[v]
    // This creates a Directed Acyclic Graph (DAG)
    let mut out_adj: Vec<Vec<usize>> = vec![vec![]; n];
    for (u, out) in out_adj.iter_mut().enumerate() {
        let u_id = G::NodeIdType::from_usize(u);
        for v in adj.iter_neighbors(u_id) {
            let v = v.as_usize();
            if order_pos.pos[u] < order_pos.pos[v] {
                out.push(v);
            }
        }
    }

    // 3. Triangle counting
    let mut count = 0;
    // We use usize::MAX as the "unmarked" value since 0 is a valid vertex ID
    let mut marks = vec![usize::MAX; n];

    for &u in order_pos.order.iter() {
        let u = u.as_usize();
        // Mark all neighbors of u
        for &v in &out_adj[u] {
            marks[v] = u;
        }

        // Check neighbors of neighbors
        for &v in &out_adj[u] {
            for &w in &out_adj[v] {
                if marks[w] == u {
                    count += 1;
                }
            }
        }
    }

    count
}
