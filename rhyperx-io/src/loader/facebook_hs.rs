use std::fs::File;
use std::io::{BufRead, BufReader};

use rhyperx_core::hypergraph::{Hypergraph, SizedHx};

use crate::loader::common::Loader;
use crate::loader::error::LoaderError;

use super::FacebookHsStdUnweightedLoader;

impl Loader for FacebookHsStdUnweightedLoader {
    type Output = Hypergraph<u32, ()>;

    const VARIANT: &'static str = "uw";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();

        let file = File::open(dataset_location)?;
        let reader = BufReader::new(file);

        let mut hg = Hypergraph::new();

        for line in reader.lines() {
            let l = line?;
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() >= 2 {
                let a = parts[0].parse().unwrap_or(0);
                let b = parts[1].parse().unwrap_or(0);
                // Only include confirmed if a third column exists and equals 1, otherwise include by default
                let include = if parts.len() >= 3 {
                    parts[2] == "1" || parts[2] == "1.0"
                } else {
                    true
                };
                if include {
                    let nodes = [a.min(b), a.max(b)];
                    hg.add_edge_sized(SizedHx::<2, u32, ()>::new_unchecked(nodes, ()));
                }
            }
        }

        Ok(hg)
    }
}
