use std::{error::Error, time::Instant};

use hashbrown::HashSet;
use rust_core::{
    loader::DatasetLoader,
    misc::{hyper_degeneracy_ordering, hyper_inclusion_forest::inclusion_forest},
    types::{Hx, Hypergraph, hyperadj_list::HyperAdjList},
};

pub fn main() -> Result<(), Box<dyn Error>> {
    dblp()?;

    Ok(())
}

pub fn small() {
    let mut hg = Hypergraph::new();

    hg.extend_with_edges(vec![Hx::new_unchecked([0, 3], ())]);

    hg.extend_with_edges(vec![Hx::new_unchecked([0, 3, 4], ())]);

    hg.extend_with_edges(vec![
        Hx::new_unchecked([0, 1, 2, 3], ()),
        Hx::new_unchecked([0, 2, 3, 4], ()),
    ]);

    let mut adj = HyperAdjList::from_hypergraph_unmapped(hg);
    let (order_pos, _deg) = hyper_degeneracy_ordering(&adj);

    let forest = inclusion_forest(&mut adj, Some(&order_pos));
    println!("Inclusion forest: {:?}", forest);
}

pub fn dblp() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .dblp()
        .unweighted()
        .load()?;
    hg.normalize_node_ids();
    seq_macro::seq!(N in 5..=10 {
        hg.take_edges::<N>();
    });

    let time = Instant::now();
    {
        let mut edges2 = HashSet::new();
        let mut edges3 = HashSet::new();
        for edge in hg.iter_edges::<2>() {
            edges2.insert(edge);
        }

        for edge in hg.iter_edges::<3>() {
            edges3.insert(edge);
        }

        for edge in hg.iter_edges::<4>() {
            let nodes = edge.nodes;

            edges2.contains(&Hx::new_unchecked([nodes[0], nodes[1]], ()));
            edges2.contains(&Hx::new_unchecked([nodes[0], nodes[2]], ()));
            edges2.contains(&Hx::new_unchecked([nodes[0], nodes[3]], ()));
            edges2.contains(&Hx::new_unchecked([nodes[1], nodes[2]], ()));
            edges2.contains(&Hx::new_unchecked([nodes[1], nodes[3]], ()));
            edges2.contains(&Hx::new_unchecked([nodes[2], nodes[3]], ()));

            edges3.contains(&Hx::new_unchecked([nodes[0], nodes[1], nodes[2]], ()));
            edges3.contains(&Hx::new_unchecked([nodes[0], nodes[2], nodes[3]], ()));
            edges3.contains(&Hx::new_unchecked([nodes[0], nodes[1], nodes[3]], ()));
            edges3.contains(&Hx::new_unchecked([nodes[1], nodes[2], nodes[3]], ()));
        }
    }
    println!("Edge containment check time: {:?}", time.elapsed());

    let mut adj = HyperAdjList::from_hypergraph_unmapped(hg.0);

    let time = Instant::now();
    let (order_pos, _deg) = hyper_degeneracy_ordering(&adj);
    println!("Hyper degeneracy compute time: {:?}", time.elapsed());

    let time = Instant::now();
    let _forest = inclusion_forest(&mut adj, Some(&order_pos));
    println!("Inclusion forest compute time: {:?}", time.elapsed());

    // println!("Inclusion forest: {:?}", forest);

    Ok(())
}
