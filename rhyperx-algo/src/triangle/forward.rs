use rhyperx_core::graph::{
    IndexedNeighborEntry, IndexedNeighbors, IndexedNeighborsMut, Undirected,
};
use rhyperx_core::misc::order::{Order, Pos};
use rhyperx_core::types::NodeId;

use crate::misc::common_neighbors::{
    common_neighbors_sorted_list_by_key, count_common_neighbors_sorted_list,
};
use crate::misc::sorting::degree_ordering;
use crate::triangle::cbs::hcbs::HCBSGraph;

pub struct Triangle<'a, N: NodeId, W> {
    pub nodes: [N; 3],
    pub weights: [&'a W; 3],
}

/// Forward algorithm for triangle counting. If sort_degrees is true, a degree ordering is computed, otherwise edges are processed in
/// the natural order (u < v). Common neighbors are counted with the sorted list strategy
pub fn forward<G>(adj: &G, sort_degrees: bool) -> usize
where
    G: IndexedNeighbors<Dir = Undirected>,
{
    let n = adj.n();
    let mut a = vec![Vec::new(); n];
    for (i, a_i) in a.iter_mut().enumerate() {
        a_i.reserve(adj.degree(G::NodeIdType::from_usize(i)));
    }

    let mut count = 0;

    if sort_degrees {
        let (order_pos, _) = degree_ordering(adj, true);

        for i in 0..n {
            let u = order_pos.order[i].as_usize();
            let u_id = G::NodeIdType::from_usize(u);
            for v in adj.neighbors(u_id) {
                let v = v.node().as_usize();
                if i < order_pos.pos[v] {
                    count += count_common_neighbors_sorted_list(&a[u], &a[v]);
                    a[v].push(order_pos.pos[u]);
                }
            }
        }
    } else {
        for u in 0..n {
            let u_id = G::NodeIdType::from_usize(u);
            for v in adj.neighbors(u_id) {
                let v = v.node().as_usize();
                if u < v {
                    count += count_common_neighbors_sorted_list(&a[u], &a[v]);
                    a[v].push(u);
                }
            }
        }
    }
    count
}

/// Compact forward/forward hashed algorithm for triangle counting. If sort_degrees is true, a degree ordering is computed, otherwise edges are processed in
/// the natural order (u < v). Common neighbors are counted with the hash map strategy
pub fn forward_hashed<G>(adj: &G, order: Option<(&Order<G::NodeIdType>, &Pos)>) -> usize
where
    G: IndexedNeighbors<Dir = Undirected>,
{
    let n = adj.n();
    let mut a = vec![Vec::new(); n];
    let mut mark = vec![0usize; n];
    let mut current = 1;
    let mut count = 0;

    let node_at = |i: usize| match order {
        Some((order, _)) => order[i].as_usize(),
        None => i,
    };
    let pos_of = |v: usize| match order {
        Some((_, pos)) => pos[v],
        None => v,
    };

    for i in 0..n {
        let u = node_at(i);
        let u_id = G::NodeIdType::from_usize(u);

        for v in adj.neighbors(u_id) {
            let v = v.node().as_usize();
            let is_forward = i < pos_of(v);

            if is_forward {
                for &w in &a[u] {
                    mark[w] = current;
                }

                for &w in &a[v] {
                    if mark[w] == current {
                        count += 1;
                    }
                }

                current += 1;
                a[v].push(u);
            }
        }
    }
    count
}

pub fn forward_hbs<G>(adj: &G, sort_degrees: bool) -> usize
where
    G: IndexedNeighbors<Dir = Undirected>,
{
    let n = adj.n();
    let mut a = HCBSGraph::<u128>::with_nodes(n);

    for i in 0..n {
        let deg = adj.degree(G::NodeIdType::from_usize(i));
        a.nodes[i].bits.reserve(deg);
        a.nodes[i].offsets.reserve(deg);
    }

    let mut count = 0;

    if sort_degrees {
        let (order_pos, _) = degree_ordering(adj, true);

        for i in 0..n {
            let u = order_pos.order[i].as_usize();
            let u_id = G::NodeIdType::from_usize(u);
            for v in adj.neighbors(u_id) {
                let v = v.node().as_usize();
                if i < order_pos.pos[v] {
                    count += a.count_common_neighbors(u, v);
                    a.append_neighbor(v, order_pos.pos[u]);
                }
            }
        }
    } else {
        for u in 0..n {
            let u_id = G::NodeIdType::from_usize(u);
            for v in adj.neighbors(u_id) {
                let v = v.node().as_usize();
                if u < v {
                    count += a.count_common_neighbors(u, v);
                    a.append_neighbor(v, u);
                }
            }
        }
    }
    count
}

/// the order parameter specifies order and position array for the nodes. A vertex degree order or
/// degeneracy order can be used. If None, the natural order is used
pub fn forward_hashed_cloj<G, F>(adj: &G, order: Option<(&Order<G::NodeIdType>, &Pos)>, mut cloj: F)
where
    G: IndexedNeighbors<Dir = Undirected>,
    F: FnMut(G::NodeIdType, G::NodeIdType, G::NodeIdType),
{
    let n = adj.n();
    let mut a = vec![Vec::new(); n];
    let mut mark = vec![0usize; n];
    let mut current = 1;

    let node_at = |i: usize| match order {
        Some((order, _)) => order[i].as_usize(),
        None => i,
    };
    let pos_of = |v: usize| match order {
        Some((_, pos)) => pos[v],
        None => v,
    };

    for i in 0..n {
        let u = node_at(i);
        let u_id = G::NodeIdType::from_usize(u);

        for v in adj.neighbors(u_id) {
            let v = v.node().as_usize();
            let is_forward = i < pos_of(v);

            if is_forward {
                for &w in &a[u] {
                    mark[w] = current;
                }

                for &w in &a[v] {
                    if mark[w] == current {
                        cloj(
                            G::NodeIdType::from_usize(u),
                            G::NodeIdType::from_usize(v),
                            G::NodeIdType::from_usize(w),
                        );
                    }
                }

                current += 1;
                a[v].push(u);
            }
        }
    }
}

/// the order parameter specifies order and position array for the nodes. A vertex degree order or
/// degeneracy order can be used. If None, the natural order is used
///
/// this differs from forward_hashed_cloj in that it sorts the adjacency lists of each node
/// according to the order and position arrays, which can improve cache locality
///
/// The weights of the three edges of the triangle are passed to the closure.
pub fn forward_sorted_cloj<G, F>(
    adj: &mut G,
    order: Option<(&Order<G::NodeIdType>, &Pos)>,
    mut cloj: F,
) where
    G: IndexedNeighborsMut<Dir = Undirected>,
    F: for<'a> FnMut([G::NodeIdType; 3], [&'a G::WeightType; 3]),
{
    let n = adj.n();

    let node_at = |i: usize| match order {
        Some((order, _)) => order[i].as_usize(),
        None => i,
    };
    let pos_of = |v: usize| match order {
        Some((_, pos)) => pos[v],
        None => v,
    };

    for u in 0..n {
        let u_id = G::NodeIdType::from_usize(u);
        adj.neighbors_mut(u_id)
            .sort_unstable_by_key(|v| pos_of(v.node().as_usize()));
    }

    let forward_start: Vec<usize> = (0..n)
        .map(|u| {
            let u_id = G::NodeIdType::from_usize(u);
            let pu = pos_of(u);
            adj.neighbors(u_id)
                .partition_point(|v| pos_of(v.node().as_usize()) < pu)
        })
        .collect();
    let mut backward_sizes = vec![0; n];

    for i in 0..n {
        let u = node_at(i);
        let u_id = G::NodeIdType::from_usize(u);
        let u_neighbors = adj.neighbors(u_id);

        for v_entry in &u_neighbors[forward_start[u]..] {
            let v = v_entry.node().as_usize();
            let v_id = G::NodeIdType::from_usize(v);
            let v_neighbors = adj.neighbors(v_id);

            common_neighbors_sorted_list_by_key(
                &u_neighbors[..backward_sizes[u]],
                &v_neighbors[..backward_sizes[v]],
                |n| pos_of(n.node().as_usize()),
                |i, j| {
                    let nodes = [G::NodeIdType::from_usize(u), v_id, u_neighbors[i].node()];
                    let weights = [
                        v_entry.weight(),
                        u_neighbors[i].weight(),
                        v_neighbors[j].weight(),
                    ];
                    cloj(nodes, weights);
                },
            );

            backward_sizes[v] += 1;
        }
    }
}
