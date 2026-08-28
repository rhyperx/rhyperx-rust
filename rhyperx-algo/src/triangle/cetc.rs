use crate::misc::{
    OrderAndPos, bfs, common_neighbors_sorted_list, degeneracy_ordering, degree_ordering,
};
use crate::triangle::forward::forward_hashed;
use crate::types::adj_list::common::{Neighbor, Undirected};
use crate::types::adj_list::traits::{AdjConfig, Incidence, NeighborContainer};
use crate::types::adj_list::{AdjList, AdjSet, WithoutIncidence};

#[cfg(feature = "bindings")]

#[pyo3::pymodule(submodule)]
pub mod cetc {
    use pyo3::prelude::*;
    use pyo3_stub_gen::derive::gen_stub_pyfunction;
    use pyo3_stub_gen::reexport_module_members;

    use crate::types::adj_list::PyUndirectedAdjList;

    #[pyfunction]
    #[gen_stub_pyfunction(module = "rust_core._core.triangle.cetc")]
    pub fn cetc(mut adj: PyUndirectedAdjList) -> usize {
        match adj {
            PyUndirectedAdjList::Weighted(g) => super::cetc(&g),
            PyUndirectedAdjList::Unweighted(g) => super::cetc(&g),
        }
    }

    #[gen_stub_pyfunction(module = "rust_core._core.triangle.cetc")]
    #[pyfunction]
    pub fn cetc_s(adj: PyUndirectedAdjList) -> usize {
        match adj {
            PyUndirectedAdjList::Weighted(g) => super::cetc(&g),
            PyUndirectedAdjList::Unweighted(g) => super::cetc(&g),
        }
    }

    reexport_module_members!("rust_core.triangle.cetc" from "rust_core._core.triangle.cetc");
}

/// Computes intersection of two sorted vectors and returns the common elements.
pub fn cetc<W, I: Incidence>(adj: &AdjList<W, Undirected, I>) -> usize
where
    W: Clone,
{
    let n = adj.n();
    let mut count = 0;

    // adj.sort_neighbors();
    let levels = bfs(adj);

    for u in 0..n {
        for v in adj[u].iter_neighbors() {
            let v = *v.node as usize;
            // Check levels and use u < v to avoid double counting
            if levels[v] == levels[u] && u < v {
                let common = common_neighbors_sorted_list(&adj[u], &adj[v]);
                for w in common {
                    // Triangle (u, v, w) logic
                    if levels[w.node as usize] != levels[u] || v < w.node as usize {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

pub fn cetc_s<W, I: Incidence>(adj: &AdjList<W, Undirected, I>) -> usize
where
    W: Clone,
{
    let n = adj.n();
    let mut adj0 = vec![vec![]; n];
    let mut adj1 = vec![vec![]; n];
    let mut hash = vec![false; n];
    let mut count = 0;

    let levels = bfs(adj);

    // Partition edges based on levels
    for u in 0..n {
        for v in adj[u].clone() {
            if levels[u] == levels[v.node as usize] {
                adj0[u].push(v);
            } else {
                adj1[u].push(v);
            }
        }
    }

    // Reuse the compact_forward from previous implementation
    // Build an AdjList from adj0 to call forward_hashed
    let mut al0 = AdjList::<(), Undirected, WithoutIncidence>::with_nodes(n);
    for u in 0..n {
        for v in adj0[u].clone() {
            al0[u].push(Neighbor::new(v.node, (), ()));
        }
    }
    let (OrderAndPos { order, pos, .. }, degeneracy) = degeneracy_ordering(&al0);
    count += forward_hashed(&al0, Some((&order, &pos)));

    for u in 0..n {
        if adj1[u].is_empty() {
            continue;
        }

        // Standard hash-based intersection logic
        for n in adj1[u].iter_neighbors() {
            hash[*n.node as usize] = true;
        }

        for n in adj0[u].iter_neighbors() {
            let v = *n.node as usize;
            if u < v {
                for w in adj1[v].iter_neighbors() {
                    let w = *w.node as usize;
                    if hash[w] {
                        count += 1;
                    }
                }
            }
        }

        // Clean up hash for the next iteration
        for n in adj1[u].iter_neighbors() {
            hash[*n.node as usize] = false;
        }
    }

    count
}
