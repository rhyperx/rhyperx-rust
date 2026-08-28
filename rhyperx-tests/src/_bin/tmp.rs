use std::{
    cmp::{max, min},
    error::Error,
    hash::Hash,
    time::Instant,
};

use bit_set::BitSet;
use hashbrown::{HashMap, HashSet};
use nohash_hasher::BuildNoHashHasher;
use rust_core::{
    loader::DatasetLoader,
    motifs::{compressed_motif::CompactMotif, compressed_node_set::CompressedNodeSet},
    types::{NodeId, NodeWeight, hyperadj_list::HyperAdjList},
};
use rustc_hash::FxHashMap; // Added FxHashMap

#[inline(always)]
fn pack_pair(a: NodeId, b: NodeId, n: usize) -> u32 {
    a + n as u32 + b
}

struct Edge {
    a: NodeId,
    b: NodeId,
}
impl Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.a.hash(state);
        self.b.hash(state);
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    test4()?;
    Ok(())
}

pub fn test1() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .dblp()
        .weighted()
        .load()?;
    hg.normalize_node_ids();

    // Estimate total insertions (3 per 3-uniform edge) to pre-allocate memory accurately
    // Replace `hg.num_edges()` with the correct method name if different in your library
    let estimated_capacity = hg.m() * 3;

    // --- Benchmark FoldHash ---
    let time_fold = Instant::now();
    let mut fold_map: HashMap<(NodeId, NodeId), Vec<NodeId>> =
        HashMap::with_capacity(estimated_capacity);
    for edge in hg.iter_edges::<3>() {
        let nodes = edge.nodes;
        const AVG_SIZE: usize = 10;

        fold_map
            .entry((nodes[0], nodes[1]))
            .and_modify(|v| v.push(nodes[2]))
            .or_insert_with(|| {
                let mut rv = Vec::with_capacity(AVG_SIZE);
                rv.push(nodes[2]);
                rv
            });

        fold_map
            .entry((nodes[0], nodes[2]))
            .and_modify(|v| v.push(nodes[1]))
            .or_insert_with(|| {
                let mut rv = Vec::with_capacity(AVG_SIZE);
                rv.push(nodes[1]);
                rv
            });

        fold_map
            .entry((nodes[1], nodes[2]))
            .and_modify(|v| v.push(nodes[0]))
            .or_insert_with(|| {
                let mut rv = Vec::with_capacity(AVG_SIZE);
                rv.push(nodes[0]);
                rv
            });
    }

    let mut count = 0;
    let mut avg = 0;
    let mut max = 0;
    let mut min = u32::MAX;
    for (key, extension_nodes) in fold_map.iter() {
        let &(a, b) = key;
        avg += extension_nodes.len();
        max = max.max(extension_nodes.len());
        min = min.min(extension_nodes.len() as u32);
        for i in 0..extension_nodes.len() {
            for j in (i + 1)..extension_nodes.len() {
                let c = extension_nodes[i];
                let d = extension_nodes[j];

                count += fold_map.contains_key(&(a, b)) as usize;
                count += fold_map.contains_key(&(a, c)) as usize;
                count += fold_map.contains_key(&(c, d)) as usize;
                count += fold_map.contains_key(&(b, d)) as usize;
                count += fold_map.contains_key(&(a, d)) as usize;
                count += fold_map.contains_key(&(b, c)) as usize;
            }
        }
    }
    println!(
        "avg: {}, min: {}, max: {}",
        avg as f64 / fold_map.len() as f64,
        min,
        max
    );
    println!("count: {}", count);

    println!("RandomState Finished in: {:?}", time_fold.elapsed());

    // --- Benchmark rustc_hash (FxHash) ---
    let time_fx = Instant::now();
    let mut fx_map = FxHashMap::with_capacity_and_hasher(estimated_capacity, Default::default());
    for edge in hg.iter_edges::<3>() {
        let nodes = edge.nodes;
        fx_map.insert(pack_pair(nodes[0], nodes[1], hg.n()), nodes[2]);
        fx_map.insert(pack_pair(nodes[0], nodes[2], hg.n()), nodes[1]);
        fx_map.insert(pack_pair(nodes[1], nodes[2], hg.n()), nodes[0]);
    }
    println!("rustc_hash Finished in:  {:?}", time_fx.elapsed());

    // --- Benchmark NoHash ---
    let time_nohash = Instant::now();
    let mut nohash_map: HashMap<u32, NodeId, BuildNoHashHasher<u32>> =
        HashMap::with_capacity_and_hasher(estimated_capacity, BuildNoHashHasher::default());
    for edge in hg.iter_edges::<3>() {
        let nodes = edge.nodes;
        nohash_map.insert(pack_pair(nodes[0], nodes[1], hg.n()), nodes[2]);
        nohash_map.insert(pack_pair(nodes[0], nodes[2], hg.n()), nodes[1]);
        nohash_map.insert(pack_pair(nodes[1], nodes[2], hg.n()), nodes[0]);
    }
    println!("nohash Finished in:      {:?}", time_nohash.elapsed());

    println!(
        "fold_map len: {}, fx_map len: {}, nohash_map len: {}",
        fold_map.len(),
        fx_map.len(),
        nohash_map.len()
    );

    let time_vector = Instant::now();
    let mut triples: Vec<(u32, u32, NodeId)> = Vec::with_capacity(estimated_capacity);
    for edge in hg.iter_edges::<3>() {
        let nodes = edge.nodes;
        triples.push((nodes[0], nodes[1], nodes[2]));
        triples.push((nodes[0], nodes[2], nodes[1]));
        triples.push((nodes[1], nodes[2], nodes[0]));
    }
    triples.sort_unstable_by_key(|&(a, b, _)| (a, b));
    println!("Plain Vector Finished in:{:?}", time_vector.elapsed());

    // lookup: triples.binary_search_by_key(&(a,b), |&(a,b,_)| (a,b))
    // // --- Benchmark Plain Vector (Flat Matrix) ---
    // // Note: Verify if max_node_id() or a similar method exists to get the upper bound of NodeId.
    // // If your NodeId is an alias for u32/usize, cast it appropriately.
    // let matrix_dim = hg.n();
    //
    // let mut flat_matrix = vec![0 as NodeId; matrix_dim * matrix_dim];
    // let time_vector = Instant::now();
    // for edge in hg.iter_edges::<3>() {
    //     let nodes = edge.nodes;
    //     let n0 = nodes[0] as usize;
    //     let n1 = nodes[1] as usize;
    //     let n2 = nodes[2] as usize;
    //
    //     flat_matrix[n0 * matrix_dim + n1] = nodes[2];
    //     flat_matrix[n0 * matrix_dim + n2] = nodes[1];
    //     flat_matrix[n1 * matrix_dim + n2] = nodes[0];
    // }
    // println!("Plain Vector Finished in:{:?}", time_vector.elapsed());

    Ok(())
}

pub fn test2() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .dblp()
        .weighted()
        .load()?;
    hg.normalize_node_ids();

    let adj = HyperAdjList::<NodeWeight>::from_hypergraph_unmapped(hg.0);
    let estimated_capacity = adj.m() * 3;

    let run = || {
        let time = Instant::now();
        let mut map: HashMap<(NodeId, NodeId), Vec<NodeId>> =
            HashMap::with_capacity(estimated_capacity);

        for (_edge_id, edge) in adj.iter_by_size(3) {
            let nodes = edge.nodes;
            const AVG_SIZE: usize = 10;

            map.entry((nodes[0], nodes[1]))
                .and_modify(|v| v.push(nodes[2]))
                .or_insert_with(|| {
                    let mut rv = Vec::with_capacity(AVG_SIZE);
                    rv.push(nodes[2]);
                    rv
                });

            map.entry((nodes[0], nodes[2]))
                .and_modify(|v| v.push(nodes[1]))
                .or_insert_with(|| {
                    let mut rv = Vec::with_capacity(AVG_SIZE);
                    rv.push(nodes[1]);
                    rv
                });

            map.entry((nodes[1], nodes[2]))
                .and_modify(|v| v.push(nodes[0]))
                .or_insert_with(|| {
                    let mut rv = Vec::with_capacity(AVG_SIZE);
                    rv.push(nodes[0]);
                    rv
                });
        }

        let mut gorups_4 = HashSet::<[NodeId; 4]>::with_capacity(4 * adj.m());
        for (&(a, b), extension_nodes) in &mut map {
            let (a, b) = (min(a, b), max(a, b));
            for i in 0..extension_nodes.len() {
                for j in (i + 1)..extension_nodes.len() {
                    let c = extension_nodes[i];
                    let d = extension_nodes[j];
                    let (c, d) = (min(c, d), max(c, d));

                    let a_prime = min(a, c);
                    let c_prime = max(a, c);

                    let b_prime = min(b, d);
                    let d_prime = max(b, d);

                    let middle_low = min(c_prime, b_prime);
                    let middle_high = max(c_prime, b_prime);

                    gorups_4.insert([a_prime, middle_low, middle_high, d_prime]);
                }
            }
        }

        for (_edge_id, edge) in adj.iter_by_size(3) {
            for (_edge_id, edge2) in adj.iter_incident_by_size(edge.nodes[0], 2) {
                let outer = if edge2.nodes[0] == edge.nodes[0] {
                    edge2.nodes[1]
                } else {
                    edge2.nodes[0]
                };

                if outer != edge.nodes[1] && outer != edge.nodes[2] {
                    // gorups_4.insert([edge.nodes[0], edge.nodes[0], edge.nodes[0], outer]);
                }
            }
            for (_edge_id, edge2) in adj.iter_incident_by_size(edge.nodes[1], 2) {
                let outer = if edge2.nodes[0] == edge.nodes[1] {
                    edge2.nodes[1]
                } else {
                    edge2.nodes[0]
                };

                if outer != edge.nodes[0] && outer != edge.nodes[2] {
                    // gorups_4.insert([edge.nodes[1], edge.nodes[1], edge.nodes[1], outer]);
                }
            }
            for (_edge_id, edge2) in adj.iter_incident_by_size(edge.nodes[2], 2) {
                let outer = if edge2.nodes[0] == edge.nodes[2] {
                    edge2.nodes[1]
                } else {
                    edge2.nodes[0]
                };
                if outer != edge.nodes[0] && outer != edge.nodes[1] {
                    // gorups_4.insert([edge.nodes[2], edge.nodes[2], edge.nodes[2], outer]);
                }
            }
        }

        println!("edges_4.len(): {}", gorups_4.len());
        println!("Finished in: {:?}", time.elapsed());
    };
    run();
    Ok(())
}

pub fn test3() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .dblp()
        .weighted()
        .load()?;
    hg.normalize_node_ids();

    let adj = HyperAdjList::<NodeWeight>::from_hypergraph_unmapped(hg.0);

    println!("n: {}, m: {}", adj.n(), adj.m());
    let time = Instant::now();
    let mut edges2 = HashSet::<(NodeId, NodeId)>::with_capacity(adj.count_by_size(2));
    let mut edges3 = HashSet::<(NodeId, NodeId, NodeId)>::with_capacity(adj.count_by_size(3));
    // let mut edge3_ext = HashSet::<(EdgeId, NodeId, NodeId)>::with_capacity(adj.count_by_size(3));

    let mut hyperdiamonds: HashMap<(NodeId, NodeId), Vec<NodeId>> =
        HashMap::with_capacity(3 * adj.m()); // Pre-allocate for 3-uniform edges
    println!("HashSet initialized in: {:?}", time.elapsed());

    let time = Instant::now();
    for (_edge_id, edge) in adj.iter_by_size(3) {
        let nodes = edge.nodes;
        const AVG_SIZE: usize = 10;

        hyperdiamonds
            .entry((nodes[0], nodes[1]))
            .or_insert_with(|| Vec::with_capacity(AVG_SIZE))
            .push(nodes[2]);

        hyperdiamonds
            .entry((nodes[0], nodes[2]))
            .or_insert_with(|| Vec::with_capacity(AVG_SIZE))
            .push(nodes[1]);

        hyperdiamonds
            .entry((nodes[1], nodes[2]))
            .or_insert_with(|| Vec::with_capacity(AVG_SIZE))
            .push(nodes[0])
    }
    println!("Hyperdiamonds populated in: {:?}", time.elapsed());

    let time = Instant::now();
    for (_edge_id, edge) in adj.iter_by_size(2) {
        edges2.insert((edge.nodes[0], edge.nodes[1]));
    }

    for (_edge_id, edge) in adj.iter_by_size(3) {
        edges3.insert((edge.nodes[0], edge.nodes[1], edge.nodes[2]));
    }
    println!("Edges 2 and edge 3 populated in: {:?}", time.elapsed());

    let time = Instant::now();
    let mut map = vec![0; adj.n()];
    // let mut motifs_by_outer_node = vec![(CompactMotif::<4>::zero(), 0); adj.n()];
    let mut exluded = HashSet::<NodeId>::with_capacity(40);

    let mut outer_nodes = HashMap::with_capacity(adj.n() / 2);

    for (_edge_id, pivot_edge) in adj.iter_by_size(3) {
        // outer_nodes.clear();
        exluded.clear();
        for i in 0..2 {
            for j in i..3 {
                hyperdiamonds
                    .entry((pivot_edge.nodes[i], pivot_edge.nodes[j]))
                    .and_modify(|v| {
                        for n in v {
                            exluded.insert(*n);
                        }
                    });
            }
        }

        map[pivot_edge.nodes[0] as usize] = 0;
        map[pivot_edge.nodes[1] as usize] = 1;
        map[pivot_edge.nodes[2] as usize] = 2;

        let mut center_motif = const {
            let mut rv = CompactMotif::<4>::zero();
            rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2]));
            rv
        };
        let mut center_intensity = 0.0;

        macro_rules! extend_with_edge2 {
            ($pivot: literal, $a: literal, $b: literal) => {
                for (_edge_id, edge) in adj.iter_incident_by_size(pivot_edge.nodes[$pivot], 2) {
                    let outer = if edge.nodes[0] == pivot_edge.nodes[$pivot] {
                        edge.nodes[0]
                    } else {
                        edge.nodes[1]
                    };

                    if exluded.contains(&outer) {
                        continue;
                    }

                    // 0 if node is contained in center  i + 1 if connected the ith node of pivot edge
                    let other_inner = ((outer == pivot_edge.nodes[$a]) as usize) * ($a + 1)
                        + ((outer == pivot_edge.nodes[$b]) as usize) * ($b + 1);

                    if other_inner == 0 {
                        let outer_motif = outer_nodes
                            .entry(outer)
                            .or_insert((CompactMotif::<4>::zero(), 1.0));
                        outer_motif.0.const_bitor(
                                const {
                                    let mut rv = CompactMotif::<4>::zero();
                                    rv.const_add_edge_with_nodes(CompressedNodeSet::from_array([
                                        $pivot, 3,
                                    ]));
                                    rv
                                },
                            );
                        outer_motif.1 *= edge.weight;
                    } else {
                        center_intensity *=
                            (($pivot < (other_inner - 1)) as usize as f32) * edge.weight;
                        center_motif.add_edge_with_nodes(CompressedNodeSet::from_array([
                            $pivot,
                            (other_inner - 1) as u8,
                        ]));
                    }
                }

            };
        }

        extend_with_edge2!(0, 1, 2);
        extend_with_edge2!(1, 0, 2);
        extend_with_edge2!(2, 0, 1);

        for (_outer_node, (_motif, _intensity)) in outer_nodes.iter() {
            //
        }

        // println!("{}", center_intensity);
    }

    println!("Edge3 extensions computed in: {:?}", time.elapsed());

    let time = Instant::now();
    let mut count = 0;
    for (_edge_id, edge) in adj.iter_by_size(4) {
        count += edges2.contains(&(edge.nodes[0], edge.nodes[1])) as usize;
        count += edges2.contains(&(edge.nodes[0], edge.nodes[2])) as usize;
        count += edges2.contains(&(edge.nodes[0], edge.nodes[3])) as usize;
        count += edges2.contains(&(edge.nodes[1], edge.nodes[2])) as usize;
        count += edges2.contains(&(edge.nodes[1], edge.nodes[3])) as usize;
        count += edges2.contains(&(edge.nodes[2], edge.nodes[3])) as usize;

        count += edges3.contains(&(edge.nodes[0], edge.nodes[1], edge.nodes[2])) as usize;
        count += edges3.contains(&(edge.nodes[0], edge.nodes[1], edge.nodes[3])) as usize;
        count += edges3.contains(&(edge.nodes[0], edge.nodes[2], edge.nodes[3])) as usize;
        count += edges3.contains(&(edge.nodes[1], edge.nodes[2], edge.nodes[3])) as usize;
    }
    println!("order 4 done in {:?}", time.elapsed());
    println!("count: {}", count);

    Ok(())
}

pub fn test4() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .dblp()
        .weighted()
        .load()?;
    hg.normalize_node_ids();

    let adj = HyperAdjList::<NodeWeight>::from_hypergraph_unmapped(hg.0);

    println!("n: {}, m: {}", adj.n(), adj.m());
    let run = || {
        let mut groups4 = vec![HashSet::new(); adj.m()];
        groups4[0].insert(0);

        // let mut curr = 0;
        // // let mut groups4 = vec![Vec::with_capacity(40); adj.n()];
        let mut groups4: HashSet<[NodeId; 4]> =
            HashSet::with_capacity(adj.count_by_size(3) + adj.count_by_size(4));

        let mut extension_nodes_map = vec![(CompactMotif::<4>::zero(), 1.0, 1.0); adj.m()];
        let mut extension_nodes_list = vec![0; adj.n()];
        let mut inserted = BitSet::with_capacity(adj.n());

        for (pivot_edge_id, pivot_edge) in adj.iter_by_size(3) {
            extension_nodes_list.clear();
            let mut center_motif = CompactMotif::<4>::zero();
            let mut center_intensity = 1.0;
            center_motif.add_edge_with_nodes(CompressedNodeSet::from_array([0, 1, 2]));
            for i in 0..3 {
                for (_edge_id, edge) in adj.iter_incident_by_size(pivot_edge.nodes[i], 2) {
                    let non_pivot = if edge.nodes[0] == pivot_edge.nodes[i] {
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

                    if is_inner && pivot_edge.nodes[i] < non_pivot {
                        center_motif.add_edge_with_nodes(CompressedNodeSet::from_array([
                            i as u8,
                            inner_index,
                        ]));
                        center_intensity *= edge.weight;
                    } else {
                        if !inserted.contains(non_pivot as usize) {
                            inserted.insert(non_pivot as usize);
                            extension_nodes_list.push(non_pivot);
                        }

                        extension_nodes_map[non_pivot as usize]
                            .0
                            .add_edge_with_nodes(CompressedNodeSet::from_array([i as u8, 3]));

                        extension_nodes_map[non_pivot as usize].1 *= edge.weight;
                    }
                }

                for (edge_id, edge) in adj.iter_incident_by_size(pivot_edge.nodes[i], 3) {
                    if pivot_edge_id == edge_id {
                        continue;
                    }

                    // let nodes = if edge.nodes[0] == pivot_edge.nodes[i] {
                    //     [edge.nodes[0], edge.nodes[1], edge.nodes[2]]
                    // } else if edge.nodes[1] == pivot_edge.nodes[i] {
                    //     [edge.nodes[1], edge.nodes[0], edge.nodes[2]]
                    // } else {
                    //     [edge.nodes[2], edge.nodes[0], edge.nodes[1]]
                    // };

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
                        let (_inner_node, inner_index) = if inner[0].0 == pivot_edge.nodes[i] {
                            inner[1]
                        } else {
                            inner[0]
                        };

                        if !inserted.contains(outer as usize) {
                            inserted.insert(outer as usize);
                            extension_nodes_list.push(outer);
                        }

                        extension_nodes_map[outer as usize].0.add_edge_with_nodes(
                            CompressedNodeSet::from_array([i as u8, inner_index, 3]),
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

                let _c2 = center_motif.filtered_by_order(2).edge_count()
                    + extension_nodes_map[outer as usize]
                        .0
                        .filtered_by_order(2)
                        .edge_count();

                let _c3 = extension_nodes_map[outer as usize]
                    .0
                    .filtered_by_order(3)
                    .edge_count()
                    + 1;

                let _i2 = extension_nodes_map[outer as usize].1 * center_intensity;
                let _i3 = extension_nodes_map[outer as usize].2 * pivot_edge.weight;

                extension_nodes_map[outer as usize] = (CompactMotif::<4>::zero(), 1.0, 1.0);
                inserted.remove(outer as usize);

                groups4.insert(sorted_group4);
            }

            // groups4.insert((
            //     pivot_edge.nodes[0],
            //     pivot_edge.nodes[1],
            //     pivot_edge.nodes[2],
            // ));
            // for (_edge_id, edge) in adj.iter_incident_by_size(pivot_edge.nodes[i], 2) {
            // let outer = if edge.nodes[0] == pivot_edge.nodes[i] {
            //     edge.nodes[1]
            // } else {
            //     edge.nodes[0]
            // };
            //
            // if outer == pivot_edge.nodes[1] || outer == pivot_edge.nodes[2] {
            //     // inner edge
            // } else {
            //     // groups4[outer as usize].push(curr);
            //     // groups4[pivot_edge.nodes[0] as usize].push(curr);
            //     // groups4[pivot_edge.nodes[1] as usize].push(curr);
            //     // groups4[pivot_edge.nodes[2] as usize].push(curr);
            // }
            // }
        }
    };

    let time = Instant::now();
    run();

    // let mut sum = 0;
    // for group in &groups4 {
    //     sum += group.len();
    // }
    // println!("med: {}", sum / groups4.len());

    println!("Vec populated in {:?}", time.elapsed());

    Ok(())
}
