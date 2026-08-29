use rhyperx_core::graph::{IndexedNeighborsMut, NeighborRetrieval};
use rhyperx_core::hypergraph::static_adj_list::StaticAdjList;
use rhyperx_core::misc::order::{Order, OrderAndPos, Other, Pos};
use rhyperx_core::types::{EdgeId, NodeId};

/// Returns a degree ordering of the vertices, the position of each vertex in that ordering, and
/// the maximum degree of the graph.
/// Time Complexity: O(n)
pub fn degree_ordering<G: NeighborRetrieval>(
    g: &G,
    decreasing: bool,
) -> (OrderAndPos<G::NodeIdType, Other>, usize) {
    let n = g.n();
    if n == 0 {
        return (OrderAndPos::empty(), 0);
    }

    let deg: Vec<usize> = (0..n)
        .map(|v| g.degree(G::NodeIdType::from_usize(v)))
        .collect();
    let max_deg = *deg.iter().max().unwrap_or(&0);

    let mut bin_count = vec![0; max_deg + 1];
    for &d in &deg {
        bin_count[d] += 1;
    }

    let mut start_pos = 0;
    let mut bin_starts = vec![0; max_deg + 1];
    for d in 0..=max_deg {
        bin_starts[d] = start_pos;
        start_pos += bin_count[d];
    }

    let mut order = vec![G::NodeIdType::zero(); n];
    let mut pos = vec![0; n];

    if decreasing {
        for v in (0..n).rev() {
            let d = deg[v];
            pos[v] = bin_starts[d];
            order[bin_starts[d]] = G::NodeIdType::from_usize(v);
            bin_starts[d] += 1;
        }
    } else {
        for v in 0..n {
            let d = deg[v];
            pos[v] = bin_starts[d];
            order[bin_starts[d]] = G::NodeIdType::from_usize(v);
            bin_starts[d] += 1;
        }
    }

    (OrderAndPos::new(Order::new(order), Pos::new(pos)), max_deg)
}

/// Sorts the neighbors of each vertex in the adjacency list by the following conditions:
/// u ≺ v if deg(u) < deg(v); if deg(u) = deg(v) the tie breaker is arbitrary
///
/// Time Complexity: O(e log d), where e is the number of edges and d is the maximum degree.
pub fn sort_by_degree<G>(
    adj: &mut G,
    _decreasing: bool,
) -> (OrderAndPos<G::NodeIdType, Other>, usize)
where
    G: IndexedNeighborsMut,
{
    let (order_pos, max_deg) = degree_ordering(adj, false);

    for v in 0..adj.n() {
        let v_id = G::NodeIdType::from_usize(v);
        adj.sort_neighbors_by_key(v_id, |node, _weight| order_pos.pos[node.as_usize()]);
    }

    (order_pos, max_deg)
}

/// Returns a degeneracy ordering of the graph, the position of each vertex,
/// and the degeneracy (k) of the graph.
/// Complexity: O(n + m)
pub fn degeneracy_ordering<G: NeighborRetrieval>(
    g: &G,
) -> (OrderAndPos<G::NodeIdType, Other>, usize) {
    let n = g.n();
    if n == 0 {
        return (OrderAndPos::empty(), 0);
    }

    let mut deg: Vec<usize> = (0..n)
        .map(|v| g.degree(G::NodeIdType::from_usize(v)))
        .collect();
    let max_deg = *deg.iter().max().unwrap_or(&0);

    let mut bin_count = vec![0; max_deg + 1];
    for &d in &deg {
        bin_count[d] += 1;
    }

    let mut bin_starts = vec![0; max_deg + 1];
    let mut start_pos = 0;
    for d in 0..=max_deg {
        bin_starts[d] = start_pos;
        start_pos += bin_count[d];
    }

    let mut temp_starts = bin_starts.clone();
    let mut order = vec![G::NodeIdType::zero(); n];
    let mut pos = vec![0; n];
    for v in 0..n {
        pos[v] = temp_starts[deg[v]];
        order[pos[v]] = G::NodeIdType::from_usize(v);
        temp_starts[deg[v]] += 1;
    }

    let mut k = 0;
    macro_rules! decrease_node {
        ($node:expr) => {{
            unsafe {
                let n = $node;
                let u_deg = *deg.get_unchecked(n);
                let u_pos = *pos.get_unchecked(n);

                let first_node_pos = *bin_starts.get_unchecked(u_deg);
                let first_node = *order.get_unchecked(first_node_pos);
                let first_node_idx = first_node.as_usize();

                if first_node_idx != n {
                    let tmp = *pos.get_unchecked(first_node_idx);
                    *pos.get_unchecked_mut(first_node_idx) = *pos.get_unchecked(n);
                    *pos.get_unchecked_mut(n) = tmp;

                    let tmp = *order.get_unchecked(first_node_pos);
                    *order.get_unchecked_mut(first_node_pos) = *order.get_unchecked(u_pos);
                    *order.get_unchecked_mut(u_pos) = tmp;
                }

                *bin_starts.get_unchecked_mut(u_deg) += 1;
                *deg.get_unchecked_mut(n) -= 1;
            }
        }};
    }

    for i in 0..n {
        let v = order[i].as_usize();
        k = std::cmp::max(k, deg[v]);

        for u in g.iter_neighbors(G::NodeIdType::from_usize(v)) {
            let u = u.as_usize();
            if pos[u] > i {
                decrease_node!(u);
                decrease_node!(v);
            }
        }
    }

    (OrderAndPos::new(Order::new(order), Pos::new(pos)), k)
}

/// Returns a degeneracy ordering of the hypergraph, the position of each vertex,
/// and the degeneracy (k) of the hypergraph.
/// Complexity: O(n + m)
pub fn hyper_degeneracy_ordering<N: NodeId, E: EdgeId, W>(
    adj: &StaticAdjList<N, E, W>,
) -> (OrderAndPos<N, Other>, usize) {
    let n = adj.n();
    if n == 0 {
        return (OrderAndPos::empty(), 0);
    }

    let mut deg: Vec<usize> = (0..n)
        .map(|v| adj.count_incident(N::from_usize(v)))
        .collect();
    let max_deg = *deg.iter().max().unwrap_or(&0);

    let mut bin_count = vec![0; max_deg + 1];
    for &d in &deg {
        bin_count[d] += 1;
    }

    let mut bin_starts = vec![0; max_deg + 1];
    let mut start_pos = 0;
    for d in 0..=max_deg {
        bin_starts[d] = start_pos;
        start_pos += bin_count[d];
    }

    let mut temp_starts = bin_starts.clone();
    let mut order = vec![N::zero(); n];
    let mut pos = vec![0; n];
    for v in 0..n {
        pos[v] = temp_starts[deg[v]];
        order[pos[v]] = N::from_usize(v);
        temp_starts[deg[v]] += 1;
    }

    let mut peeled = vec![false; adj.m()];
    let mut k = 0;
    for i in 0..n {
        let v_id = order[i];
        let v = v_id.as_usize();
        k = std::cmp::max(k, deg[v]);

        for (edge_id, edge) in adj.iter_incident_edges(v_id) {
            let edge_id = edge_id.as_usize();
            if peeled[edge_id] {
                continue;
            }
            peeled[edge_id] = true;

            for &n_node in edge.nodes {
                let u = n_node.as_usize();
                let u_deg = deg[u];
                let u_pos = pos[u];

                let first_node_pos = bin_starts[u_deg];
                let first_node = order[first_node_pos];
                let first_node_idx = first_node.as_usize();

                if u != first_node_idx {
                    pos.swap(u, first_node_idx);
                    order.swap(u_pos, first_node_pos);
                }

                bin_starts[u_deg] += 1;
                deg[u] -= 1;
            }
        }
    }

    (OrderAndPos::new(Order::new(order), Pos::new(pos)), k)
}
