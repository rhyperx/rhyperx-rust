use std::{error::Error, time::Instant};

use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};
use rhyperx::algo::misc::sorting::{degeneracy_ordering, hyper_degeneracy_ordering};
use rhyperx::graph::{AdjList, Undirected};
use rhyperx::hypergraph::static_adj_list::StaticAdjList;
use rhyperx::hypergraph::{HyperedgeContainer, Hypergraph, SizedHx};
use rhyperx::io::DatasetLoader;
use seq_macro::seq;

pub fn main() -> Result<(), Box<dyn Error>> {
    // degeneracy_small()?;
    degeneracy_big()?;
    // degeneracy_random_hypergraphs(50000, 10, 10);
    Ok(())
}

pub fn degeneracy_small() -> Result<(), Box<dyn Error>> {
    let mut hg: Hypergraph<u32, ()> = Hypergraph::new();
    // 2-uniform edges (cross-connections)

    seq!(N in 3..11 {
        hg.take_edges(N);
    });
    hg.remove_isolated_nodes();
    hg.normalize_node_ids();

    test_common(hg)?;

    Ok(())
}

pub fn degeneracy_big() -> Result<(), Box<dyn Error>> {
    let time = Instant::now();
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .dblp()
        .unweighted()
        .load()?;
    println!("Loaded in: {:?}", time.elapsed());
    seq!(N in 3..11 {
        hg.take_edges(N);
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

fn test_common<W: Clone + PartialEq>(mut hg: Hypergraph<u32, W>) -> Result<(), Box<dyn Error>> {
    seq!(N in 3..11 {
        hg.take_edges(N);
    });
    hg.remove_isolated_nodes();
    hg.normalize_node_ids();

    println!("n: {}, m: {}", hg.n(), hg.m());
    println!("Hyperedge distribution: ");
    seq!(N in 2..11 {
        println!("{}-edges: {}", N, hg.edges(N).map_or(0, |c| c.len()));
    });

    println!();
    let time = Instant::now();
    let (adj1, _, _) = AdjList::<u32, (), Undirected>::from_edges_mapped(
        hg.iter_edges(2)
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
    let adj2 = StaticAdjList::<u32, u32, W>::from_hypergraph_unmapped(hg.clone());
    println!("Created HyperAdjacencyList in {:?}", time.elapsed());
    println!("adj2: {}, {}", adj2.n(), adj2.m());

    let time = Instant::now();
    let (_, deg2) = hyper_degeneracy_ordering(&adj2);
    println!("Degeneracy: {}", deg2);
    println!("Computed degeneracy ordering in {:?}", time.elapsed());

    if deg1 != deg2 {
        for e in hg.iter_edges(2) {
            println!("{}-{}", e.nodes[0], e.nodes[1]);
        }
        panic!("Found incoherent degeneracy")
    }

    Ok(())
}

pub fn generate_random_hypergraph(num_nodes: usize, max_total_edges: usize) -> Hypergraph<u32, ()> {
    let mut hg: Hypergraph<u32, ()> = Hypergraph::new();
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
                let arr: [u32; 2] = [edge[0] as u32, edge[1] as u32];
                edges2.push(SizedHx::<2, u32, ()>::new(arr, ()).expect("Malformed edge"));
            }
        }
        hg.extend_with_edges_sized::<2>(edges2);
        edges_left -= count;
    }

    // 2. Generate 3-uniform edges
    if edges_left > 0 {
        let count = rng.random_range(1..=edges_left);
        let mut edges3 = Vec::new();
        for _ in 0..count {
            if let Some(edge) = pick_unique_nodes(&nodes, 3, &mut rng) {
                let arr: [u32; 3] = [edge[0] as u32, edge[1] as u32, edge[2] as u32];
                edges3.push(SizedHx::<3, u32, ()>::new(arr, ()).expect("Malformed edge"));
            }
        }
        hg.extend_with_edges_sized::<3>(edges3);
        edges_left -= count;
    }

    // 3. Generate 4-uniform edges
    if edges_left > 0 {
        let mut edges4 = Vec::new();
        for _ in 0..edges_left {
            // Use up the remaining budget
            if let Some(edge) = pick_unique_nodes(&nodes, 4, &mut rng) {
                let arr: [u32; 4] = [
                    edge[0] as u32,
                    edge[1] as u32,
                    edge[2] as u32,
                    edge[3] as u32,
                ];
                edges4.push(SizedHx::<4, u32, ()>::new(arr, ()).expect("Malformed edge"));
            }
        }
        hg.extend_with_edges_sized::<4>(edges4);
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
