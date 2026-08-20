use rust_core::loader::DatasetLoader;
use std::time::Instant;

macro_rules! load_dataset {
    ($dataset:expr) => {{
        let time = Instant::now();
        println!("Loading dataset {}", $dataset.dataset_location.display());

        match $dataset.load() {
            Ok(hg) => {
                println!("edges.len m {}", hg.m());
                println!("order 2 {}", hg.0.edges::<2>().len());
            }
            Err(e) => assert!(false, "Failed to load dataset: {}", e),
        }
        println!("Total time: {:?}", time.elapsed());
    }};
}

#[rustfmt::skip]
pub fn main() {

    load_dataset!(DatasetLoader::builder().hospital().unweighted().cached(true));
    load_dataset!(DatasetLoader::builder().conference().unweighted().cached(true));
}
