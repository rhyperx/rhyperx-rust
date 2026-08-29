use num_traits::{AsPrimitive, Float};

use rhyperx_core::graph::{IndexedNeighbors, IndexedNeighborsMut, Undirected};
use rhyperx_core::misc::order::Order;
use rhyperx_core::types::{EdgeId, NodeId};

use crate::misc::sorting::sort_by_degree;

pub struct Cycle4<'a, N: NodeId, W, E: EdgeId> {
    pub nodes: [N; 4],
    pub edges: [E; 4],
    pub weights: [&'a W; 4],
}

// Orient the adj list such that:
//
// Sort the adjacency list N +(v) of each vertex v, such that the neighborhood
// begins with N −(v) (in arbitrary order), and then is followed by N +(v) (sorted in order of ≺).
fn count_c4_base<G>(adj: &G, order: &Order<G::NodeIdType>) -> usize
where
    G: IndexedNeighbors<Dir = Undirected>,
{
    let n_less_count = (0..adj.n())
        .map(|v| {
            let v_id = G::NodeIdType::from_usize(v);
            let v_deg = adj.degree(v_id);
            (0..v_deg)
                .filter(|&idx| {
                    let neighbor = adj.neighbor_node(v_id, idx);
                    let neighbor_deg = adj.degree(neighbor);
                    neighbor_deg < v_deg || (neighbor_deg == v_deg && neighbor < v_id)
                })
                .count()
        })
        .collect::<Vec<usize>>();

    let mut c_4_count = 0;
    let mut l = vec![0; adj.n()];

    for i in 0..adj.n() {
        let x_id = order[i];
        let x = x_id.as_usize();
        for j in 0..n_less_count[x] {
            let y = adj.neighbor_node(x_id, j);
            let mut k = 0;
            loop {
                let z = adj.neighbor_node(y, k).as_usize();
                if z == x {
                    break;
                }
                c_4_count += l[z];

                l[z] += 1;
                k += 1;
            }
        }

        for j in 0..n_less_count[x] {
            let y = adj.neighbor_node(x_id, j);
            let mut k = 0;
            loop {
                let z = adj.neighbor_node(y, k).as_usize();
                if z == x {
                    break;
                }
                l[z] = 0;
                k += 1;
            }
        }
    }
    c_4_count
}

/// Paul Burkhardt and David G. Harris 4-cycle heuristic
///
/// A mut ref is required for adj list neighbors sorting
pub fn count_c4<G>(adj: &mut G) -> usize
where
    G: IndexedNeighborsMut<Dir = Undirected>,
{
    let (order_pos, _degeneracy) = sort_by_degree(adj, false);
    count_c4_base(adj, &order_pos.order)
}

/// Paul Burkhardt and David G. Harris 4-cycle heuristic
///
/// The implementation assumes that the adjacency list is already sorted by degree, so no
/// preprocessing is required. If the adjacency list is not sorted, use `count_c_4()` instead,
/// otherwise the result will be incorrect.
pub fn count_c4_no_sort<G>(adj: &G, order: &Order<G::NodeIdType>) -> usize
where
    G: IndexedNeighbors<Dir = Undirected>,
{
    count_c4_base(adj, order)
}

/// Paul Burkhardt and David G. Harris 4-cycle heuristic
///
/// The implementation assumes that the adjacency list is already sorted by degree, so no
/// preprocessing is required. If the adjacency list is not sorted, use `count_c_4()` instead,
/// otherwise the result will be incorrect.
fn intensity_c4_base<G>(adj: &G, order: &Order<G::NodeIdType>) -> (usize, f64)
where
    G: IndexedNeighbors<Dir = Undirected>,
    G::WeightType: Float + AsPrimitive<f64>,
{
    let n_less_count = (0..adj.n())
        .map(|v| {
            let v_id = G::NodeIdType::from_usize(v);
            let v_deg = adj.degree(v_id);
            (0..v_deg)
                .filter(|&idx| {
                    let neighbor = adj.neighbor_node(v_id, idx);
                    let neighbor_deg = adj.degree(neighbor);
                    neighbor_deg < v_deg || (neighbor_deg == v_deg && neighbor < v_id)
                })
                .count()
        })
        .collect::<Vec<usize>>();

    let mut c_4_count = 0;
    let mut c_4_intensity = 0.0;

    let mut l_count = vec![0; adj.n()];
    let mut l_intensity = vec![0.0; adj.n()];

    for i in 0..adj.n() {
        let x_id = order[i];
        let x = x_id.as_usize();
        for j in 0..n_less_count[x] {
            let y = adj.neighbor_node(x_id, j);
            let w_xy = *adj.neighbor_weight(x_id, j);

            let mut k = 0;
            loop {
                let z = adj.neighbor_node(y, k).as_usize();
                let w_yz = *adj.neighbor_weight(y, k);
                if z == x {
                    break;
                }
                c_4_count += l_count[z];
                l_count[z] += 1;

                let curr_part_intensity = (w_xy * w_yz).as_().powf(1.0 / 4.0);
                c_4_intensity += curr_part_intensity * l_intensity[z];
                l_intensity[z] += curr_part_intensity;

                k += 1;
            }
        }

        for j in 0..n_less_count[x] {
            let y = adj.neighbor_node(x_id, j);
            let mut k = 0;
            loop {
                let z = adj.neighbor_node(y, k).as_usize();
                if z == x {
                    break;
                }

                l_count[z] = 0;
                l_intensity[z] = 0.0;
                k += 1;
            }
        }
    }
    (c_4_count, c_4_intensity / c_4_count as f64)
}

/// Paul Burkhardt and David G. Harris 4-cycle heuristic
///
/// The implementation assumes that the adjacency list is already sorted by degree, so no
/// preprocessing is required.
pub fn intensity_c4<G>(adj: &mut G) -> (usize, f64)
where
    G: IndexedNeighborsMut<Dir = Undirected>,
    G::WeightType: Float + AsPrimitive<f64>,
{
    let (order_pos, _degeneracy) = sort_by_degree(adj, false);
    intensity_c4_base(adj, &order_pos.order)
}

/// Paul Burkhardt and David G. Harris 4-cycle heuristic
///
/// The implementation assumes that the adjacency list is already sorted by degree, so no
/// preprocessing is required.
pub fn intensity_c4_no_sort<G>(adj: &G, order: &Order<G::NodeIdType>) -> (usize, f64)
where
    G: IndexedNeighbors<Dir = Undirected>,
    G::WeightType: Float + AsPrimitive<f64>,
{
    intensity_c4_base(adj, order)
}

// Just to pair with the escape_4 algorithm; they also compute the number of path4 in c4, crucial
// for weighted escape_4 algorithm

/// Paul Burkhardt and David G. Harris 4-cycle heuristic
///
/// The implementation assumes that the adjacency list is already sorted by degree, so no
/// preprocessing is required. If the adjacency list is not sorted, use `count_c_4()` instead,
/// otherwise the result will be incorrect.
fn intensity_c4_subinc_base<G>(adj: &G, order: &Order<G::NodeIdType>) -> (usize, f64, f64)
where
    G: IndexedNeighbors<Dir = Undirected>,
    G::WeightType: Float + AsPrimitive<f64>,
{
    let n_less_count = (0..adj.n())
        .map(|v| {
            let v_id = G::NodeIdType::from_usize(v);
            let v_deg = adj.degree(v_id);
            (0..v_deg)
                .filter(|&idx| {
                    let neighbor = adj.neighbor_node(v_id, idx);
                    let neighbor_deg = adj.degree(neighbor);
                    neighbor_deg < v_deg || (neighbor_deg == v_deg && neighbor < v_id)
                })
                .count()
        })
        .collect::<Vec<usize>>();

    let mut c4_count = 0;
    let mut c4_intensity = 0.0;
    let mut p4_intensity = 0.0;

    let mut l_count = vec![0; adj.n()];
    let mut l_intensity_4 = vec![0.0; adj.n()];
    let mut l_intensity_3 = vec![0.0; adj.n()];
    let mut l_low_int = vec![0.0; adj.n()];
    let mut l_high_int = vec![0.0; adj.n()];

    for i in 0..adj.n() {
        let x_id = order[i];
        let x = x_id.as_usize();
        for j in 0..n_less_count[x] {
            let y = adj.neighbor_node(x_id, j);
            let w_xy = *adj.neighbor_weight(x_id, j);

            let mut k = 0;
            loop {
                let z = adj.neighbor_node(y, k).as_usize();
                let w_yz = *adj.neighbor_weight(y, k);
                if z == x {
                    break;
                }
                c4_count += l_count[z];
                l_count[z] += 1;

                let intensity_4 = (w_xy * w_yz).as_().powf(1.0 / 4.0);
                c4_intensity += intensity_4 * l_intensity_4[z];

                let low_int = w_xy.as_().powf(1.0 / 3.0);
                let high_int = w_yz.as_().powf(1.0 / 3.0);
                let intensity_3 = low_int * high_int;
                p4_intensity += intensity_3 * (l_low_int[z] + l_high_int[z])
                    + l_intensity_3[z] * (low_int + high_int);

                l_intensity_3[z] += intensity_3;
                l_intensity_4[z] += intensity_4;
                l_low_int[z] += low_int;
                l_high_int[z] += high_int;

                k += 1;
            }
        }

        for j in 0..n_less_count[x] {
            let y = adj.neighbor_node(x_id, j);
            let mut k = 0;
            loop {
                let z = adj.neighbor_node(y, k).as_usize();
                if z == x {
                    break;
                }

                l_count[z] = 0;
                l_intensity_3[z] = 0.0;
                l_intensity_4[z] = 0.0;
                l_low_int[z] = 0.0;
                l_high_int[z] = 0.0;
                k += 1;
            }
        }
    }

    (
        c4_count,
        c4_intensity / c4_count.max(1) as f64,
        p4_intensity / 4.0 / c4_count.max(1) as f64,
    )
}

/// Paul Burkhardt and David G. Harris 4-cycle heuristic
///
/// The implementation assumes that the adjacency list is already sorted by degree, so no
/// preprocessing is required.
pub fn intensity_c4_subinc<G>(adj: &mut G) -> (usize, f64, f64)
where
    G: IndexedNeighborsMut<Dir = Undirected>,
    G::WeightType: Float + AsPrimitive<f64>,
{
    let (order_pos, _degeneracy) = sort_by_degree(adj, false);
    intensity_c4_subinc_base(adj, &order_pos.order)
}

/// Paul Burkhardt and David G. Harris 4-cycle heuristic
///
/// The implementation assumes that the adjacency list is already sorted by degree, so no
/// preprocessing is required.
pub fn intensity_c4_subinc_no_sort<G>(adj: &G, order: &Order<G::NodeIdType>) -> (usize, f64, f64)
where
    G: IndexedNeighbors<Dir = Undirected>,
    G::WeightType: Float + AsPrimitive<f64>,
{
    intensity_c4_subinc_base(adj, order)
}
