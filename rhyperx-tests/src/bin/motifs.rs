use rust_core::loader::DatasetLoader;
use rust_core::motifs::algorithms::orca;

pub fn main() {
    // Unweighted
    let hg = DatasetLoader::builder()
        .conference()
        .unweighted()
        .cached(true)
        .load()
        .unwrap();

    let rv = orca::unweighted_3(&hg);
    for (fingerprint, stats) in rv.iter() {
        println!("{}", fingerprint.get_canonical_rep());
        println!("{}", stats);
    }

    // Weighted
    let hg = DatasetLoader::builder()
        .conference()
        .weighted()
        .cached(true)
        .load()
        .unwrap();

    let rv = orca::weighted_3(&hg);
    for (fingerprint, stats) in rv.iter() {
        println!("{}", fingerprint.get_canonical_rep());
        println!("{}", stats);
    }
}
