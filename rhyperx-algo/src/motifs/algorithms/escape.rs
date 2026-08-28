use std::{cmp::max, ops::BitOr};

use bit_set::BitSet;
use hashbrown::{HashMap, HashSet};

use crate::{
    misc::{
        common_neighbors_sorted_list_3_by_key,
        cycle::{count_c4, intensity_c4_subinc},
        degeneracy_ordering,
    },
    motifs::{
        algorithms::const_graphlets::{
            DIAMOND, FOUR_CLIQUE, FOUR_CYCLE, PATH_4, STAR_4, STRAIGHT_PATH, TAILED_TRIANGLE,
            TRIANGLE,
        },
        compressed_motif::CompactMotif,
        compressed_node_set::CompressedNodeSet,
        fingerprint::{Fingerprint3, Fingerprint4},
        types::MotifStats,
    },
    triangle::forward::{forward_hashed_cloj, forward_sorted_cloj},
    types::{
        NodeId, NodeWeight,
        adj_list::{AdjList, AdjSet, Undirected, WithIncidence, WithoutIncidence},
        hyperadj_list::HyperAdjList,
    },
};

pub fn unweighted_3(hg: &HyperAdjList<()>) -> HashMap<Fingerprint3, MotifStats> {
    let mut motif_stats = HashMap::new();
    let mut triangles = MotifStats::new();
    let mut straight_paths = MotifStats::new();

    let edges_2: Vec<(NodeId, NodeId, ())> = hg
        .iter_by_size(2)
        .map(|(_edge_id, edge)| (edge.nodes[0], edge.nodes[1], ()))
        .collect();

    let edge2_set: HashSet<(NodeId, NodeId)> =
        HashSet::from_iter(edges_2.iter().map(|(a, b, _)| (*a, *b)));

    let (adj, _direct_map, _inverse_map) =
        AdjList::<(), Undirected, WithIncidence>::from_edges_mapped(edges_2);

    let (mut order_pos, _degeneracy) = degeneracy_ordering(&adj);
    order_pos.reverse();

    forward_hashed_cloj(&adj, Some(&order_pos), |_a, _b, _c| {
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

    for (_edge_id, edge) in hg.iter_by_size(3) {
        let (a, b, c) = (edge.nodes[0], edge.nodes[1], edge.nodes[2]);
        let mut inner_count = 0;
        let mut inner_edges = [CompressedNodeSet::new(0); 3];

        if edge2_set.contains(&(a, b)) {
            inner_edges[inner_count] = CompressedNodeSet::from_array([0, 1]);
            inner_count += 1;
        }

        if edge2_set.contains(&(a, c)) {
            inner_edges[inner_count] = CompressedNodeSet::from_array([0, 2]);
            inner_count += 1;
        }

        if edge2_set.contains(&(b, c)) {
            inner_edges[inner_count] = CompressedNodeSet::from_array([1, 2]);
            inner_count += 1;
        }

        let motif = {
            let mut rv = const {
                let mut rv = CompactMotif::<3>::zero();
                rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2]));
                rv
            };
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

pub fn weighted_3(hg: &HyperAdjList<NodeWeight>) -> HashMap<Fingerprint3, MotifStats> {
    let mut motif_stats = HashMap::new();
    let mut triangles = MotifStats::new();
    let mut straight_paths = MotifStats::new();

    let edges_2: Vec<(NodeId, NodeId, NodeWeight)> = hg
        .iter_by_size(2)
        .map(|(_edge_id, edge)| (edge.nodes[0], edge.nodes[1], *edge.weight))
        .collect();

    let edge2_set: HashMap<(NodeId, NodeId), NodeWeight> = {
        let mut rv = HashMap::with_capacity(edges_2.len());
        for (a, b, w) in edges_2.iter().cloned() {
            rv.insert((a, b), w);
        }
        rv
    };

    let (adj, _direct_map, _inverse_map) =
        AdjList::<NodeWeight, Undirected, WithIncidence>::from_edges_mapped(edges_2);
    let adj_hash: AdjSet<NodeWeight, Undirected, WithoutIncidence> = adj.clone().into();

    // Computing the sum of products of all possible 3 pairs of incident pairs for each vertex using
    // Newton sum O(n + e)
    for x in 0..adj.n() {
        straight_paths.count += adj[x].len() * (adj[x].len() - 1) / 2;
        let mut s1 = 0.0;
        let mut s2 = 0.0;
        for y in &adj[x] {
            s1 += y.weight.sqrt();
            s2 += y.weight;
        }
        straight_paths.mean_intensity += (s1 * s1 - s2) as f64 / 2.0;
    }

    let (mut order_pos, _degeneracy) = degeneracy_ordering(&adj);
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
    for (_edge_id, edge) in hg.iter_by_size(3) {
        let (a, b, c) = (edge.nodes[0], edge.nodes[1], edge.nodes[2]);
        let mut inner_count = 0;
        let mut inner_intensity = 1.0;

        let mut motif = const {
            let mut rv = CompactMotif::<3>::zero();
            rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2]));
            rv
        };

        if let Some(&w) = edge2_set.get(&(a, b)) {
            motif.add_edge_with_nodes(const { CompressedNodeSet::from_array([0, 1]) });
            inner_intensity *= w;
            inner_count += 1;
        }

        if let Some(&w) = edge2_set.get(&(a, c)) {
            motif.add_edge_with_nodes(const { CompressedNodeSet::from_array([0, 2]) });
            inner_intensity *= w;
            inner_count += 1;
        }

        if let Some(&w) = edge2_set.get(&(b, c)) {
            motif.add_edge_with_nodes(const { CompressedNodeSet::from_array([1, 2]) });
            inner_intensity *= w;
            inner_count += 1;
        }

        if inner_count == 2 {
            straight_paths.count -= 1;
            straight_paths.mean_intensity -= inner_intensity.sqrt() as f64;
        }
        if inner_count == 3 {
            triangles.count -= 1;
            triangles.mean_intensity -= inner_intensity.cbrt() as f64;
        }

        let stat = motif_stats
            .entry(motif.fingerprint())
            .or_insert(MotifStats::new());
        stat.count += 1;
        stat.mean_intensity +=
            (edge.weight * inner_intensity).powf(1.0 / (inner_count + 1) as f32) as f64;
    }

    motif_stats.insert(triangle_fingeprint, triangles);
    motif_stats.insert(straight_path_fingerprint, straight_paths);

    for (_, stats) in motif_stats.iter_mut() {
        stats.mean_intensity /= stats.count.max(1) as f64;
    }

    motif_stats
}

pub fn unweighted_4(adj: &HyperAdjList<()>) -> HashMap<Fingerprint4, MotifStats> {
    let edges_2: Vec<(NodeId, NodeId, ())> = adj
        .iter_by_size(2)
        .map(|(_, e)| (e.nodes[0], e.nodes[1], ()))
        .collect();

    let (mut adj_list, _direct_map, _inverse_map) =
        AdjList::<(), Undirected, WithIncidence>::from_edges_mapped(edges_2);
    let adj_set: AdjSet<(), Undirected, WithIncidence> = adj_list.clone().into();

    let mut rv = HashMap::new();

    // Initialize motif stats
    let mut triangle = MotifStats::new();

    let mut path4 = MotifStats::new();
    let mut star4 = MotifStats::new();

    let mut k4 = MotifStats::new();
    let mut c4 = MotifStats::new();

    let mut diamond = MotifStats::new();
    let mut tailed_triangle = MotifStats::new();

    let mut tri_edge = vec![0; adj.m()];
    let mut tri_vertex = vec![0; adj.n()];

    adj_list.sort_neighbors();
    let (mut order_pos, _degeneracy) = degeneracy_ordering(&adj_list);
    order_pos.reverse();
    // Compute triangles + cliques
    // Count triangles with forward hashed in O(m^1.5)
    // TODO: make it use degeneracy order instead of degree order
    forward_hashed_cloj(&adj_list, Some(&order_pos), |a, b, c| {
        let upper_bound = a.min(b).min(c);
        let edge_ab = adj_set[a][&b].1 as usize;
        let edge_ac = adj_set[a][&c].1 as usize;
        let edge_bc = adj_set[b][&c].1 as usize;

        triangle.count += 1;

        tri_edge[edge_ab] += 1;
        tri_edge[edge_ac] += 1;
        tri_edge[edge_bc] += 1;

        tri_vertex[a as usize] += 1;
        tri_vertex[b as usize] += 1;
        tri_vertex[c as usize] += 1;

        // let na = adj_list[a].iter().map(|e| e.node).collect::<Vec<_>>();
        // let nb = adj_list[b].iter().map(|e| e.node).collect::<Vec<_>>();
        // let nc = adj_list[c].iter().map(|e| e.node).collect::<Vec<_>>();

        // 4-clique counting
        common_neighbors_sorted_list_3_by_key(
            &adj_list[a],
            &adj_list[b],
            &adj_list[c],
            &upper_bound,
            |e| &e.node,
            |_i, _j, _k| {
                // let _common = adj_list[a][i].node;
                k4.count += 1;
                // Add K4
            },
        );
    });

    // Compute other non-induced counts
    for x in 0..adj_list.n() {
        let deg_x = adj_list[x].len();
        star4.count += deg_x * (deg_x - 1) * (deg_x - 2) / 6;
        tailed_triangle.count += tri_vertex[x] * (deg_x - 2);

        let mut y = 0;
        loop {
            if y >= adj_list[x].len() {
                break;
            }
            let neighbor_y = adj_list[x][y].node as usize;
            if neighbor_y >= x {
                break;
            }

            let neighbor_y = adj_list[x][y].node as usize;
            let edge_xy = adj_list[x][y].edge as usize;
            let deg_y = adj_list[neighbor_y].len();

            path4.count += (deg_x - 1) * (deg_y - 1);
            diamond.count += tri_edge[edge_xy] * (tri_edge[edge_xy] - 1) / 2;

            y += 1;
        }
    }
    path4.count -= 3 * triangle.count;

    // c4 are enumerated efficiently
    c4.count = count_c4(&mut adj_list);

    // converting to induced counts
    diamond.count -= 6 * k4.count;
    c4.count -= 3 * k4.count + diamond.count;
    tailed_triangle.count -= 12 * k4.count + 4 * diamond.count;
    star4.count -= 4 * k4.count + 2 * diamond.count + tailed_triangle.count;
    path4.count -= 12 * k4.count + 6 * diamond.count + 2 * tailed_triangle.count + 4 * c4.count;

    // Add results to the motif stats hashmap
    rv.insert(PATH_4.fingerprint(), path4);
    rv.insert(STAR_4.fingerprint(), star4);

    rv.insert(FOUR_CLIQUE.fingerprint(), k4);
    rv.insert(FOUR_CYCLE.fingerprint(), c4);

    rv.insert(DIAMOND.fingerprint(), diamond);
    rv.insert(TAILED_TRIANGLE.fingerprint(), tailed_triangle);

    let mut mapped_nodes = vec![u8::MAX; adj.n()];
    let mut black_nodes = BitSet::with_capacity(adj.n());
    let mut inserted = BitSet::with_capacity(adj.n());
    let mut extension_nodes = vec![[CompactMotif::<4>::zero(); 2]; adj.n()];
    let mut node_list = Vec::with_capacity(adj.n() / 2);

    // let mut black_nodes = HashSet::new();
    // let mut extension_nodes = HashMap::new();

    for (pivot_edge_id, pivot_edge) in adj.iter_by_size(3) {
        let nodes = pivot_edge.nodes;
        // let min_inner_node = *nodes.iter().min().unwrap();

        // let mut mapped_nodes = [0; 3];
        mapped_nodes[nodes[0] as usize] = 0;
        mapped_nodes[nodes[1] as usize] = 1;
        mapped_nodes[nodes[2] as usize] = 2;

        let mut center_motif = const {
            let mut motif_3 = CompactMotif::<4>::zero();
            motif_3.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2]));

            let motif_2 = CompactMotif::<4>::zero();
            [motif_2, motif_3] // keeping them separated to subtract overcounted 2-uniform-motifs fast
        };

        for i in 0..3 {
            for (edge_id, edge) in adj.iter_incident_edges(nodes[i]) {
                if edge.nodes.len() < 4 && pivot_edge_id != edge_id {
                    let (mut i1, mut i2) = (0, 0);
                    let mut inner_nodes = [0; 2];
                    let mut outer_nodes = [0; 2];

                    for n in edge.nodes {
                        if mapped_nodes[*n as usize] != u8::MAX {
                            inner_nodes[i1] = *n;
                            i1 += 1;
                        } else {
                            outer_nodes[i2] = *n;
                            i2 += 1;
                        }
                    }

                    if i2 == 0 {
                        center_motif[0].add_edge_with_nodes(CompressedNodeSet::from_iter(
                            inner_nodes[0..i1]
                                .into_iter()
                                .map(|e| mapped_nodes[*e as usize]),
                        ));
                    } else if i2 == 1 {
                        let outer_node = outer_nodes[0] as usize;
                        //avoid counting over counting
                        if edge.nodes.len() == 3 && !(pivot_edge_id < edge_id) {
                            if !inserted.contains(outer_node) {
                                node_list.push(outer_node);
                                inserted.insert(outer_node);
                            }
                            black_nodes.insert(outer_node);
                            // (outer_nodes[0]);
                            // extension_nodes.remove(&outer_nodes[0]);
                            continue;
                        }

                        if black_nodes.contains(outer_node) {
                            continue;
                        }

                        let nodes = {
                            let mut rv = CompressedNodeSet::new(0);
                            for n in inner_nodes[0..i1].iter() {
                                rv.insert(mapped_nodes[*n as usize] as usize);
                            }
                            // assuming the added node is the last one without loss of generality
                            rv.insert(3);
                            rv
                        };

                        let motif = {
                            let mut rv = CompactMotif::<4>::zero();
                            rv.add_edge_with_nodes(nodes);
                            rv
                        };

                        let bucket = edge.nodes.len() - 2;
                        let motifs = &mut extension_nodes[outer_node];

                        if inserted.contains(outer_node) {
                            motifs[bucket] = motifs[bucket].bitor(motif);
                        } else {
                            let mut peripheral_motifs = [CompactMotif::<4>::zero(); 2];
                            peripheral_motifs[bucket] = motif;
                            *motifs = peripheral_motifs;
                            inserted.insert(outer_node);
                            node_list.push(outer_node);
                        }
                        // extension_nodes
                        //     .entry(outer_nodes[0])
                        //     .and_modify(|motifs: &mut [CompactMotif<4>; 2]| {
                        //         motifs[bucket] = motifs[bucket].bitor(motif);
                        //     })
                        //     .or_insert(peripheral_motifs);
                    }
                }
            }
        }

        for &node in node_list.iter() {
            if !black_nodes.contains(node) {
                let motifs = &extension_nodes[node];
                let uniform_2_motif = center_motif[0].bitor(motifs[0]);
                let uniform_3_motif = center_motif[1].bitor(motifs[1]);
                let combined = uniform_2_motif.bitor(uniform_3_motif);

                // Correcting overcounting of 2-uniform motifs
                rv.entry(uniform_2_motif.fingerprint())
                    .and_modify(|stats: &mut MotifStats| stats.count -= 1);

                rv.entry(combined.fingerprint())
                    .and_modify(|stats: &mut MotifStats| stats.count += 1)
                    .or_insert(MotifStats {
                        count: 1,
                        mean_intensity: 0.,
                        mean_coherence: 0.,
                        actual_intensity: 0.,
                    });
            }

            black_nodes.remove(node);
            inserted.remove(node);
        }

        node_list.clear();

        mapped_nodes[nodes[0] as usize] = u8::MAX;
        mapped_nodes[nodes[1] as usize] = u8::MAX;
        mapped_nodes[nodes[2] as usize] = u8::MAX;
    }

    let mut edges_2 = HashSet::new();
    let mut edges_3 = HashSet::new();

    for (_edge_id, edge) in adj.iter_by_size(2) {
        edges_2.insert(edge.nodes);
    }

    for (_edge_id, edge) in adj.iter_by_size(3) {
        edges_3.insert(edge.nodes);
    }

    for (_edge_id, edge) in adj.iter_by_size(4) {
        mapped_nodes[edge.nodes[0] as usize] = 0;
        mapped_nodes[edge.nodes[1] as usize] = 1;
        mapped_nodes[edge.nodes[2] as usize] = 2;
        mapped_nodes[edge.nodes[3] as usize] = 3;

        let mut motif = CompactMotif::<4>::zero();
        for i in 0..4 {
            for j in (i + 1)..4 {
                if edges_2.contains([edge.nodes[i], edge.nodes[j]].as_slice()) {
                    motif.add_edge_with_nodes(CompressedNodeSet::from_array([i as u8, j as u8]));
                }
            }
        }

        for i in 0..4 {
            for j in (i + 1)..4 {
                for k in (j + 1)..4 {
                    if edges_3.contains([edge.nodes[i], edge.nodes[j], edge.nodes[k]].as_slice()) {
                        motif.add_edge_with_nodes(CompressedNodeSet::from_array([
                            i as u8, j as u8, k as u8,
                        ]));
                    }
                }
            }
        }

        rv.entry(motif.fingerprint())
            .and_modify(|stats: &mut MotifStats| stats.count -= 1);

        motif.add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2, 3]));

        rv.entry(motif.fingerprint())
            .and_modify(|stats: &mut MotifStats| stats.count += 1)
            .or_insert(MotifStats {
                count: 1,
                mean_intensity: 0.,
                mean_coherence: 0.,
                actual_intensity: 0.,
            });
    }

    rv
}

#[derive(Debug, Clone, Copy)]
struct EdgeInfo {
    /// the number of triangle incident to the node a
    t_count: usize,

    /// sum of (w(a,b)w(a,d)w(b,d))^(1/3) for each triangle incident to the edge (a, b)
    triangle_13: f32,
    /// sum of (w(a,b)w(a,c)w(b,c))^(1/4) for each triangle incident to the edge (a,b)
    triangle_14: f32,
    /// sum of (w(a,b)w(a,c)w(b,c))^(2/4) for each triangle incident to the edge (a,b)
    triangle_24: f32,

    /// sum of (w(a,b)w(a,c)w(b,c))^(1/5) for each triangle incident to the edge (a,b)
    triangle_15: f32,
    /// sum of (w(a,b)w(a,c)w(b,c))^(2/5) for each triangle incident to the edge (a,b)
    triangle_25: f32,

    /// sum of w(a,d)^(1/3) if a>b, else w(b,d)^(1/3) for each triangle incident to the edge (a, b)
    edge_upper_13: f32,
    /// sum of w(a,d)^(1/3) if a>b, else w(b,d)^(2/3) for each triangle incident to the edge (a, b)
    edge_upper_23: f32,

    /// sum of w(b,d)^(1/3) if a>b, else w(b,d)^(1/3) for each triangle incident to the edge (a, b)
    edge_lower_13: f32,
    /// sum of w(b,d)^(1/3) if a>b, else w(b,d)^(2/3) for each triangle incident to the edge (a, b)
    edge_lower_23: f32,

    /// sum of w(a,d)^(1/3) + w(b,d)^(1/3) for each triangle incident to the edge (a, b)
    distal_edge_sum_13: f32,
    /// sum of w(a,d)^(1/3) + w(b,d)^(1/4) for each triangle incident to the edge (a, b)
    distal_edge_sum_14: f32,

    /// sum of (w(a,b)w(a,d)w(a,d)w(b,d))^(1/3) + (w(a,b)w(a,d)w(b,d)w(b,d))^(1/3) for each triangle incident to the edge (a, b)
    unbalanced_triangle_13: f32,
    /// sum of (w(a,b)w(a,d)w(a,d)w(b,d))^(1/4) + (w(a,b)w(a,d)w(b,d)w(b,d))^(1/3) for each triangle incident to the edge (a, b)
    unbalanced_triangle_14: f32,
}

impl EdgeInfo {
    pub fn empty() -> Self {
        Self {
            t_count: 0,
            triangle_14: 0.,
            triangle_24: 0.,
            triangle_15: 0.,
            triangle_25: 0.,
            edge_upper_13: 0.,
            edge_upper_23: 0.,
            edge_lower_13: 0.,
            edge_lower_23: 0.,
            triangle_13: 0.,
            distal_edge_sum_13: 0.,
            unbalanced_triangle_13: 0.,
            distal_edge_sum_14: 0.,
            unbalanced_triangle_14: 0.,
        }
    }
}

pub fn newton_girard_2(s1: f32, s2: f32) -> f32 {
    (s1 * s1 - s2) / 2.0
}

pub fn newton_girard_3(s1: f32, s2: f32, s3: f32) -> f32 {
    (s1 * s1 * s1 - 3.0 * s1 * s2 + 2.0 * s3) / 6.0
}

#[derive(Debug, Clone, Copy)]
struct NodeInfo {
    /// the number of triangle incident to the node a
    t_count: usize,

    /// sum w(a,b) for each edge (a,b) incident to the node a
    sum_11: f32,
    /// sum w(a,b)^(1/3) for each edge (a,b) incident to the node a
    sum_13: f32,
    /// sum w(a,b)^(2/3) for each edge (a,b) incident to the node a
    sum_23: f32,
    /// sum w(a,b)^(1/4) for each edge (a,b) incident to the node a
    sum_14: f32,
}

#[derive(Debug, Clone)]
struct MotifStatsPair {
    induced: MotifStats,
    non_induced: MotifStats,
}

impl MotifStatsPair {
    pub fn new() -> Self {
        Self {
            induced: MotifStats::new(),
            non_induced: MotifStats::new(),
        }
    }
}

/// Helper macro to access the mean intensity of the induced motif stats in a MotifStatsPair
macro_rules! ii {
    ($name:ident) => {
        $name.induced.mean_intensity
    };
}

/// Helper macro to access the mean intensity of the non-induced motif stats in a MotifStatsPair
macro_rules! nii {
    ($name:ident) => {
        $name.non_induced.mean_intensity
    };
}

pub fn weighted_4(adj: &HyperAdjList<NodeWeight>) -> HashMap<Fingerprint4, MotifStats> {
    let edges_2 = adj
        .iter_by_size(2)
        .map(|(_, e)| (e.nodes[0], e.nodes[1], *e.weight))
        .collect::<Vec<_>>();

    let (mut adj_list, _direct_map, _inverse_map) =
        AdjList::<NodeWeight, Undirected, WithIncidence>::from_edges_mapped(edges_2);
    // adj_list.sort_neighbors();
    // let adj_set: AdjSet<NodeWeight, Undirected, WithIncidence> = adj_list.clone().into();

    let mut rv = HashMap::new();

    // Final motif stats
    let mut triangle = MotifStats::new();

    let mut path4 = MotifStats::new();
    let mut star4 = MotifStats::new();

    let mut k4 = MotifStats::new();
    let mut c4 = MotifStats::new();

    let mut diamond = MotifStats::new();
    let mut paw = MotifStats::new();

    // let mut tri_edge_count = vec![0; adj.m()];
    // let mut tri_edge_intensity = vec![0.0; adj.m()];
    // let mut tri_vertex = vec![0; adj.n()];
    // let mut tri_distal_edge = vec![((0.0, 0.0), (0.0, 0.0)); adj.m()];

    // Saving partial stats to convert rom induced to non induced
    // <motif_a>_in_<motif_b> stores the stats of the non induced occurences of motif_a in motif_b

    // diamon
    let mut diamond_in_k4 = MotifStatsPair::new();

    // c4
    let mut c4_in_k4 = MotifStatsPair::new();
    let mut c4_in_diamond = MotifStatsPair::new();

    // paw
    let mut paw_in_k4 = MotifStatsPair::new();
    let mut paw_in_diamond = MotifStatsPair::new();

    // star4
    let mut star4_in_paw = MotifStatsPair::new();
    let mut star4_in_diamond = MotifStatsPair::new();
    let mut star4_in_k4 = MotifStatsPair::new();

    // path4
    let mut path4_in_paw = MotifStatsPair::new();
    let mut path4_in_c4 = MotifStatsPair::new();

    let mut path4_in_diamond_ring = MotifStatsPair::new();
    let mut path4_in_diamond_inner = MotifStatsPair::new();

    let mut path4_in_k4 = MotifStatsPair::new();

    // Coefficient per edge used for fast combinatorial computation of iintensities
    let mut edge_infos = vec![EdgeInfo::empty(); adj.m()];
    let mut node_infos = Vec::with_capacity(adj.n());

    for x in 0..adj_list.n() {
        let mut sum_11 = 0.0;
        let mut sum_13 = 0.0;
        let mut sum_23 = 0.0;
        let mut sum_14 = 0.0;

        for y in adj_list[x].iter() {
            sum_11 += y.weight;
            sum_13 += y.weight.powf(1.0 / 3.0);
            sum_23 += y.weight.powf(2.0 / 3.0);
            sum_14 += y.weight.powf(1.0 / 4.0);
        }

        node_infos.push(NodeInfo {
            t_count: 0,
            sum_11,
            sum_13,
            sum_23,
            sum_14,
        });
    }

    let (mut order_pos, _degeneracy) = degeneracy_ordering(&adj_list);
    order_pos.reverse();

    // Compute triangles + cliques
    // Count triangles with forward hashed in O(m^1.5)
    forward_sorted_cloj(&mut adj_list, Some(&order_pos), |adj_list, t| {
        let a = t.nodes[0] as usize;
        let b = t.nodes[1] as usize;
        let c = t.nodes[2] as usize;

        let edge_ab = t.edges[0] as usize;
        let edge_ac = t.edges[1] as usize;
        let edge_bc = t.edges[2] as usize;

        let weight_ab = *t.weights[0];
        let weight_ac = *t.weights[1];
        let weight_bc = *t.weights[2];

        let prod = weight_ab * weight_ac * weight_bc;

        triangle.count += 1;
        triangle.mean_intensity += prod.powf(1.0 / 3.0) as f64;

        edge_infos[edge_ab].t_count += 1;
        edge_infos[edge_ac].t_count += 1;
        edge_infos[edge_bc].t_count += 1;

        node_infos[a].t_count += 1;
        node_infos[b].t_count += 1;
        node_infos[c].t_count += 1;

        {
            // for paw counting
            let prod = prod.powf(1.0 / 4.0);
            let weight_ab = weight_ab.powf(1.0 / 4.0);
            let weight_ac = weight_ac.powf(1.0 / 4.0);
            let weight_bc = weight_bc.powf(1.0 / 4.0);

            edge_infos[edge_ab].distal_edge_sum_14 += weight_ac + weight_bc;
            edge_infos[edge_ab].unbalanced_triangle_14 += prod * weight_ac + prod * weight_bc;

            edge_infos[edge_ac].distal_edge_sum_14 += weight_ab + weight_bc;
            edge_infos[edge_ac].unbalanced_triangle_14 += prod * weight_ab + prod * weight_bc;

            edge_infos[edge_bc].distal_edge_sum_14 += weight_ab + weight_ac;
            edge_infos[edge_bc].unbalanced_triangle_14 += prod * weight_ab + prod * weight_ac;

            paw.mean_intensity += (prod * (node_infos[a].sum_14 - weight_ab - weight_ac)) as f64;
            paw.mean_intensity += (prod * (node_infos[b].sum_14 - weight_ab - weight_bc)) as f64;
            paw.mean_intensity += (prod * (node_infos[c].sum_14 - weight_ac - weight_bc)) as f64;
        }

        {
            // for star4 counting
            let prod = prod.powf(1.0 / 3.0);
            let weight_ab = weight_ab.powf(1.0 / 3.0);
            let weight_ac = weight_ac.powf(1.0 / 3.0);
            let weight_bc = weight_bc.powf(1.0 / 3.0);

            edge_infos[edge_ab].distal_edge_sum_13 += weight_ac + weight_bc;
            edge_infos[edge_ab].unbalanced_triangle_13 += prod * weight_ac + prod * weight_bc;

            edge_infos[edge_ac].distal_edge_sum_13 += weight_ab + weight_bc;
            edge_infos[edge_ac].unbalanced_triangle_13 += prod * weight_ab + prod * weight_bc;

            edge_infos[edge_bc].distal_edge_sum_13 += weight_ab + weight_ac;
            edge_infos[edge_bc].unbalanced_triangle_13 += prod * weight_ab + prod * weight_ac;

            nii!(star4_in_paw) +=
                ((prod / weight_bc) * (node_infos[a].sum_13 - weight_ab - weight_ac)) as f64;
            nii!(star4_in_paw) +=
                ((prod / weight_ac) * (node_infos[b].sum_13 - weight_ab - weight_bc)) as f64;
            nii!(star4_in_paw) +=
                ((prod / weight_ab) * (node_infos[c].sum_13 - weight_ac - weight_bc)) as f64;

            nii!(path4_in_paw) += (weight_bc
                * (weight_ab * (node_infos[a].sum_13 - weight_ab - weight_ac)
                    + weight_ac * (node_infos[a].sum_13 - weight_ab - weight_ac)))
                as f64;

            nii!(path4_in_paw) += (weight_ac
                * (weight_ab * (node_infos[b].sum_13 - weight_ab - weight_bc)
                    + weight_bc * (node_infos[b].sum_13 - weight_ab - weight_bc)))
                as f64;

            nii!(path4_in_paw) += (weight_ab
                * (weight_ac * (node_infos[c].sum_13 - weight_ac - weight_bc)
                    + weight_bc * (node_infos[c].sum_13 - weight_ac - weight_bc)))
                as f64;
        }

        let s13 = prod.powf(1.0 / 3.0);

        let s14 = prod.powf(1.0 / 4.0);
        let s24 = prod.powf(2.0 / 4.0);

        let s15 = prod.powf(1.0 / 5.0);
        let s25 = prod.powf(2.0 / 5.0);

        edge_infos[edge_ab].triangle_13 += s13;
        edge_infos[edge_ac].triangle_13 += s13;
        edge_infos[edge_bc].triangle_13 += s13;

        edge_infos[edge_ab].triangle_14 += s14;
        edge_infos[edge_ab].triangle_24 += s24;

        edge_infos[edge_ac].triangle_14 += s14;
        edge_infos[edge_ac].triangle_24 += s24;

        edge_infos[edge_bc].triangle_14 += s14;
        edge_infos[edge_bc].triangle_24 += s24;

        edge_infos[edge_ab].triangle_15 += s15;
        edge_infos[edge_ab].triangle_25 += s25;

        edge_infos[edge_ac].triangle_15 += s15;
        edge_infos[edge_ac].triangle_25 += s25;

        edge_infos[edge_bc].triangle_15 += s15;
        edge_infos[edge_bc].triangle_25 += s25;

        if a < b {
            edge_infos[edge_ab].edge_lower_13 += weight_ac.powf(1.0 / 3.0);
            edge_infos[edge_ab].edge_lower_23 += weight_ac.powf(2.0 / 3.0);
            edge_infos[edge_ab].edge_upper_13 += weight_bc.powf(1.0 / 3.0);
            edge_infos[edge_ab].edge_upper_23 += weight_bc.powf(2.0 / 3.0);
        } else {
            edge_infos[edge_ab].edge_upper_13 += weight_ac.powf(1.0 / 3.0);
            edge_infos[edge_ab].edge_upper_23 += weight_ac.powf(2.0 / 3.0);
            edge_infos[edge_ab].edge_lower_13 += weight_bc.powf(1.0 / 3.0);
            edge_infos[edge_ab].edge_lower_23 += weight_bc.powf(2.0 / 3.0);
        }

        if a < c {
            edge_infos[edge_ac].edge_lower_13 += weight_ab.powf(1.0 / 3.0);
            edge_infos[edge_ac].edge_lower_23 += weight_ab.powf(2.0 / 3.0);
            edge_infos[edge_ac].edge_upper_13 += weight_bc.powf(1.0 / 3.0);
            edge_infos[edge_ac].edge_upper_23 += weight_bc.powf(2.0 / 3.0);
        } else {
            edge_infos[edge_ac].edge_upper_13 += weight_ab.powf(1.0 / 3.0);
            edge_infos[edge_ac].edge_upper_23 += weight_ab.powf(2.0 / 3.0);
            edge_infos[edge_ac].edge_lower_13 += weight_bc.powf(1.0 / 3.0);
            edge_infos[edge_ac].edge_lower_23 += weight_bc.powf(2.0 / 3.0);
        }

        if b < c {
            edge_infos[edge_bc].edge_lower_13 += weight_ab.powf(1.0 / 3.0);
            edge_infos[edge_bc].edge_lower_23 += weight_ab.powf(2.0 / 3.0);
            edge_infos[edge_bc].edge_upper_13 += weight_ac.powf(1.0 / 3.0);
            edge_infos[edge_bc].edge_upper_23 += weight_ac.powf(2.0 / 3.0);
        } else {
            edge_infos[edge_bc].edge_upper_13 += weight_ab.powf(1.0 / 3.0);
            edge_infos[edge_bc].edge_upper_23 += weight_ab.powf(2.0 / 3.0);
            edge_infos[edge_bc].edge_lower_13 += weight_ac.powf(1.0 / 3.0);
            edge_infos[edge_bc].edge_lower_23 += weight_ac.powf(2.0 / 3.0);
        }

        let upper_bound = order_pos.pos[a].min(order_pos.pos[b]).min(order_pos.pos[c]);
        // 4-clique counting
        // forward hashed sorts the adj_list neighbors based on degeneracy ordering so we need to
        // use pos[i] instead of i as key
        common_neighbors_sorted_list_3_by_key(
            &adj_list[a],
            &adj_list[b],
            &adj_list[c],
            &(upper_bound as usize),
            |e| &order_pos.pos[e.node as usize],
            |i, j, k| {
                // let common = adj_list[a][i].node;

                let weight_ad = adj_list[a][i].weight;
                let weight_bd = adj_list[b][j].weight;
                let weight_cd = adj_list[c][k].weight;

                let prod = weight_ab * weight_ac * weight_bc * weight_ad * weight_bd * weight_cd;
                k4.count += 1;
                k4.mean_intensity += prod.powf(1.0 / 6.0) as f64;

                nii!(diamond_in_k4) += ((prod / weight_ab).powf(1.0 / 5.0)
                    + (prod / weight_ac).powf(1.0 / 5.0)
                    + (prod / weight_bc).powf(1.0 / 5.0)
                    + (prod / weight_ad).powf(1.0 / 5.0)
                    + (prod / weight_bd).powf(1.0 / 5.0)
                    + (prod / weight_cd).powf(1.0 / 5.0))
                    as f64;

                nii!(c4_in_k4) += ((prod / weight_ab / weight_cd).powf(1.0 / 4.0)
                    + (prod / weight_ac / weight_bd).powf(1.0 / 4.0)
                    + (prod / weight_ad / weight_bc).powf(1.0 / 4.0))
                    as f64;

                let t1 = weight_ab * weight_ac * weight_bc;
                let t2 = weight_ac * weight_ad * weight_cd;
                let t3 = weight_bc * weight_cd * weight_bd;
                let t4 = weight_ab * weight_ad * weight_bd;

                nii!(paw_in_k4) += ((t1 * weight_bd).powf(1.0 / 4.0)
                    + (t1 * weight_ad).powf(1.0 / 4.0)
                    + (t1 * weight_cd).powf(1.0 / 4.0)
                    + (t2 * weight_ab).powf(1.0 / 4.0)
                    + (t2 * weight_bc).powf(1.0 / 4.0)
                    + (t2 * weight_bd).powf(1.0 / 4.0)
                    + (t3 * weight_ab).powf(1.0 / 4.0)
                    + (t3 * weight_ad).powf(1.0 / 4.0)
                    + (t3 * weight_ac).powf(1.0 / 4.0)
                    + (t4 * weight_ac).powf(1.0 / 4.0)
                    + (t4 * weight_bc).powf(1.0 / 4.0)
                    + (t4 * weight_cd).powf(1.0 / 4.0)) as f64;

                nii!(star4_in_k4) += ((weight_ab * weight_ac * weight_ad).powf(1.0 / 3.0)
                    + (weight_ab * weight_bc * weight_bd).powf(1.0 / 3.0)
                    + (weight_ac * weight_bc * weight_cd).powf(1.0 / 3.0)
                    + (weight_bd * weight_cd * weight_ad).powf(1.0 / 3.0))
                    as f64;

                let vertical = (weight_ac * weight_bd).powf(1.0 / 3.0);
                let horizontal = (weight_ab * weight_cd).powf(1.0 / 3.0);
                let inner = (weight_ad * weight_bc).powf(1.0 / 3.0);

                nii!(path4_in_k4) += ((weight_ad.powf(1.0 / 3.0) + weight_bc.powf(1.0 / 3.0))
                    * (horizontal + vertical)
                    + weight_ab.powf(1.0 / 3.0) * inner
                    + weight_cd.powf(1.0 / 3.0) * inner
                    + weight_ac.powf(1.0 / 3.0) * inner
                    + weight_bd.powf(1.0 / 3.0) * inner
                    + weight_ab.powf(1.0 / 3.0) * vertical
                    + weight_cd.powf(1.0 / 3.0) * vertical
                    + weight_ac.powf(1.0 / 3.0) * horizontal
                    + weight_bd.powf(1.0 / 3.0) * horizontal)
                    as f64;
            },
        );
    });

    // Compute other non-induced counts. Here
    for x in 0..adj_list.n() {
        let deg_x = adj_list[x].len();
        star4.count += deg_x * (deg_x - 1) * (deg_x - 2) / 6;
        star4.mean_intensity += newton_girard_3(
            node_infos[x].sum_13,
            node_infos[x].sum_23,
            node_infos[x].sum_11,
        ) as f64;

        paw.count += node_infos[x].t_count * (deg_x - 2);

        let mut y = 0;
        loop {
            if y >= adj_list[x].len() {
                break;
            }
            let neighbor_y = adj_list[x][y].node as usize;
            if order_pos.pos[neighbor_y] >= order_pos.pos[x] {
                break;
            }

            let edge_xy = adj_list[x][y].edge as usize;
            let weight_xy = adj_list[x][y].weight;
            let deg_y = adj_list[neighbor_y].len();

            path4.count += (deg_x - 1) * (deg_y - 1);
            path4.mean_intensity += (weight_xy.powf(1.0 / 3.0)
                * (node_infos[x].sum_13 - weight_xy.powf(1.0 / 3.0))
                * (node_infos[neighbor_y].sum_13 - weight_xy.powf(1.0 / 3.0)))
                as f64;

            edge_infos[edge_xy].t_count = max(edge_infos[edge_xy].t_count, 1);
            diamond.count += edge_infos[edge_xy].t_count * (edge_infos[edge_xy].t_count - 1) / 2;
            diamond.mean_intensity += (newton_girard_2(
                edge_infos[edge_xy].triangle_15,
                edge_infos[edge_xy].triangle_25,
            ) / weight_xy.powf(1.0 / 5.0)) as f64;

            nii!(c4_in_diamond) += (newton_girard_2(
                edge_infos[edge_xy].triangle_14,
                edge_infos[edge_xy].triangle_24,
            ) / weight_xy.powf(2.0 / 4.0)) as f64;

            nii!(paw_in_diamond) +=
                (edge_infos[edge_xy].triangle_14 * edge_infos[edge_xy].distal_edge_sum_14
                    - edge_infos[edge_xy].unbalanced_triangle_14) as f64;

            nii!(star4_in_diamond) += (weight_xy.powf(1.0 / 3.0)
                * (newton_girard_2(
                    edge_infos[edge_xy].edge_upper_13,
                    edge_infos[edge_xy].edge_upper_23,
                ) + newton_girard_2(
                    edge_infos[edge_xy].edge_lower_13,
                    edge_infos[edge_xy].edge_lower_23,
                ))) as f64;

            nii!(path4_in_diamond_ring) += ((edge_infos[edge_xy].triangle_13
                * edge_infos[edge_xy].distal_edge_sum_13
                - edge_infos[edge_xy].unbalanced_triangle_13)
                / weight_xy.powf(1.0 / 3.0)) as f64;

            // can collapse into triangles but they are subtracted later
            nii!(path4_in_diamond_inner) += (weight_xy.powf(1.0 / 3.0)
                * (edge_infos[edge_xy].edge_upper_13)
                * (edge_infos[edge_xy].edge_lower_13))
                as f64;
            y += 1;
        }
    }
    path4.count -= 3 * triangle.count;
    path4.mean_intensity -= 3.0 * triangle.mean_intensity;
    nii!(path4_in_diamond_inner) -= 3.0 * triangle.mean_intensity;

    // c4 are enumerated efficiently. the adj list's neighbors are sorted by degree!!
    (c4.count, c4.mean_intensity, nii!(path4_in_c4)) = intensity_c4_subinc(&mut adj_list);
    c4.mean_intensity *= c4.count.max(1) as f64; // restore to sum instead of mean
    nii!(path4_in_c4) *= 4.0 * c4.count.max(1) as f64;

    // converting subgraphlets to induced counts
    // diamond
    ii!(diamond_in_k4) = nii!(diamond_in_k4);

    // c4
    ii!(c4_in_k4) = nii!(c4_in_k4);
    ii!(c4_in_diamond) = nii!(c4_in_diamond) - 2.0 * ii!(c4_in_k4);

    // paw
    ii!(paw_in_k4) = nii!(paw_in_k4);
    ii!(paw_in_diamond) = nii!(paw_in_diamond) - 2.0 * ii!(paw_in_k4);

    // star4
    ii!(star4_in_k4) = nii!(star4_in_k4);
    ii!(star4_in_diamond) = nii!(star4_in_diamond) - 3.0 * ii!(star4_in_k4);
    ii!(star4_in_paw) = nii!(star4_in_paw) - 2.0 * ii!(star4_in_diamond) - 3.0 * ii!(star4_in_k4);

    // path4
    ii!(path4_in_k4) = nii!(path4_in_k4);
    // ii!(path4_in_diamond) = nii!(path4_in_diamond) - 3.0 * ii!(path4_in_k4);
    ii!(path4_in_diamond_ring) = nii!(path4_in_diamond_ring) - 2.0 * ii!(path4_in_k4);
    ii!(path4_in_diamond_inner) = nii!(path4_in_diamond_inner) - ii!(path4_in_k4);
    ii!(path4_in_c4) = nii!(path4_in_c4) - ii!(path4_in_diamond_ring) - ii!(path4_in_k4);
    ii!(path4_in_paw) = nii!(path4_in_paw)
        - ii!(path4_in_diamond_ring)
        - 2.0 * ii!(path4_in_diamond_inner)
        - 2.0 * ii!(path4_in_k4);

    // converting to induced counts
    diamond.count -= 6 * k4.count;
    c4.count -= 3 * k4.count + diamond.count;
    paw.count -= 12 * k4.count + 4 * diamond.count;
    star4.count -= 4 * k4.count + 2 * diamond.count + paw.count;
    path4.count -= 12 * k4.count + 6 * diamond.count + 2 * paw.count + 4 * c4.count;

    // converting to induced intensities
    diamond.mean_intensity -= ii!(diamond_in_k4);
    c4.mean_intensity -= ii!(c4_in_diamond) + ii!(c4_in_k4);
    paw.mean_intensity -= ii!(paw_in_diamond) + ii!(paw_in_k4);
    star4.mean_intensity -= ii!(star4_in_paw) + ii!(star4_in_diamond) + ii!(star4_in_k4);
    path4.mean_intensity -= ii!(path4_in_paw)
        + ii!(path4_in_c4)
        + ii!(path4_in_diamond_ring)
        + ii!(path4_in_diamond_inner)
        + ii!(path4_in_k4);

    // Add results to the motif stats hashmap
    if path4.count > 0 {
        rv.insert(PATH_4.fingerprint(), path4);
    }
    if star4.count > 0 {
        rv.insert(STAR_4.fingerprint(), star4);
    }

    if c4.count > 0 {
        rv.insert(FOUR_CYCLE.fingerprint(), c4);
    }
    if k4.count > 0 {
        rv.insert(FOUR_CLIQUE.fingerprint(), k4);
    }

    if diamond.count > 0 {
        rv.insert(DIAMOND.fingerprint(), diamond);
    }
    if paw.count > 0 {
        rv.insert(TAILED_TRIANGLE.fingerprint(), paw);
    }

    let mut groups4: HashSet<[NodeId; 4]> =
        HashSet::with_capacity(adj.count_by_size(3) + adj.count_by_size(4));

    let mut extension_nodes_map = vec![(CompactMotif::<4>::zero(), 1.0, 1.0); adj.n()];
    let mut extension_nodes_list = vec![0; adj.n()];
    let mut inserted = BitSet::with_capacity(adj.n());

    for (pivot_edge_id, pivot_edge) in adj.iter_by_size(3) {
        extension_nodes_list.clear();
        let mut center_motif = CompactMotif::<4>::zero();
        let mut center_intensity_2 = 1.0;
        center_motif.add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2]));
        for curr_pivot in 0..3 {
            for (_edge_id, edge) in adj.iter_incident_by_size(pivot_edge.nodes[curr_pivot], 2) {
                assert!(edge.nodes.len() == 2);

                let non_pivot = if edge.nodes[0] == pivot_edge.nodes[curr_pivot] {
                    edge.nodes[1]
                } else {
                    edge.nodes[0]
                };

                let (is_inner, inner_index) = if non_pivot == pivot_edge.nodes[0] {
                    (true, 0)
                } else if non_pivot == pivot_edge.nodes[1] {
                    (true, 1)
                } else if non_pivot == pivot_edge.nodes[2] {
                    (true, 2)
                } else {
                    (false, 0)
                };

                if is_inner {
                    if pivot_edge.nodes[curr_pivot] < non_pivot {
                        center_motif.add_edge_with_nodes(CompressedNodeSet::from_array([
                            curr_pivot as u8,
                            inner_index,
                        ]));
                        center_intensity_2 *= edge.weight;
                    }
                } else {
                    if !inserted.contains(non_pivot as usize) {
                        inserted.insert(non_pivot as usize);
                        extension_nodes_list.push(non_pivot);
                    }

                    extension_nodes_map[non_pivot as usize]
                        .0
                        .add_edge_with_nodes(CompressedNodeSet::from_array([curr_pivot as u8, 3]));

                    extension_nodes_map[non_pivot as usize].1 *= edge.weight;
                }
            }

            for (edge_id, edge) in adj.iter_incident_by_size(pivot_edge.nodes[curr_pivot], 3) {
                assert!(edge.nodes.len() == 3);
                if pivot_edge_id == edge_id {
                    continue;
                }

                let mut outer = [0; 2];
                let mut inner = [(0, 0); 2];
                let mut outer_count = 0;
                let mut inner_count = 0;
                for i in 0..3 {
                    if edge.nodes[i] == pivot_edge.nodes[0] {
                        inner[inner_count] = (edge.nodes[i], 0);
                        inner_count += 1;
                    } else if edge.nodes[i] == pivot_edge.nodes[1] {
                        inner[inner_count] = (edge.nodes[i], 1);
                        inner_count += 1;
                    } else if edge.nodes[i] == pivot_edge.nodes[2] {
                        inner[inner_count] = (edge.nodes[i], 2);
                        inner_count += 1;
                    } else {
                        outer[outer_count] = edge.nodes[i];
                        outer_count += 1;
                    }
                }

                if outer_count == 1 {
                    let outer = outer[0];
                    // let pivot = pivot_edge.nodes[i];
                    let (inner_node, inner_index) = if inner[0].0 == pivot_edge.nodes[curr_pivot] {
                        inner[1]
                    } else {
                        inner[0]
                    };

                    if pivot_edge.nodes[curr_pivot] < inner_node {
                        continue;
                    }

                    if !inserted.contains(outer as usize) {
                        inserted.insert(outer as usize);
                        extension_nodes_list.push(outer);
                    }

                    extension_nodes_map[outer as usize].0.add_edge_with_nodes(
                        CompressedNodeSet::from_array([curr_pivot as u8, inner_index, 3]),
                    );

                    extension_nodes_map[outer as usize].2 *= edge.weight;
                }
            }

            // if edge.nodes.len() == 2 {
            // } else if edge.nodes.len() == 3 {
            //     let outer = 0;
            //     for j in 0..3 {
            //         if edge.nodes[j] != pivot_edge.nodes[i] && edge.nodes[j] {
            //             continue;
            //         }
            //     }
            //
            //     let outer = {
            // };
            // }
        }

        for &outer in &extension_nodes_list {
            let sorted_group4 = {
                let mut v = [
                    outer,
                    pivot_edge.nodes[0],
                    pivot_edge.nodes[1],
                    pivot_edge.nodes[2],
                ];
                for i in 1..4 {
                    if v[i] < v[i - 1] {
                        v.swap(i, i - 1);
                    }
                }
                v
            };

            if groups4.contains(&sorted_group4) {
                extension_nodes_map[outer as usize] = (CompactMotif::<4>::zero(), 1.0, 1.0);
                inserted.remove(outer as usize);
                continue;
            }

            let peripheral_motif = extension_nodes_map[outer as usize].0;
            let c2 = center_motif.filtered_by_order(2).edge_count()
                + extension_nodes_map[outer as usize]
                    .0
                    .filtered_by_order(2)
                    .edge_count();

            let c3 = extension_nodes_map[outer as usize]
                .0
                .filtered_by_order(3)
                .edge_count()
                + 1;

            let i2 = extension_nodes_map[outer as usize].1 * center_intensity_2;
            let i3 = extension_nodes_map[outer as usize].2 * pivot_edge.weight;

            // correcting overcounting
            rv.entry(
                center_motif
                    .bitor(peripheral_motif)
                    .filtered_by_order(2)
                    .fingerprint(),
            )
            .and_modify(|e| {
                e.count -= 1;
                e.mean_intensity -= i2.powf(1.0 / c2 as f32) as f64;
            });

            // adding actual count
            let entry = rv
                .entry(center_motif.bitor(peripheral_motif).fingerprint())
                .or_insert(MotifStats::new());
            entry.count += 1;
            entry.mean_intensity += (i2 * i3).powf(1.0 / (c2 + c3) as f32) as f64;

            extension_nodes_map[outer as usize] = (CompactMotif::<4>::zero(), 1.0, 1.0);
            inserted.remove(outer as usize);

            groups4.insert(sorted_group4);
        }
    }

    let mut edges_2 = HashMap::with_capacity(adj.count_by_size(2));

    let mut edges_3 = HashMap::with_capacity(adj.count_by_size(3));

    for (_edge_id, edge) in adj.iter_by_size(2) {
        edges_2.insert(edge.nodes, *edge.weight);
    }

    for (_edge_id, edge) in adj.iter_by_size(3) {
        edges_3.insert(edge.nodes, *edge.weight);
    }

    for (_edge_id, edge) in adj.iter_by_size(4) {
        let mut inner_intensity = 1.0;

        let mut motif = CompactMotif::<4>::zero();
        for i in 0..4 {
            for j in (i + 1)..4 {
                if let Some(&weight) = edges_2.get([edge.nodes[i], edge.nodes[j]].as_slice()) {
                    motif.add_edge_with_nodes(CompressedNodeSet::from_array([i as u8, j as u8]));
                    inner_intensity *= weight as f32;
                }
            }
        }

        for i in 0..4 {
            for j in (i + 1)..4 {
                for k in (j + 1)..4 {
                    if let Some(&weight) =
                        edges_3.get([edge.nodes[i], edge.nodes[j], edge.nodes[k]].as_slice())
                    {
                        motif.add_edge_with_nodes(CompressedNodeSet::from_array([
                            i as u8, j as u8, k as u8,
                        ]));

                        inner_intensity *= weight as f32;
                    }
                }
            }
        }

        // fix overcounting
        rv.entry(motif.fingerprint()).and_modify(|stats| {
            stats.count -= 1;
            stats.mean_intensity -= inner_intensity.powf(1.0 / motif.edge_count() as f32) as f64;
        });

        // add motif
        motif.add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2, 3]));
        let stats = rv.entry(motif.fingerprint()).or_insert(MotifStats::new());
        stats.count += 1;
        stats.mean_intensity +=
            (inner_intensity * edge.weight).powf(1.0 / motif.edge_count() as f32) as f64;
    }

    for (_fingerprint, stats) in rv.iter_mut() {
        stats.mean_intensity /= stats.count.max(1) as f64;
    }

    rv.retain(|_f, v| v.count > 0);

    rv
}
