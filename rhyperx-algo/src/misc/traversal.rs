use std::collections::VecDeque;

use rhyperx_core::graph::NeighborRetrieval;
use rhyperx_core::types::NodeId;

/// A simple BFS to calculate levels/distances from a starting component.
/// In most CETC contexts, this assumes node 0 is the root or
/// it iterates through all components.
pub fn bfs<G: NeighborRetrieval>(g: &G) -> Vec<i32> {
    let n = g.n();
    let mut levels = vec![-1; n];
    let mut queue = VecDeque::new();

    for i in 0..n {
        if levels[i] == -1 {
            levels[i] = 0;
            queue.push_back(i);
            while let Some(u) = queue.pop_front() {
                for v in g.iter_neighbors(G::NodeIdType::from_usize(u)) {
                    let v = v.as_usize();
                    if levels[v] == -1 {
                        levels[v] = levels[u] + 1;
                        queue.push_back(v);
                    }
                }
            }
        }
    }
    levels
}
