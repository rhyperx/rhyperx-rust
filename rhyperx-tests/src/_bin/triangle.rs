use std::error::Error;

use rust_core::{
    loader::DatasetLoader,
    misc::degeneracy_ordering,
    triangle::forward::{forward_hashed_cloj, forward_sorted_cloj},
    types::{
        NodeId,
        adj_list::{AdjList, Undirected, WithIncidence},
    },
};

pub fn main() -> Result<(), Box<dyn Error>> {
    let hg = DatasetLoader::builder()
        .cached(true)
        .dblp()
        .unweighted()
        .load()?;
    let edges_2: Vec<(NodeId, NodeId, ())> =
        hg.0.edges::<2>()
            .iter()
            .cloned()
            .map(|e| (e.nodes[0], e.nodes[1], ()))
            .collect();

    let (mut adj, _, _) = AdjList::<(), Undirected, WithIncidence>::from_edges_mapped(edges_2);
    let (mut order, _) = degeneracy_ordering(&adj);
    order.reverse();

    let time1 = std::time::Instant::now();
    let mut count1 = 0;
    forward_hashed_cloj(&adj, Some(&order), |_a, _b, _c| {
        count1 += 1;
    });
    let elapsed1 = time1.elapsed();

    let time2 = std::time::Instant::now();
    let mut count2 = 0;
    forward_sorted_cloj(&mut adj, Some(&order), |_, _| {
        count2 += 1;
    });
    let elapsed2 = time2.elapsed();

    let time3 = std::time::Instant::now();
    let mut count3 = 0;
    forward_sorted_cloj(&mut adj, None, |_, _| {
        count3 += 1;
    });
    let elapsed3 = time3.elapsed();

    println!("Count1: {}, Count2: {}", count1, count2);

    println!(
        "forward_hashed_cloj time: {:?}, forward_hashed_cloj_orient: {:?}, forward_hashed_cloj_orient (natural order): {:?}",
        elapsed1, elapsed2, elapsed3
    );
    assert_eq!(count1, count2);

    Ok(())
}
