use std::fs::File;
use std::io::{BufRead, BufReader};

use hashbrown::HashMap;

use rhyperx_core::hypergraph::{HxUnsizedRef, Hypergraph};

use crate::loader::common::Loader;
use crate::loader::error::LoaderError;

use super::{DblpStdUnweightedLoader, DblpStdWeightedLoader};

impl Loader for DblpStdUnweightedLoader {
    type Output = Hypergraph<u32, ()>;

    const VARIANT: &'static str = "uw";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let file = File::open(dataset_location)?;
        let reader = BufReader::new(file);

        let mut graph: HashMap<String, Vec<u32>> = HashMap::new();
        for line in reader.lines().skip(1) {
            let l = line?;
            if l.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = l.split(',').collect();
            if parts.len() >= 2 {
                let paper = parts[0].to_string();
                if let Ok(author) = parts[1].trim().parse::<u32>() {
                    graph.entry(paper).or_default().push(author);
                }
            }
        }

        let mut hg = Hypergraph::new();

        for (_paper, mut authors) in graph.into_iter() {
            authors.sort_unstable();
            authors.dedup();
            if authors.len() > 1 && authors.len() <= 10 {
                hg.add_edge_slice_unchecked(&authors, ());
            }
        }

        Ok(hg)
    }
}

impl Loader for DblpStdWeightedLoader {
    type Output = Hypergraph<u32, f32>;

    const VARIANT: &'static str = "w";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let file = File::open(dataset_location)?;
        let reader = BufReader::new(file);

        let mut graph: HashMap<String, Vec<u32>> = HashMap::new();

        for line in reader.lines() {
            let l = line?;
            if l.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = l.split(',').collect();
            if parts.len() >= 2 {
                let paper = parts[0].to_string();
                if let Ok(author) = parts[1].trim().parse::<u32>() {
                    graph.entry(paper).or_default().push(author);
                }
            }
        }

        let mut hg = Hypergraph::new();

        for (_paper, mut authors) in graph.into_iter() {
            authors.sort_unstable();
            authors.dedup();

            if authors.len() > 1 && authors.len() <= 10 {
                match hg.get_hyperedge_mut(HxUnsizedRef::new(&authors, &0.0)) {
                    Some(r) => *r.weight += 1.0,
                    None => {
                        hg.add_edge_slice_unchecked(&authors, 1.0);
                    }
                }
            }
        }

        Ok(hg)
    }
}
