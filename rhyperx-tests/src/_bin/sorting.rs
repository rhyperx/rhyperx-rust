use std::{error::Error, time::Instant};

use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};
use rust_core::{
    loader::DatasetLoader,
    misc::{degeneracy_ordering, hyper_degeneracy_ordering},
    types::{
        Hx, Hypergraph, NodeId,
        adj_list::{AdjList, Undirected, WithoutIncidence},
        hyperadj_list::HyperAdjListBase,
    },
};
use seq_macro::seq;

pub fn main() -> Result<(), Box<dyn Error>> {
    // degeneracy_small()?;
    degeneracy_big()?;
    // degeneracy_random_hypergraphs(50000, 10, 10);
    Ok(())
}

pub fn degeneracy_small() -> Result<(), Box<dyn Error>> {
    let mut hg: Hypergraph<NodeId, ()> = Hypergraph::new();
    // 2-uniform edges (cross-connections)

    seq!(N in 3..11 {
        hg.take_edges::<N>();
    });
    hg.remove_isolated_nodes();
    hg.normalize_node_ids();

    // println!("Hypergraph: {:?}", hg);

    test_common(hg)?;

    Ok(())
}

pub fn degeneracy_big() -> Result<(), Box<dyn Error>> {
    let time = Instant::now();
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .dblp()
        .unweighted()
        .load()?
        .0;
    println!("Loaded in: {:?}", time.elapsed());
    seq!(N in 3..11 {
        hg.take_edges::<N>();
    });
    hg.remove_isolated_nodes();
    hg.normalize_node_ids();

    test_common(hg)?;

    Ok(())
}

pub fn degeneracy_random_hypergraphs(
    count: usize,
    n: usize,
    m: usize,
) -> Result<(), Box<dyn Error>> {
    for _ in 0..count {
        let hg = generate_random_hypergraph(n, m);
        test_common(hg)?;
    }
    Ok(())
}

fn test_common<W: Clone>(mut hg: Hypergraph<NodeId, W>) -> Result<(), Box<dyn Error>> {
    seq!(N in 3..11 {
        hg.take_edges::<N>();
    });
    hg.remove_isolated_nodes();
    hg.normalize_node_ids();

    println!("n: {}, m: {}", hg.n(), hg.m());
    println!("Hyperedge distribution: ");
    seq!(N in 2..11 {
        println!("{}-edges: {}", N, hg.edges::<N>().len());
    });

    println!();
    let time = Instant::now();
    let (adj1, _, _) = AdjList::<(), Undirected, WithoutIncidence>::from_edges_mapped(
        hg.edges::<2>()
            .iter()
            .map(|e| (e.nodes[0], e.nodes[1], ()))
            .collect(),
    );
    println!("Created AdjacencyList in {:?}", time.elapsed());
    println!("adj1: {}, {}", adj1.n(), adj1.m());

    let time = Instant::now();
    let (_, deg1) = degeneracy_ordering(&adj1);
    println!("Degeneracy: {}", deg1);
    println!(
        "Computed 2-uniform degeneracy ordering in {:?}",
        time.elapsed()
    );

    println!();
    let time = Instant::now();
    let adj2 = HyperAdjListBase::from_hypergraph_unmapped(hg.clone());
    println!("Created HyperAdjacencyList in {:?}", time.elapsed());
    println!("adj2: {}, {}", adj2.n(), adj2.m());

    let time = Instant::now();
    let (_, deg2) = hyper_degeneracy_ordering(&adj2);
    println!("Degeneracy: {}", deg2);
    println!("Computed degeneracy ordering in {:?}", time.elapsed());

    if deg1 != deg2 {
        for e in hg.edges::<2>() {
            println!("{}-{}", e.nodes[0], e.nodes[1]);
        }
        panic!("Found incoherent degeneracy")
    }

    Ok(())
}

pub fn generate_random_hypergraph(
    num_nodes: usize,
    max_total_edges: usize,
) -> Hypergraph<NodeId, ()> {
    let mut hg: Hypergraph<NodeId, ()> = Hypergraph::new();
    let mut rng = rand::rng();

    // Create a pool of available node indices
    let nodes: Vec<usize> = (0..num_nodes).collect();

    // Distribute total edge budget among sizes 2, 3, and 4
    let mut edges_left = max_total_edges;

    // 1. Generate 2-uniform edges
    if edges_left > 0 {
        let count = rng.random_range(1..=edges_left);
        let mut edges2 = Vec::new();
        for _ in 0..count {
            if let Some(edge) = pick_unique_nodes(&nodes, 2, &mut rng) {
                // Hx::new_unchecked expected an array: [0, 1]
                let arr: [NodeId; 2] = [edge[0] as NodeId, edge[1] as NodeId];
                edges2.push(Hx::new(arr, ()).expect("Malformed edge"));
            }
        }
        hg.extend_with_edges::<2>(edges2);
        edges_left -= count;
    }

    // 2. Generate 3-uniform edges
    if edges_left > 0 {
        let count = rng.random_range(1..=edges_left);
        let mut edges3 = Vec::new();
        for _ in 0..count {
            if let Some(edge) = pick_unique_nodes(&nodes, 3, &mut rng) {
                let arr: [NodeId; 3] = [edge[0] as NodeId, edge[1] as NodeId, edge[2] as NodeId];
                edges3.push(Hx::new(arr, ()).expect("Malformed edge"));
            }
        }
        hg.extend_with_edges::<3>(edges3);
        edges_left -= count;
    }

    // 3. Generate 4-uniform edges
    if edges_left > 0 {
        let mut edges4 = Vec::new();
        for _ in 0..edges_left {
            // Use up the remaining budget
            if let Some(edge) = pick_unique_nodes(&nodes, 4, &mut rng) {
                let arr: [NodeId; 4] = [
                    edge[0] as NodeId,
                    edge[1] as NodeId,
                    edge[2] as NodeId,
                    edge[3] as NodeId,
                ];
                edges4.push(Hx::new(arr, ()).expect("Malformed edge"));
            }
        }
        hg.extend_with_edges::<4>(edges4);
    }

    hg
}

/// Helper function to randomly sample `k` unique nodes from the node pool
fn pick_unique_nodes(nodes: &[usize], k: usize, rng: &mut impl Rng) -> Option<Vec<usize>> {
    if nodes.len() < k {
        return None;
    }
    let mut sampled = nodes.sample(rng, k).cloned().collect::<Vec<usize>>();
    sampled.sort_unstable();
    Some(sampled)
}
