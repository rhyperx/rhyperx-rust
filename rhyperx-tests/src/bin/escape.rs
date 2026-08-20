use std::error::Error;
use std::sync::LazyLock;

use rust_core::loader::DatasetLoader;
use rust_core::misc::cycle::intensity_c4_subinc;
use rust_core::motifs::algorithms::escape::{self, unweighted_4, weighted_4};
use rust_core::types::adj_list::{AdjList, Undirected, WithoutIncidence};
use rust_core::types::hyperadj_list::HyperAdjList;
use rust_core::types::{Hx, Hypergraph, NodeWeight};
use rust_core_tests::shared::graphlets::*;
use seq_macro::seq;

pub fn main() -> Result<(), Box<dyn Error>> {
    // simple_graph_count();
    // simple_graph_intensity();
    // all_simple_graphlets();

    // dblp_uw()?;
    // dblp_w()?;

    // hospital_uw3()?;
    hospital_w3()?;
    // hospital_uw4()?;
    // hospital_w()?;
    // friendship_hs()?;
    // simple_c4_w();

    Ok(())
}

const SIMPLE_UNWEIGHTED: LazyLock<AdjList<(), Undirected, WithoutIncidence>> =
    LazyLock::new(|| {
        let edges = vec![(0, 1), (1, 2), (2, 3), (3, 0), (2, 0), (0, 4), (4, 2)]
            .into_iter()
            .map(|(u, v)| (u, v, ()))
            .collect::<Vec<_>>();

        let (adj, _, _) = AdjList::from_edges_mapped(edges);

        adj
    });

const SIMPLE_WEIGHTED: LazyLock<HyperAdjList<NodeWeight>> = LazyLock::new(|| {
    let mut hg = Hypergraph::new();
    //
    hg.extend_with_edges(vec![
        Hx::new_unchecked([0, 1], 1.0),
        Hx::new_unchecked([0, 2], 1.0),
        Hx::new_unchecked([0, 3], 1.0),
        Hx::new_unchecked([1, 2], 1.0),
        Hx::new_unchecked([1, 3], 1.0),
        Hx::new_unchecked([2, 3], 1.0),
        // Hx::new_unchecked([0, 4], 1.0),
        // Hx::new_unchecked([1, 4], 1.0),
    ]);
    //
    // hg.extend_with_edges(vec![
    //     Hx::new_unchecked([0, 1], 1.0),
    //     Hx::new_unchecked([1, 2], 1.0),
    //     Hx::new_unchecked([2, 3], 1.0),
    //     Hx::new_unchecked([0, 3], 1.0),
    // ]);

    // hg.extend_with_edges(vec![
    //     // Hx::new_unchecked([0, 2, 3], ()),
    //     // Hx::new_unchecked([1, 2, 3], ()),
    //     Hx::new_unchecked([0, 1, 2], 1.0),
    //     Hx::new_unchecked([0, 1, 3], 1.0),
    // ]);

    HyperAdjList::from_hypergraph_unmapped(hg)
});

const SIMPLE_UNWEIGHTED_HYPERGRAPH: LazyLock<HyperAdjList<()>> = LazyLock::new(|| {
    let mut hg = Hypergraph::new();

    hg.extend_with_edges(vec![
        // Hx::new_unchecked([0, 2, 3], ()),
        // Hx::new_unchecked([1, 2, 3], ()),
        Hx::new_unchecked([0, 1, 2], ()),
        Hx::new_unchecked([0, 1, 3], ()),
    ]);

    // hg.extend_with_edges(vec![
    //     Hx::new_unchecked([2, 3], ()),
    //     Hx::new_unchecked([0, 1], ()),
    // ]);

    // // K4
    // hg.extend_with_edges(vec![
    //     Hx::new_unchecked([0, 1], ()),
    //     Hx::new_unchecked([0, 2], ()),
    //     Hx::new_unchecked([0, 3], ()),
    //     Hx::new_unchecked([1, 2], ()),
    //     Hx::new_unchecked([1, 3], ()),
    //     Hx::new_unchecked([2, 3], ()),
    // ]);
    //
    // // Diamond
    // hg.extend_with_edges(vec![
    //     Hx::new_unchecked([0, 1], ()),
    //     Hx::new_unchecked([0, 2], ()),
    //     Hx::new_unchecked([0, 3], ()),
    //     Hx::new_unchecked([1, 2], ()),
    //     Hx::new_unchecked([1, 3], ()),
    // ]);
    //
    // // Tailed triangle
    // hg.extend_with_edges(vec![
    //     Hx::new_unchecked([0, 1], ()),
    //     Hx::new_unchecked([0, 2], ()),
    //     Hx::new_unchecked([0, 3], ()),
    //     Hx::new_unchecked([1, 2], ()),
    // ]);
    //
    // // C4
    // hg.extend_with_edges(vec![
    //     Hx::new_unchecked([0, 1], ()),
    //     Hx::new_unchecked([0, 2], ()),
    //     Hx::new_unchecked([1, 3], ()),
    //     Hx::new_unchecked([2, 3], ()),
    // ]);
    //
    // // Star 4
    // hg.extend_with_edges(vec![
    //     Hx::new_unchecked([0, 1], ()),
    //     Hx::new_unchecked([0, 2], ()),
    //     Hx::new_unchecked([0, 3], ()),
    // ]);
    //
    // // Path 4
    // hg.extend_with_edges(vec![
    //     Hx::new_unchecked([0, 1], ()),
    //     Hx::new_unchecked([0, 2], ()),
    //     Hx::new_unchecked([1, 3], ()),
    //     Hx::new_unchecked([2, 3], ()),
    // ]);

    // hg.extend_with_edges(vec![Hx::new_unchecked([0, 1, 2], ())]);
    // hg.extend_with_edges(vec![Hx::new_unchecked([0, 1, 3], ())]);

    HyperAdjList::from_hypergraph_unmapped(hg.clone())
});

pub fn simple_graph_count() {
    let adj = SIMPLE_UNWEIGHTED_HYPERGRAPH.clone();
    let rv = unweighted_4(&adj);
    for (motif, stats) in rv.iter() {
        println!("{}\t{}", stats.count, motif.get_canonical_rep());
    }
}

pub fn simple_graph_intensity() {
    let adj = SIMPLE_WEIGHTED.clone();
    let rv = weighted_4(&adj);
    for (motif, stats) in rv.iter() {
        println!(
            "{}\t{}\t{}",
            stats.count,
            stats.mean_intensity,
            motif.get_canonical_rep()
        );
    }
}

pub fn simple_c4_w() {
    let adj = SIMPLE_WEIGHTED.clone();
    let edges = adj
        .iter_by_size(2)
        .map(|(_, e)| (e.nodes[0], e.nodes[1], *e.weight))
        .collect::<Vec<_>>();
    let (mut adj, _, _) =
        AdjList::<NodeWeight, Undirected, WithoutIncidence>::from_edges_mapped(edges);
    println!("{:?}", intensity_c4_subinc(&mut adj));
}

pub fn dblp_uw() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .dblp()
        .unweighted()
        .load()?;

    seq!(N in 4..11 {
        hg.take_edges::<N>();
    });
    hg.remove_isolated_nodes();

    let (hyperadj, _, _) = HyperAdjList::<()>::from_hypergraph_mapped(hg.0);

    let t = std::time::Instant::now();
    println!("n: {}, m: {}", hyperadj.n(), hyperadj.m());
    escape::unweighted_4(&hyperadj);
    println!("Finished in: {:?}", t.elapsed());

    Ok(())
}

pub fn dblp_w() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .dblp()
        .weighted()
        .load()?;

    seq!(N in 4..11 {
        hg.take_edges::<N>();
    });
    hg.remove_isolated_nodes();

    let (hyperadj, _, _) = HyperAdjList::<NodeWeight>::from_hypergraph_mapped(hg.0);

    let t = std::time::Instant::now();
    println!("n: {}, m: {}", hyperadj.n(), hyperadj.m());
    escape::weighted_4(&hyperadj);
    println!("Finished in: {:?}", t.elapsed());

    Ok(())
}

pub fn hospital_uw3() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .hospital()
        .unweighted()
        .load()?;

    seq!(N in 4..11 {
        hg.take_edges::<N>();
    });
    hg.remove_isolated_nodes();

    let (hyperadj, _, _) = HyperAdjList::<()>::from_hypergraph_mapped(hg.0);

    let t = std::time::Instant::now();
    println!("n: {}, m: {}", hyperadj.n(), hyperadj.m());
    let rv = escape::unweighted_3(&hyperadj);
    println!("Finished in: {:?}", t.elapsed());

    for (motif, stats) in rv.iter() {
        println!(
            "{}\t{}\t{}",
            stats.count,
            stats.mean_intensity,
            motif.get_canonical_rep()
        );
    }

    Ok(())
}

pub fn hospital_w3() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .hospital()
        .weighted()
        .load()?;

    seq!(N in 4..11 {
        hg.take_edges::<N>();
    });
    hg.remove_isolated_nodes();

    let (hyperadj, _, _) = HyperAdjList::<NodeWeight>::from_hypergraph_mapped(hg.0);

    let t = std::time::Instant::now();
    println!("n: {}, m: {}", hyperadj.n(), hyperadj.m());
    let rv = escape::weighted_3(&hyperadj);
    println!("Finished in: {:?}", t.elapsed());

    for (motif, stats) in rv.iter() {
        println!(
            "{}\t{}\t{}",
            stats.count,
            stats.mean_intensity,
            motif.get_canonical_rep()
        );
    }

    Ok(())
}

pub fn hospital_uw4() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .hospital()
        .unweighted()
        .load()?;

    seq!(N in 5..11 {
        hg.take_edges::<N>();
    });
    hg.remove_isolated_nodes();

    let (hyperadj, _, _) = HyperAdjList::<()>::from_hypergraph_mapped(hg.0);

    let t = std::time::Instant::now();
    println!("n: {}, m: {}", hyperadj.n(), hyperadj.m());
    let rv = escape::unweighted_4(&hyperadj);
    println!("Finished in: {:?}", t.elapsed());

    for (motif, stats) in rv.iter() {
        println!(
            "{}\t{}\t{}",
            stats.count,
            stats.mean_intensity,
            motif.get_canonical_rep()
        );
    }

    Ok(())
}

pub fn hospital_w4() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .hospital()
        .weighted()
        .load()?;

    let mut sum = 0.0;
    for edge in hg.0.edges::<2>() {
        sum += edge.weight;
    }
    println!("Medium 2-deg = {}", sum / hg.0.edges::<2>().len() as f32);

    seq!(N in 4..11 {
        hg.take_edges::<N>();
    });
    hg.remove_isolated_nodes();

    let (hyperadj, _, _) = HyperAdjList::<NodeWeight>::from_hypergraph_mapped(hg.0);

    let t = std::time::Instant::now();
    println!("n: {}, m: {}", hyperadj.n(), hyperadj.m());
    let rv = escape::weighted_4(&hyperadj);
    println!("Finished in: {:?}", t.elapsed());

    for (motif, stats) in rv.iter() {
        println!(
            "{}\t{}\t{}",
            stats.count,
            stats.mean_intensity,
            motif.get_canonical_rep()
        );
    }

    Ok(())
}

pub fn friendship_hs() -> Result<(), Box<dyn Error>> {
    let mut hg = DatasetLoader::builder()
        .cached(true)
        .friendship_hs()
        .unweighted()
        .load()?;

    // seq!(N in 4..11 {
    //     hg.take_edges::<N>();
    // });
    // hg.remove_isolated_nodes();

    println!("hg.m: {}", hg.m());
    let mut count = 0;
    seq!(N in 2..11 {
        count += hg.0.edges::<N>().len();
    });
    println!("m before {}", count);

    hg.normalize_node_ids();

    let mut count = 0;
    seq!(N in 2..11 {
        count += hg.0.edges::<N>().len();
    });
    println!("m before {}", count);

    // let (hyperadj, _, _) = HyperAdjList::<()>::from_hypergraph_mapped(hg.0);
    //
    // let t = std::time::Instant::now();
    // println!("n: {}, m: {}", hyperadj.n(), hyperadj.m());
    // let rv = escape::unweighted_4(&hyperadj);
    // println!("Finished in: {:?}", t.elapsed());
    //
    // for (_number, (motif, stats)) in rv.iter().enumerate() {
    //     println!("{}\t{}", stats.count, motif.get_canonical_rep());
    // }

    Ok(())
}

pub fn all_simple_graphlets() {
    let graphlets = [
        ("Path 4", PATH4_W.clone()),
        ("Star 4", STAR4_W.clone()),
        ("Paw", PAW_W.clone()),
        ("C4", C4_W.clone()),
        ("Diamond", DIAMOND_W.clone()),
        ("K4", K4_W.clone()),
    ];

    for (name, (_hg, adj)) in graphlets.iter() {
        println!("{}", name);
        let rv = weighted_4(adj);
        for (motif, stats) in rv.iter() {
            println!(
                "{}\t{}\t{}",
                stats.count,
                stats.mean_intensity,
                motif.get_canonical_rep()
            );
        }
    }
}
