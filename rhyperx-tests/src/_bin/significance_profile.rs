use rust_core::{
    loader::DatasetLoader,
    motifs::{
        algorithms::{escape::weighted_3, null_model::compute_sp3},
        types::SPStrategy,
    },
    types::{NodeWeight, hyperadj_list::HyperAdjList},
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let hg = DatasetLoader::builder()
        .cached(true)
        .hospital()
        .weighted()
        .load()?
        .0;

    let (adj, _, _) = HyperAdjList::<NodeWeight>::from_hypergraph_mapped(hg);

    compute_sp3(&adj, 100, SPStrategy::ShuffleBoth(10 * adj.m()), |adj| {
        weighted_3(adj)
    });

    Ok(())
}
