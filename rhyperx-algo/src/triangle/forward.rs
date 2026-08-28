use std::borrow::Cow;
use std::ops::Deref;

use crate::triangle::cbs::hcbs::HCBSGraph;

use crate::misc::{
    OrderAndPos, common_neighbors_sorted_list_by_key, count_common_neighbors_sorted_list,
    degeneracy_ordering, degree_ordering,
};
use crate::types::adj_list::AdjList;
use crate::types::adj_list::common::Undirected;
use crate::types::adj_list::traits::Incidence;
use crate::types::{EdgeId, NodeId};

pub struct Triangle<'a, W, I: Incidence> {
    pub nodes: [NodeId; 3],
    pub edges: [I::EdgeType; 3],
    pub weights: [&'a W; 3],
}

/// Computes the degree ordering: (order, position)

#[cfg(feature = "bindings")]
#[pyo3::pymodule(submodule)]
pub mod forward {
    use pyo3::prelude::*;
    use pyo3_stub_gen::derive::gen_stub_pyfunction;
    use pyo3_stub_gen::reexport_module_members;

    use crate::types::adj_list::{PyAdjList, PyUndirectedAdjList};

    #[pyfunction]
    #[gen_stub_pyfunction(module = "rust_core._core.triangle.forward")]
    pub fn forward(adj: PyUndirectedAdjList, sort_degrees: bool) -> usize {
        match adj {
            PyUndirectedAdjList::Weighted(g) => super::forward(&g, sort_degrees),
            PyUndirectedAdjList::Unweighted(g) => super::forward(&g, sort_degrees),
        }
    }

    #[pyfunction]
    #[gen_stub_pyfunction(module = "rust_core._core.triangle.forward")]
    pub fn forward_hashed(adj: PyUndirectedAdjList, sort_degrees: bool) -> usize {
        match adj {
            PyUndirectedAdjList::Weighted(g) => super::forward(&g, sort_degrees),
            PyUndirectedAdjList::Unweighted(g) => super::forward(&g, sort_degrees),
        }
    }

    #[pyfunction]
    #[gen_stub_pyfunction(module = "rust_core._core.triangle.forward")]
    pub fn forward_hbs(adj: PyUndirectedAdjList, sort_degrees: bool) -> usize {
        match adj {
            PyUndirectedAdjList::Weighted(g) => super::forward(&g, sort_degrees),
            PyUndirectedAdjList::Unweighted(g) => super::forward(&g, sort_degrees),
        }
    }

    reexport_module_members!("rust_core.triangle.forward" from "rust_core._core.triangle.forward");
}

/// Forward algorithm for triangle counting. If sort_degrees is true, a degree ordering is computed, otherwise edges are processed in
/// the natural order (u < v). Common neighbors are counted with the sorted list strategy
pub fn forward<W, I: Incidence>(adj: &AdjList<W, Undirected, I>, sort_degrees: bool) -> usize {
    let n = adj.n();
    let mut a = vec![Vec::new(); n];
    for i in 0..n {
        a[i].reserve(adj[i].len());
    }

    let mut count = 0;

    if sort_degrees {
        let (OrderAndPos { order, pos, .. }, _) = degree_ordering(adj, true);

        for i in 0..n {
            let u = order[i]; // order usually contains NodeId
            for neighbor in &adj[u] {
                // Using Index trait
                let v = neighbor.node as usize;
                // let w = neighbor.weight.clone();
                if i < pos[v] as usize {
                    // a[u] works if u is usize. If u is NodeId, use u as usize
                    count += count_common_neighbors_sorted_list(&a[u as usize], &a[v]);
                    a[v].push((pos[u as usize] as NodeId, ())); // Cast back to NodeId for storage
                }
            }
        }
    } else {
        for u in 0..n {
            for neighbor in &adj[u] {
                // &(v_node, ref w)
                let v = neighbor.node as usize;
                // let w = neighbor.weight.clone();
                if u < v {
                    count += count_common_neighbors_sorted_list(&a[u], &a[v]);
                    a[v].push((u as NodeId, ()));
                }
            }
        }
    }
    count
}

/// Compact forward/forward hashed algorithm for triangle counting. If sort_degrees is true, a degree ordering is computed, otherwise edges are processed in
/// the natural order (u < v). Common neighbors are counted with the hash map strategy
pub fn forward_hashed<W, I: Incidence>(
    adj: &AdjList<W, Undirected, I>,
    order: Option<(&[NodeId], &[usize])>,
) -> usize {
    let n = adj.n();
    let mut a = vec![Vec::new(); n];
    let mut mark = vec![0usize; n];
    let mut current = 1;
    let mut count = 0;

    let (order, pos) = match order {
        Some((o, p)) => (Cow::Borrowed(o), Cow::Borrowed(p)),
        None => {
            let n = adj.n();
            let natural_order = ((0 as NodeId)..(n as NodeId)).collect::<Vec<_>>();
            let natural_pos = (0..n).collect::<Vec<_>>();
            (Cow::Owned(natural_order), Cow::Owned(natural_pos))
        }
    };

    for i in 0..n {
        let u = order[i] as usize; // Cast once per outer loop

        for neighbor in &adj[u] {
            // &(v_node, ref _w)
            let v = neighbor.node as usize;
            let is_forward = i < pos[v] as usize;

            if is_forward {
                for &w in &a[u] {
                    mark[w as usize] = current;
                }

                for &w in &a[v] {
                    if mark[w as usize] == current {
                        count += 1;
                    }
                }

                current += 1;
                a[v].push(u as NodeId);
            }
        }
    }
    count
}

pub fn forward_hbs<W, I: Incidence>(adj: &AdjList<W, Undirected, I>, sort_degrees: bool) -> usize {
    let n = adj.n();
    let mut a = HCBSGraph::<u128>::with_nodes(n);

    // Optimization: Pre-reserve
    for i in 0..n {
        a.nodes[i].bits.reserve(adj[i].len());
        a.nodes[i].offsets.reserve(adj[i].len());
    }

    let mut count = 0;

    if sort_degrees {
        let (OrderAndPos { order, pos, .. }, _) = degree_ordering(adj, true);

        for i in 0..n {
            let u = order[i];
            let u_idx = u as usize;
            for neighbor in &adj[u_idx] {
                let v = neighbor.node as usize;
                if i < pos[v] as usize {
                    count += a.count_common_neighbors(u_idx, v);
                    a.append_neighbor(v as NodeId, pos[u_idx] as NodeId);
                }
            }
        }
    } else {
        for u in 0..n {
            for neighbor in &adj[u] {
                let v = neighbor.node as usize;
                if u < v {
                    count += a.count_common_neighbors(u, v);
                    a.append_neighbor(v as NodeId, u as NodeId);
                }
            }
        }
    }
    count
}

/// the order parameter specifies order and position array for the nodes. A vertex degree order or
/// degeneracy order can be used. If None, the natural order is used
pub fn forward_hashed_cloj<W, I, F>(
    adj: &AdjList<W, Undirected, I>,
    order: Option<&OrderAndPos>,
    mut cloj: F,
) where
    F: FnMut(NodeId, NodeId, NodeId),
    I: Incidence,
{
    let n = adj.n();
    let mut a = vec![Vec::new(); n];
    let mut mark = vec![0usize; n];
    let mut current = 1;

    let (order, pos) = match order {
        Some(OrderAndPos { order, pos, .. }) => (Cow::Borrowed(order), Cow::Borrowed(pos)),
        None => {
            let n = adj.n();
            let natural_order = ((0 as NodeId)..(n as NodeId)).collect::<Vec<_>>();
            let natural_pos = (0..n).collect::<Vec<_>>();
            (Cow::Owned(natural_order), Cow::Owned(natural_pos))
        }
    };

    for i in 0..n {
        let u = order[i] as usize;

        for neighbor in &adj[u] {
            let v = neighbor.node as usize;
            let is_forward = i < pos[v] as usize;

            if is_forward {
                for &w in &a[u] {
                    mark[w as usize] = current;
                }

                for &w in &a[v] {
                    if mark[w as usize] == current {
                        cloj(u as NodeId, v as NodeId, w);
                    }
                }

                current += 1;
                a[v].push(u as NodeId);
            }
        }
    }
}

/// the order parameter specifies order and position array for the nodes. A vertex degree order or
/// degeneracy order can be used. If None, the natural order is used
///
/// this differs from forward_hashed_cloj in that it sorts the adjacency lists of each node
/// according to the order and position arrays, which can improve cache locality and also allows to
/// return edge ids as well without excessive hashing overhead
pub fn forward_sorted_cloj<W, I, F>(
    adj: &mut AdjList<W, Undirected, I>,
    order: Option<&OrderAndPos>,
    mut cloj: F,
) where
    F: FnMut(&AdjList<W, Undirected, I>, Triangle<W, I>),
    I: Incidence,
{
    let n = adj.n();

    let (order, pos) = match order {
        Some(OrderAndPos { order, pos, .. }) => (Cow::Borrowed(order), Cow::Borrowed(pos)),
        None => {
            let n = adj.n();
            let natural_order = ((0 as NodeId)..(n as NodeId)).collect::<Vec<_>>();
            let natural_pos = (0..n).collect::<Vec<_>>();
            (Cow::Owned(natural_order), Cow::Owned(natural_pos))
        }
    };

    for (u, neighbors) in adj.iter_neighbors_mut().enumerate() {
        neighbors.sort_unstable_by_key(|v| pos[v.node as usize]);
    }
    let forward_start = {
        let mut rv = Vec::with_capacity(n);
        for (u, neighbors) in adj.iter_neighbors().enumerate() {
            rv.push(neighbors.partition_point(|n| pos[n.node as usize] < pos[u]))
        }
        rv
    };
    let mut backward_sizes = vec![0; n];

    for i in 0..n {
        let u = order[i] as usize;

        for neighbor in &adj[u][forward_start[u]..] {
            let v = neighbor.node as usize;

            common_neighbors_sorted_list_by_key(
                &adj[u][..backward_sizes[u]],
                &adj[v][..backward_sizes[v]],
                |n| pos[n.node as usize],
                |i, j| {
                    let common = adj[u][i].node;
                    cloj(
                        adj,
                        Triangle {
                            nodes: [u as NodeId, v as NodeId, common],
                            edges: [neighbor.edge, adj[u][i].edge, adj[v][j].edge],
                            weights: [&neighbor.weight, &adj[u][i].weight, &adj[v][j].weight],
                        },
                    );
                },
            );

            backward_sizes[v] += 1;
        }
    }
}
