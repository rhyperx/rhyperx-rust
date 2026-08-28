use foldhash::fast::FixedState;
use hashbrown::HashMap;

use crate::{
    misc::OrderAndPos,
    types::{EdgeId, NodeId, hyperadj_list::HyperAdjList},
};
use std::{borrow::Cow, ops::Deref};

/// Returns the inclusion forest of a hypergraph represented as a hyper adjacency list.
/// If order is None, the order is assumed to be the natural order of the nodes (0..n). Hypergraph
/// degeneracy is the preferred order for this algorithm, but it can be slow to compute for
/// hypergraphs with large hyperedges, hence any order can be used (e.g. degree sort is somethimes
/// good enough).
fn inclusion_forest_base<W: Clone>(
    adj: &HyperAdjList<W>,
    order: Option<&OrderAndPos>,
) -> Vec<Vec<EdgeId>> {
    let order_pos = match order {
        Some(o) => Cow::Borrowed(o),
        None => {
            let n = adj.n();
            let natural_order = (0..n).map(|i| i as NodeId).collect::<Vec<_>>();
            let natural_pos = (0..n).collect::<Vec<_>>();

            Cow::Owned(OrderAndPos::new(natural_order, natural_pos, true))
        }
    };

    let oriented = adj.get_oriented(&order_pos);
    // let oriented = adj;
    // println!("Oriented {:?}", oriented);

    let mut size_map = HashMap::with_hasher(FixedState::default());

    for i in 0..adj.n() {
        for (edge_id, edge_ref) in oriented.iter_incident_edges(i as NodeId) {
            for n in edge_ref.nodes {
                size_map
                    .entry(edge_ref.nodes.len())
                    .and_modify(|e: &mut Vec<NodeId>| e.push(*n))
                    .or_insert(vec![*n]);
            }
        }
    }
    let mut size_map = size_map
        .into_iter()
        .map(|(size, v)| (size, v))
        .collect::<Vec<_>>();
    size_map.sort_unstable_by(|e1, e2| e2.0.cmp(&e1.0));

    let mut marks = vec![Vec::new(); adj.n()];
    let mut sizes: Vec<Vec<u8>> = vec![Vec::new(); adj.n()];
    let mut ranges: Vec<Vec<u16>> = vec![Vec::new(); adj.n()];
    let mut rv = vec![Vec::new(); adj.m()];

    // keeps track of where incident edges iteration was left for each node
    let mut indexes = vec![0; adj.n()];

    // for each n consider the set of oriented incident hyperedges
    // let mut oriented_iter = (0..oriented.n())
    //     .map(|i| oriented.iter_incident_edges(i as NodeId))
    //     .collect::<Vec<_>>();

    for (size, nodes) in size_map {
        let time = std::time::Instant::now();
        for &n in nodes.iter() {
            loop {
                if !(indexes[n as usize] < oriented[n].len()) {
                    break;
                }

                let edge_id = oriented[n][indexes[n as usize]];
                let edge = oriented.csr.get_edge_by_id(edge_id);

                if edge.nodes.len() != size {
                    break;
                }

                indexes[n as usize] += 1;

                for &n in edge.nodes {
                    marks[n as usize].push(edge_id);
                    match sizes[n as usize].last() {
                        Some(last_size) => {
                            if *last_size == size as u8 {
                                *ranges[n as usize].last_mut().unwrap() += 1;
                            } else {
                                let next_range = ranges[n as usize].last().unwrap() + 1;
                                sizes[n as usize].push(size as u8);
                                ranges[n as usize].push(next_range);
                            }
                        }
                        None => {
                            // used to keep the 2 vec aligned and later to iterate mor easily
                            sizes[n as usize].push(0);
                            sizes[n as usize].push(size as u8);
                            ranges[n as usize].push(0);
                            ranges[n as usize].push(1);
                        }
                    }
                }
                // println!("");
                // println!("Marks:\t {:?}", marks);
                // println!("Sizes:\t {:?}", sizes);
                // println!("Ranges:\t {:?}", sizes);

                // check inclusion in bigger hyperedges (we don't check for auto-inclusions so we start
                // at order + 1)
                //

                // since edge_id sorting is only guaranteed inside hyperedges with same size we need a 2
                // step approach, checking each size at a time
                // 1 steps: find shared sizes between nodes of current hyperedge
                let mut pointers = vec![0; edge.nodes.len()];
                let mut found = true;
                for (pos, &n) in edge.nodes.iter().enumerate() {
                    let n = n as usize;

                    if sizes[n].len() < 3 {
                        found = false;
                        break;
                    }

                    // -2 since the last size if always the current one; we decided to ignore auto inclusions
                    pointers[pos] = sizes[n].len() as usize - 2;
                    // print!("{} ", pointers[pos]);
                }
                // println!("Found {:?}", found);

                if found {
                    let nodes = edge.nodes;
                    let mut curr_max = (0..nodes.len())
                        .map(|i| sizes[nodes[i] as usize][pointers[i]])
                        .max()
                        .unwrap();
                    // println!("Curr_max {}", curr_max);
                    let mut curr_pos = 0;
                    let mut matched_buckets = 0;

                    loop {
                        if curr_pos == edge.nodes.len() {
                            // Found common
                            // println!("hx {} contained in hxs with size{:?}", edge_id, curr_max);

                            {
                                // Part 2: get offsets for current common edge size and find common ids
                                // with sorted lists. Can go forward
                                let mut inner_pointers = vec![0; edge.nodes.len()];
                                let mut inner_pointers_end = vec![0; edge.nodes.len()];
                                let mut inner_curr_pos = 0;

                                for (pos, &n) in edge.nodes.iter().enumerate() {
                                    let n = n as usize;
                                    inner_pointers[pos] = ranges[n][pointers[pos] - 1] as usize;
                                    inner_pointers_end[pos] = ranges[n][pointers[pos]] as usize; // Right bounds are not inclusive
                                }
                                // println!("inner_pointer_starts: {:?}", inner_pointers);
                                // println!("inner_pointer_ends: {:?}", inner_pointers_end);

                                let mut inner_curr_max = (0..nodes.len())
                                    .map(|i| marks[nodes[i] as usize][inner_pointers[i]])
                                    .max()
                                    .unwrap();

                                loop {
                                    // println!("inner_curr_pos: {}", inner_curr_pos);
                                    // println!("inner_pointer: {:?}", inner_pointers);
                                    if inner_curr_pos == edge.nodes.len() {
                                        // println!("INCLUSION FOUND");
                                        // Found inclusion
                                        rv[inner_curr_max as usize].push(edge_id);

                                        for i in 0..nodes.len() {
                                            inner_pointers[i] += 1;
                                        }
                                        inner_curr_max = (0..nodes.len())
                                            .map(|i| marks[nodes[i] as usize][inner_pointers[i]])
                                            .max()
                                            .unwrap();
                                        inner_curr_pos = 0;
                                    }

                                    while inner_pointers[inner_curr_pos]
                                        < inner_pointers_end[inner_curr_pos]
                                        && marks[nodes[inner_curr_pos] as usize]
                                            [inner_pointers[inner_curr_pos]]
                                            < inner_curr_max
                                    {
                                        inner_pointers[inner_curr_pos] += 1;
                                    }
                                    if inner_pointers[inner_curr_pos]
                                        == inner_pointers_end[inner_curr_pos]
                                    {
                                        // one bucket is over
                                        break;
                                    }

                                    if marks[nodes[inner_curr_pos] as usize]
                                        [inner_pointers[inner_curr_pos]]
                                        == inner_curr_max
                                    {
                                        inner_curr_pos += 1;
                                    } else {
                                        inner_curr_max = marks[nodes[inner_curr_pos] as usize]
                                            [inner_pointers[inner_curr_pos]];
                                        inner_curr_pos = 0;
                                    }
                                }
                            }

                            for i in 0..nodes.len() {
                                pointers[i] -= 1;
                            }
                            curr_max = (0..nodes.len())
                                .map(|i| sizes[nodes[i] as usize][pointers[i]])
                                .max()
                                .unwrap();
                            curr_pos = 0;
                            // println!("curr_pos: {}", curr_pos);
                        }

                        // println!("curr_pos: {}", curr_pos);
                        while pointers[curr_pos] > 0
                            && sizes[nodes[curr_pos] as usize][pointers[curr_pos]] < curr_max
                        {
                            pointers[curr_pos] -= 1;
                        }
                        if pointers[curr_pos] == 0 {
                            // one bucket is over
                            break;
                        }

                        if sizes[nodes[curr_pos] as usize][pointers[curr_pos]] == curr_max {
                            curr_pos += 1;
                        }
                        // else if pointers[curr_pos] == 1 {
                        //     // one bucket is over
                        //     break;
                        // }
                        else {
                            // pointers[curr_pos] -= 1;
                            curr_max = sizes[nodes[curr_pos] as usize][pointers[curr_pos]];
                            curr_pos = 0;
                        }
                    }

                    // check for common sizes between nodes of the hyperedge
                }
            }
        }

        println!("Finished size {} in {:?}", size, time.elapsed());
    }

    rv
}

pub fn inclusion_forest<W: Clone>(
    adj: &mut HyperAdjList<W>,
    order_pos: Option<&OrderAndPos>,
) -> Vec<Vec<EdgeId>> {
    // sorting each incident list by increasing hyperedge size
    {
        let HyperAdjList { csr, adj, .. } = adj;

        // Sort by hyperedge size; hyperedges with the same size will be sorted by their edge id
        for incident in adj.iter_mut() {
            incident.sort_by(|e1, e2| {
                let size1 = csr.lookup[*e1 as usize].1;
                let size2 = csr.lookup[*e2 as usize].1;
                size2.cmp(&size1).then(e1.cmp(e2))
            });
        }
    }
    // println!("{:?}", adj);

    inclusion_forest_base(adj, order_pos)
}

// pub fn inclusion_forest_no_sort<W: Clone>(
//     adj: &HyperAdjList<W>,
//     order: Option<(&[NodeId], &[usize])>,
// ) -> Vec<Vec<usize>> {
//     inclusion_forest_base(adj, order)
// }
