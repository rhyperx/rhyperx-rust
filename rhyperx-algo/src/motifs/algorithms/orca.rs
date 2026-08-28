use super::const_graphlets::*;
use foldhash::fast::FixedState;

use crate::{
    misc::{common_neighbors_sorted_list_3_cloj, degeneracy_ordering},
    motifs::{
        compressed_motif::{CompactMotif, CompactMotif3},
        compressed_node_set::CompressedNodeSet,
        fingerprint::{Fingerprint3, Fingerprint4},
        types::MotifStats,
    },
    types::{
        EdgeId, NodeId, NodeWeight,
        adj_list::{
            AdjList, AdjSet, Undirected, WithIncidence, WithoutIncidence, common::Neighbor,
            traits::NeighborContainer,
        },
        hypergraph::{Hypergraph, HypergraphAccessor},
    },
};

use crate::triangle::forward::forward_hashed_cloj;

use hashbrown::HashMap;

#[derive(Clone, Default, Debug)]
struct Equation4<T> {
    f_12_14: T,
    f_10_13: T,
    f_13_14: T,
    f_11_13: T,
    f_7_11: T,
    f_5_8: T,
    f_6_9: T,
    f_9_12: T,
    f_4_8: T,
    f_8_12: T,
    f_14: T,
}

impl<T> Equation4<T>
where
    T: num_traits::Zero,
{
    pub fn new() -> Self {
        Self {
            f_12_14: T::zero(),
            f_10_13: T::zero(),
            f_13_14: T::zero(),
            f_11_13: T::zero(),
            f_7_11: T::zero(),
            f_5_8: T::zero(),
            f_6_9: T::zero(),
            f_9_12: T::zero(),
            f_4_8: T::zero(),
            f_8_12: T::zero(),
            f_14: T::zero(),
        }
    }
}

pub fn unweighted_3(hg: &Hypergraph<NodeId, ()>) -> HashMap<Fingerprint3, MotifStats> {
    let mut motif_stats = HashMap::new();
    let mut triangles = MotifStats::new();
    let mut straight_paths = MotifStats::new();

    let edges_2: Vec<(NodeId, NodeId, ())> = hg
        .edges::<2>()
        .iter()
        .cloned()
        .map(|e| (e.nodes[0], e.nodes[1], ()))
        .collect();

    // Undirected by design
    let (mut adj, direct_map, inverse_map) =
        AdjList::<(), Undirected, WithIncidence>::from_edges_mapped(edges_2);
    let adj_hash: AdjSet<(), Undirected, WithoutIncidence> = adj.clone().into();

    // adj.make_undirected();

    // let mut adj_hash: Vec<HashMap<NodeId, (), FixedState>> = adj
    //     .adj
    //     .iter()
    //     .cloned()
    //     .map(|neighboors| neighboors.into_iter().collect())
    //     .collect();

    let (mut order_pos, degeneracy) = degeneracy_ordering(&adj);
    order_pos.reverse();

    forward_hashed_cloj(&adj, Some(&order_pos), |a, b, c| {
        triangles.count += 1;
    });

    let mut tot_2_edges_motifs_count = 0;
    for neighboors in adj.iter_neighbors() {
        tot_2_edges_motifs_count += neighboors.len() * (neighboors.len() - 1) / 2;
    }
    straight_paths.count = tot_2_edges_motifs_count - 3 * triangles.count;

    let triangle_fingeprint = TRIANGLE.fingerprint();
    let straight_path_fingerprint = STRAIGHT_PATH.fingerprint();

    motif_stats.insert(triangle_fingeprint, triangles);
    motif_stats.insert(straight_path_fingerprint, straight_paths);

    for edge_e in hg.edges::<3>().iter() {
        let (a, b, c) = (
            inverse_map.get(&edge_e.nodes[0]),
            inverse_map.get(&edge_e.nodes[1]),
            inverse_map.get(&edge_e.nodes[2]),
        );
        let mut inner_count = 0;
        let mut inner_edges = [CompressedNodeSet::new(0); 3];

        if let (Some(a), Some(b)) = (a, b)
            && adj_hash[*a as usize].contains_key(b)
        {
            inner_edges[inner_count] = CompressedNodeSet::from_array([0, 1]);
            inner_count += 1;
        }
        if let (Some(a), Some(c)) = (a, c)
            && adj_hash[*a as usize].contains_key(c)
        {
            inner_edges[inner_count] = CompressedNodeSet::from_array([0, 2]);
            inner_count += 1;
        }
        if let (Some(b), Some(c)) = (b, c)
            && adj_hash[*b as usize].contains_key(c)
        {
            inner_edges[inner_count] = CompressedNodeSet::from_array([1, 2]);
            inner_count += 1;
        }

        let motif = {
            let mut rv = CompactMotif::<3>::zero();
            rv.add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2]));
            for i in 0..inner_count {
                rv.add_edge_with_nodes(inner_edges[i]);
            }
            rv
        };

        motif_stats
            .entry(motif.fingerprint())
            .or_insert_with(MotifStats::new)
            .count += 1;

        if inner_count == 2 {
            motif_stats
                .get_mut(&straight_path_fingerprint)
                .unwrap()
                .count -= 1;
        }
        if inner_count == 3 {
            motif_stats.get_mut(&triangle_fingeprint).unwrap().count -= 1;
        }
    }

    motif_stats
}

pub fn weighted_3(hg: &Hypergraph<NodeId, NodeWeight>) -> HashMap<Fingerprint3, MotifStats> {
    let mut motif_stats = HashMap::new();
    let mut triangles = MotifStats::new();
    let mut straight_paths = MotifStats::new();

    let edges_2: Vec<(NodeId, NodeId, NodeWeight)> = hg
        .edges::<2>()
        .iter()
        .cloned()
        .map(|e| (e.nodes[0], e.nodes[1], e.weight))
        .collect();

    let (mut adj, direct_map, inverse_map) =
        AdjList::<NodeWeight, Undirected, WithIncidence>::from_edges_mapped(edges_2);
    let adj_hash: AdjSet<NodeWeight, Undirected, WithoutIncidence> = adj.clone().into();

    // Computing the sum of products of all possible 3 pairs of incident pairs for each vertex using
    // Newtoon sum O(n + e)
    let mut s1: Vec<f64> = vec![0.0; adj.n()];
    let mut s2: Vec<f64> = vec![0.0; adj.n()];
    // let mut s3: Vec<f64> = vec![0.0; adj.adj.len()];

    for edge in hg.edges::<2>().iter() {
        s1[inverse_map[&edge.nodes[0]] as usize] += edge.weight.sqrt() as f64;
        s1[inverse_map[&edge.nodes[1]] as usize] += edge.weight.sqrt() as f64;

        s2[inverse_map[&edge.nodes[0]] as usize] += edge.weight as f64;
        s2[inverse_map[&edge.nodes[1]] as usize] += edge.weight as f64;
    }

    straight_paths.mean_intensity = {
        let mut tot_2_intensity = 0.0;
        for i in 0..adj.n() {
            tot_2_intensity += (s1[i] * s1[i] - s2[i]) as f64 / 2.0;
        }
        tot_2_intensity
    };

    straight_paths.count = {
        let mut tot_2_count = 0;
        for neighboors in adj.iter_neighbors() {
            tot_2_count += neighboors.len() * (neighboors.len() - 1) / 2;
        }
        tot_2_count
    };

    let (mut order_pos, degeneracy) = degeneracy_ordering(&adj);
    order_pos.reverse();

    forward_hashed_cloj(&adj, Some(&order_pos), |a, b, c| {
        let w_ab = adj_hash[a].get(&b).unwrap().0 as f64;
        let w_ac = adj_hash[a].get(&c).unwrap().0 as f64;
        let w_bc = adj_hash[b].get(&c).unwrap().0 as f64;

        triangles.count += 1;
        straight_paths.count -= 3;
        triangles.mean_intensity += (w_ab * w_ac * w_bc).cbrt();
        straight_paths.mean_intensity -=
            (w_ab * w_ac).sqrt() + (w_ab * w_bc).sqrt() + (w_ac * w_bc).sqrt();
    });

    let triangle_fingeprint = TRIANGLE.fingerprint();
    let straight_path_fingerprint = STRAIGHT_PATH.fingerprint();

    // 3-edge counting
    for edge_e in hg.edges::<3>().iter() {
        let (a, b, c) = (
            inverse_map.get(&edge_e.nodes[0]),
            inverse_map.get(&edge_e.nodes[1]),
            inverse_map.get(&edge_e.nodes[2]),
        );
        let mut inner_count = 0;
        let mut inner_inensity = 1.0;
        let mut inner_edges = [CompressedNodeSet::new(0); 3];
        let mut inner_weights = [0.0; 3];

        if let (Some(a), Some(b)) = (a, b)
            && adj_hash[*a as usize].contains_key(b)
        {
            inner_edges[inner_count] = CompressedNodeSet::from_array([0, 1]);
            inner_weights[inner_count] = adj_hash[*a].get(b).unwrap().0 as f64;

            inner_inensity *= inner_weights[inner_count];
            inner_count += 1;
        }
        if let (Some(a), Some(c)) = (a, c)
            && adj_hash[*a as usize].contains_key(c)
        {
            inner_edges[inner_count] = CompressedNodeSet::from_array([0, 2]);
            inner_weights[inner_count] = adj_hash[*a].get(c).unwrap().0 as f64;

            inner_inensity *= inner_weights[inner_count];
            inner_count += 1;
        }
        if let (Some(b), Some(c)) = (b, c)
            && adj_hash[*b as usize].contains_key(c)
        {
            inner_edges[inner_count] = CompressedNodeSet::from_array([1, 2]);
            inner_weights[inner_count] = adj_hash[*b].get(c).unwrap().0 as f64;

            inner_inensity *= inner_weights[inner_count];
            inner_count += 1;
        }

        let motif = {
            let mut rv = CompactMotif::<3>::zero();
            rv.add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2]));
            for i in 0..inner_count {
                rv.add_edge_with_nodes(inner_edges[i]);
            }
            rv
        };

        let curr_stats = motif_stats
            .entry(motif.fingerprint())
            .or_insert_with(MotifStats::new);
        curr_stats.count += 1;
        curr_stats.mean_intensity +=
            (inner_inensity * edge_e.weight as f64).powf(1.0 / (inner_count as f64 + 1.0));

        if inner_count == 2 {
            straight_paths.count -= 1;
            straight_paths.mean_intensity -= (inner_weights[0] * inner_weights[1]).sqrt();
        }
        if inner_count == 3 {
            triangles.mean_intensity -=
                (inner_weights[0] * inner_weights[1] * inner_weights[2]).cbrt();
            triangles.count -= 1;
        }
    }

    motif_stats.insert(triangle_fingeprint, triangles);
    motif_stats.insert(straight_path_fingerprint, straight_paths);

    for (_, stats) in motif_stats.iter_mut() {
        stats.mean_intensity /= stats.count as f64;
    }

    motif_stats
}

pub fn unweighted_4(hg: &Hypergraph<NodeId, ()>) -> HashMap<Fingerprint4, MotifStats> {
    let mut motif_stats = HashMap::new();

    // Extract 2-edges (regular edges) and build adjacency list
    let edges_2: Vec<(NodeId, NodeId, ())> = hg
        .edges::<2>()
        .iter()
        .cloned()
        .map(|e| (e.nodes[0], e.nodes[1], ()))
        .collect();

    let (mut adj, direct_map, inverse_map) =
        AdjList::<(), Undirected, WithIncidence>::from_edges_mapped(edges_2);
    let adj_hash: AdjSet<(), Undirected, WithIncidence> = adj.clone().into();
    adj.sort_neighbors();

    // let mut inc_list = adj.clone();
    // for u in 0..adj.n() {
    //     for v in inc_list[u] {}
    // }

    // Create hash-based adjacency for O(1) edge lookups
    // adj_hash[i] = HashMap<(neighbor_node_id, edge_id)>
    // let mut adj_hash: Vec<HashMap<NodeId, NodeId, FixedState>> = adj
    //     .adj
    //     .iter()
    //     .cloned()
    //     .map(|neighbors| neighbors.into_iter().map(|e| (e.0, e.1.0)).collect())
    //     .collect();

    // Initialize motif stats
    let mut four_clique = MotifStats::new();
    let mut diamond = MotifStats::new();
    let mut four_cycle = MotifStats::new();
    let mut paw = MotifStats::new();
    let mut path_4 = MotifStats::new();
    let mut star_4 = MotifStats::new();
    let mut two_edges_disconnected = MotifStats::new();
    let mut triangle_plus_isolated = MotifStats::new();
    let mut path_3_plus_isolated = MotifStats::new();
    let mut edge_plus_two_isolated = MotifStats::new();
    let mut four_isolated = MotifStats::new();

    let mut orbit = vec![[0.0; 15]; hg.n()];
    let mut equations: Vec<Equation4<usize>> = vec![Equation4::new(); hg.n()];
    let mut tri = vec![0; hg.m()];

    // Count triangles with forward hashed in O(m^1.5)
    let mut triangles = Vec::new();
    let (mut order_pos, degeneracy) = degeneracy_ordering(&adj);
    order_pos.reverse();

    forward_hashed_cloj(&adj, Some(&order_pos), |a, b, c| {
        triangles.push((a, b, c));

        let edge_ab = adj_hash[a][&b].1 as usize;
        let edge_ac = adj_hash[a][&c].1 as usize;
        let edge_bc = adj_hash[b][&c].1 as usize;

        tri[edge_ab] += 1;
        tri[edge_ac] += 1;
        tri[edge_bc] += 1;

        let upper_bound = Neighbor::new(a.min(b).min(c), (), edge_ab as EdgeId);

        // 4-clique counting
        common_neighbors_sorted_list_3_cloj(&adj[a], &adj[b], &adj[c], &upper_bound, |i, j, k| {
            let common = adj[a][i].node;

            orbit[a as usize][14] += 1.0;
            orbit[b as usize][14] += 1.0;
            orbit[c as usize][14] += 1.0;
            orbit[common as usize][14] += 1.0;
        });
    });

    // Wedge base
    for a in 0..hg.n() {
        let deg_a = adj[a].len();
        // These variables contain the sum over all possible 3 nodes motifs incident on x (wedges
        // and triangles).
        // let (mut sum_p_ab, mut sum_p_ba) = (0, 0);

        // for &(b, (edge_ab, _)) in adj[a].iter_neighbors() {
        for n in adj[a].iter_neighbors() {
            let (b, edge_ab) = (*n.node as usize, *n.edge as usize);
            let deg_b = adj[b].len();
            let (p_ab, p_ba) = (deg_b - 1 - tri[edge_ab], deg_a - 1 - tri[edge_ab]);

            // sum_p_ab += p_ab;
            // sum_p_ba += p_ba;

            equations[a].f_5_8 += p_ab;
            equations[a].f_7_11 += p_ba;
            equations[a].f_6_9 += p_ab * (p_ab - 1);

            equations[a].f_4_8 += p_ab * (p_ab - 1);
        }

        equations[a].f_5_8 = equations[a].f_5_8 * (deg_a - 1);
        equations[a].f_7_11 = equations[a].f_7_11 * (deg_a - 1) - (deg_a * (deg_a - 1));
        // equations[a].f_6_9 += (deg_a - 1) * sum_p_ba;
    }

    // Triangle base
    for (a, b, c) in triangles {
        let edge_ab = adj_hash[a][&b].1 as usize;
        let edge_ac = adj_hash[a][&c].1 as usize;
        let edge_bc = adj_hash[b][&c].1 as usize;

        let deg_a = adj[a].len();
        let deg_b = adj[b].len();
        let deg_c = adj[c].len();

        let (p_ab, p_ba) = (deg_b - 1 - tri[edge_ab], deg_a - 1 - tri[edge_ab]);
        let (p_ac, p_ca) = (deg_c - 1 - tri[edge_ac], deg_a - 1 - tri[edge_ac]);
        let (p_bc, p_cb) = (deg_c - 1 - tri[edge_bc], deg_b - 1 - tri[edge_bc]);

        let (a, b, c) = (a as usize, b as usize, c as usize);
        // f12_14
        equations[a].f_12_14 += tri[edge_bc] - 1;
        equations[b].f_12_14 += tri[edge_ac] - 1;
        equations[c].f_12_14 += tri[edge_ab] - 1;

        // f13_14
        equations[a].f_13_14 += (tri[edge_ab] - 1) + (tri[edge_ac] - 1);
        equations[b].f_13_14 += (tri[edge_ab] - 1) + (tri[edge_bc] - 1);
        equations[c].f_13_14 += (tri[edge_ac] - 1) + (tri[edge_bc] - 1);

        // f10_13
        equations[a].f_10_13 += p_bc + p_cb;
        equations[b].f_10_13 += p_ac + p_ca;
        equations[c].f_10_13 += p_ab + p_ba;

        // f11_13
        equations[a].f_11_13 += p_ca + p_ba;
        equations[b].f_11_13 += p_ab + p_cb;
        equations[c].f_11_13 += p_ac + p_bc;

        // Compensating overcounted motifs in the wedge-base step
    }

    // Count 4-cliques: for each triangle, count common neighbors
    // for (a, b, c) in &triangles {
    //     let neighbors_a = &adj_hash[*a as usize];
    //     let neighbors_b = &adj_hash[*b as usize];
    //     let neighbors_c = &adj_hash[*c as usize];
    //
    //     // Find common neighbors of a, b, c
    //     for (d, _) in neighbors_a {
    //         if neighbors_b.contains_key(d) && neighbors_c.contains_key(d) {
    //             four_clique.count += 1;
    //         }
    //     }
    // }
    // Each 4-clique is counted 4 times (once per triangle), so divide by 4
    // four_clique.count /= 4;

    // Count diamonds: for each triangle, count nodes connected to exactly 2 vertices of the triangle
    // for (a, b, c) in &triangles {
    //     let neighbors_a = &adj_hash[*a as usize];
    //     let neighbors_b = &adj_hash[*b as usize];
    //     let neighbors_c = &adj_hash[*c as usize];
    //
    //     // Check all nodes (could optimize by iterating over union of neighbors)
    //     for d in 0..adj.n() as NodeId {
    //         if d == *a || d == *b || d == *c {
    //             continue;
    //         }
    //         let mut connections = 0;
    //         if neighbors_a.contains_key(&d) {
    //             connections += 1;
    //         }
    //         if neighbors_b.contains_key(&d) {
    //             connections += 1;
    //         }
    //         if neighbors_c.contains_key(&d) {
    //             connections += 1;
    //         }
    //
    //         if connections == 2 {
    //             diamond.count += 1;
    //         }
    //     }
    // }
    // // Each diamond is counted 2 times (once per triangle in the diamond), so divide by 2
    // diamond.count /= 2;
    // Count 4-cycles: for each pair of non-adjacent nodes, count common neighbors
    // A 4-cycle is formed by two nodes with exactly 2 common neighbors
    // for u in 0..adj.n() as NodeId {
    //     let neighbors_u = &adj_hash[u as usize];
    //     for (v, _) in neighbors_u {
    //         if *v <= u {
    //             continue; // Process each edge once
    //         }
    //         let neighbors_v = &adj_hash[*v as usize];
    //         let mut common = 0;
    //         // Iterate over smaller neighbor set
    //         if neighbors_u.len() < neighbors_v.len() {
    //             for (w, _) in neighbors_u {
    //                 if *w != *v && neighbors_v.contains_key(w) {
    //                     common += 1;
    //                 }
    //             }
    //         } else {
    //             for (w, _) in neighbors_v {
    //                 if *w != u && neighbors_u.contains_key(w) {
    //                     common += 1;
    //                 }
    //             }
    //         }
    //         if common >= 2 {
    //             // Number of 4-cycles with u,v as opposite vertices = C(common, 2)
    //             four_cycle.count += common * (common - 1) / 2;
    //         }
    //     }
    // }
    //
    // // Count paws: triangle + pendant edge
    // // For each triangle, count nodes connected to exactly 1 vertex of the triangle
    // for (a, b, c) in &triangles {
    //     let neighbors_a = &adj_hash[*a as usize];
    //     let neighbors_b = &adj_hash[*b as usize];
    //     let neighbors_c = &adj_hash[*c as usize];
    //
    //     for d in 0..adj.n() as NodeId {
    //         if d == *a || d == *b || d == *c {
    //             continue;
    //         }
    //         let mut connections = 0;
    //         if neighbors_a.contains_key(&d) {
    //             connections += 1;
    //         }
    //         if neighbors_b.contains_key(&d) {
    //             connections += 1;
    //         }
    //         if neighbors_c.contains_key(&d) {
    //             connections += 1;
    //         }
    //
    //         if connections == 1 {
    //             paw.count += 1;
    //         }
    //     }
    // }
    //
    // // Count paths of length 3 (P4)
    // // For each edge (u,v), count pairs of neighbors (x,y) where x is neighbor of u (not v),
    // // y is neighbor of v (not u), and x != y, and no edge between x and y
    // for u in 0..adj.n() as NodeId {
    //     let neighbors_u = &adj_hash[u as usize];
    //     for (v, _) in neighbors_u {
    //         if *v <= u {
    //             continue; // Process each edge once
    //         }
    //         let neighbors_v = &adj_hash[*v as usize];
    //
    //         // Count neighbors of u excluding v
    //         let mut neighbors_u_excl_v = Vec::new();
    //         for (x, _) in neighbors_u {
    //             if *x != *v {
    //                 neighbors_u_excl_v.push(*x);
    //             }
    //         }
    //
    //         // Count neighbors of v excluding u
    //         let mut neighbors_v_excl_u = Vec::new();
    //         for (y, _) in neighbors_v {
    //             if *y != u {
    //                 neighbors_v_excl_u.push(*y);
    //             }
    //         }
    //
    //         // Count pairs (x,y) with no edge between them
    //         for x in &neighbors_u_excl_v {
    //             for y in &neighbors_v_excl_u {
    //                 if x == y {
    //                     continue;
    //                 }
    //                 // Check if x and y are adjacent
    //                 if !adj_hash[*x as usize].contains_key(y) {
    //                     path_4.count += 1;
    //                 }
    //             }
    //         }
    //     }
    // }
    // // Each P4 is counted twice (once from each end), so divide by 2
    // path_4.count /= 2;
    //
    // // Count stars (K1,3): for each node, choose 3 neighbors
    // let n = adj.n();
    // let degrees: Vec<usize> = adj.adj.iter().map(|n| n.len()).collect();
    //
    // for u in 0..n as NodeId {
    //     let deg = degrees[u as usize];
    //     if deg >= 3 {
    //         star_4.count += deg * (deg - 1) * (deg - 2) / 6;
    //     }
    // }
    // // Subtract stars that are part of larger structures (claw-free)
    // // Each 4-clique contains 4 stars (one centered at each vertex)
    // star_4.count -= 4 * four_clique.count;
    // // Each diamond contains 2 stars (centered at the two degree-3 vertices)
    // star_4.count -= 2 * diamond.count;
    // // Each paw contains 1 star (centered at the triangle vertex with the tail)
    // star_4.count -= paw.count;
    //
    // // Count disconnected motifs
    // // Total number of 4-node subsets = C(n, 4)
    // let total_4_subsets = if n >= 4 {
    //     n * (n - 1) * (n - 2) * (n - 3) / 24
    // } else {
    //     0
    // };
    //
    // // Count connected 4-node motifs
    // let connected_count = four_clique.count
    //     + diamond.count
    //     + four_cycle.count
    //     + paw.count
    //     + path_4.count
    //     + star_4.count;
    //
    // // Two disconnected edges: choose 2 edges that don't share vertices
    // // Total pairs of edges = C(m, 2)
    // let m = adj.m();
    // let total_edge_pairs = m * (m - 1) / 2;
    //
    // // Pairs of edges sharing a vertex = sum over vertices of C(deg(v), 2)
    // let mut edge_pairs_sharing_vertex = 0;
    // for deg in &degrees {
    //     if *deg >= 2 {
    //         edge_pairs_sharing_vertex += deg * (deg - 1) / 2;
    //     }
    // }
    //
    // two_edges_disconnected.count = total_edge_pairs - edge_pairs_sharing_vertex;
    //
    // // Triangle + isolated: number of triangles * (n - 3)
    // triangle_plus_isolated.count = triangles.len() * (n - 3);
    //
    // // Path of length 2 (P3) + isolated: count P3s * (n - 3)
    // // P3 count = sum over vertices of C(deg(v), 2) - 3 * triangles
    // let mut p3_count = 0;
    // for deg in &degrees {
    //     if *deg >= 2 {
    //         p3_count += deg * (deg - 1) / 2;
    //     }
    // }
    // p3_count -= 3 * triangles.len();
    // path_3_plus_isolated.count = p3_count * (n - 3);
    //
    // // Edge + two isolated: m * C(n - 2, 2)
    // if n >= 4 {
    //     edge_plus_two_isolated.count = m * (n - 2) * (n - 3) / 2;
    // }
    //
    // // Four isolated: C(n, 4) - all other 4-node subsets
    // let accounted = connected_count
    //     + two_edges_disconnected.count
    //     + triangle_plus_isolated.count
    //     + path_3_plus_isolated.count
    //     + edge_plus_two_isolated.count;
    // four_isolated.count = total_4_subsets.saturating_sub(accounted);

    // Insert all motifs into the result map
    motif_stats.insert(FOUR_CLIQUE.fingerprint(), four_clique);
    motif_stats.insert(DIAMOND.fingerprint(), diamond);
    motif_stats.insert(FOUR_CYCLE.fingerprint(), four_cycle);
    motif_stats.insert(PAW.fingerprint(), paw);
    motif_stats.insert(PATH_4.fingerprint(), path_4);
    motif_stats.insert(STAR_4.fingerprint(), star_4);
    motif_stats.insert(TWO_EDGES_DISCONNECTED.fingerprint(), two_edges_disconnected);
    motif_stats.insert(TAILED_TRIANGLE.fingerprint(), triangle_plus_isolated);
    motif_stats.insert(PATH_3_PLUS_ISOLATED.fingerprint(), path_3_plus_isolated);
    motif_stats.insert(EDGE_PLUS_TWO_ISOLATED.fingerprint(), edge_plus_two_isolated);
    motif_stats.insert(FOUR_ISOLATED.fingerprint(), four_isolated);

    motif_stats
}

pub fn weighted_4(hg: &Hypergraph<NodeId, NodeWeight>) -> HashMap<Fingerprint4, MotifStats> {
    let mut motif_stats = HashMap::new();
    // TODO: Implement weighted 4-node motif counting
    motif_stats
}

pub fn unweighted_5(hg: &Hypergraph<NodeId, ()>) -> HashMap<Fingerprint4, MotifStats> {
    let mut motif_stats = HashMap::new();
    // TODO: Implement 5-node motif counting
    motif_stats
}

pub fn weighted_5(hg: &Hypergraph<NodeId, NodeWeight>) -> HashMap<Fingerprint4, MotifStats> {
    let mut motif_stats = HashMap::new();
    // TODO: Implement weighted 5-node motif counting
    motif_stats
}
