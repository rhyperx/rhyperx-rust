use crate::{
    misc::{OrderAndPos, degeneracy_ordering},
    types::adj_list::{
        adj_list::AdjListBase,
        traits::{AdjConfig, NeighborContainer},
    },
};

#[cfg(feature = "bindings")]
#[pyo3::pymodule]
pub mod kclist {
    use pyo3::pyfunction;
    use pyo3_stub_gen::derive::gen_stub_pyfunction;
    use pyo3_stub_gen::reexport_module_members;

    use crate::types::adj_list::{PyAdjList, PyDirectedAdjList, PyUndirectedAdjList};

    #[pyfunction]
    #[gen_stub_pyfunction(module = "rust_core._core.triangle.kclist")]
    pub fn kclist(adj: PyAdjList) -> usize {
        match adj {
            PyAdjList::Undirected(g) => match g {
                PyUndirectedAdjList::Weighted(g) => super::kclist(&g),
                PyUndirectedAdjList::Unweighted(g) => super::kclist(&g),
            },
            PyAdjList::Directed(g) => match g {
                PyDirectedAdjList::Weighted(g) => super::kclist(&g),
                PyDirectedAdjList::Unweighted(g) => super::kclist(&g),
            },
        }
    }

    reexport_module_members!("rust_core.triangle.kclist" from "rust_core._core.triangle.kclist");
}

/// Counts the number of triangles in the graph using the Chiba-Nishizeki algorithm.
///
/// Complexity: O(m * k) where k is the degeneracy.
pub fn kclist<C: AdjConfig>(adj: &AdjListBase<C>) -> usize {
    let n = adj.n();
    if n < 3 {
        return 0;
    }

    // 1. Get the degeneracy ordering
    let (OrderAndPos { order, pos , ..}, _) = degeneracy_ordering(adj);

    // 2. Re-orient edges: only keep edges u -> v where pos[u] < pos[v]
    // This creates a Directed Acyclic Graph (DAG)
    let mut out_adj: Vec<Vec<usize>> = vec![vec![]; n];
    for u in 0..n {
        for neighbor in adj[u].iter_neighbors() {
            let v = *neighbor.node as usize;
            if pos[u] < pos[v] {
                out_adj[u].push(v);
            }
        }
    }

    // 3. Triangle counting
    let mut count = 0;
    // We use usize::MAX as the "unmarked" value since 0 is a valid vertex ID
    let mut marks = vec![usize::MAX; n];

    for &u in &order {
        // Mark all neighbors of u
        for &v in &out_adj[u as usize] {
            marks[v] = u as usize;
        }

        // Check neighbors of neighbors
        for &v in &out_adj[u as usize] {
            for &w in &out_adj[v] {
                if marks[w] == u as usize {
                    count += 1;
                }
            }
        }
    }

    count
}

// #[allow(unused)]
// /// A version of kclist that accepts Python objects.
// pub fn kclist_py(adj_py: Bound<'_, PyList>) -> PyResult<usize> {
//     let n = adj_py.len();
//     if n < 3 {
//         return Ok(0);
//     }
//
//     // Build an AdjList from the Python adjacency list
//     let mut al = AdjList::with_nodes(n);
//     for u in 0..n {
//         let neighbors = adj_py.get_item(u)?.cast_into::<PyList>()?;
//         for v_any in neighbors.iter() {
//             let v = v_any.extract::<usize>()?;
//             al.adj[u].push(v as NodeId);
//         }
//     }
//
//     // 1. Get the degeneracy ordering using the Rust implementation
//     let (order, pos, _) = degeneracy_ordering(&al);
//
//     // 2. Re-orient edges: only keep edges u -> v where pos[u] < pos[v]
//     let mut out_adj: Vec<Vec<usize>> = vec![vec![]; n];
//
//     for u in 0..n {
//         for &v_node in &al.adj[u] {
//             let v = v_node as usize;
//             if pos[u] < pos[v] {
//                 out_adj[u].push(v);
//             }
//         }
//     }
//
//     // 3. Triangle counting
//     let mut count = 0;
//     let mut marks = vec![usize::MAX; n];
//
//     for &u in &order {
//         // Mark all "out-neighbors" of u
//         for &v in &out_adj[u] {
//             marks[v] = u;
//         }
//
//         // Check neighbors of neighbors
//         for &v in &out_adj[u] {
//             for &w in &out_adj[v] {
//                 if marks[w] == u {
//                     count += 1;
//                 }
//             }
//         }
//     }
//
//     Ok(count)
// }
