use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use hashbrown::HashMap;

use rhyperx_core::hypergraph::{HxUnsizedRef, Hypergraph};

use crate::loader::common::Loader;
use crate::loader::error::LoaderError;

use super::GeneDiseaseStdWeightedLoader;

impl Loader for GeneDiseaseStdWeightedLoader {
    type Output = Hypergraph<u32, f32>;

    const VARIANT: &'static str = "w";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        // Parse TSV and aggregate diseases -> list of genes
        let file = File::open(dataset_location)?;
        let mut reader = BufReader::new(file);

        let mut diseases: HashMap<String, Vec<u32>> = HashMap::new();

        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let s = line.trim();
            if s.is_empty() {
                line.clear();
                continue;
            }
            let parts: Vec<&str> = s.split('\t').collect();
            if parts.len() > 4
                && let Ok(gene) = parts[0].parse::<u32>()
            {
                let dis = parts[4].to_string();
                diseases.entry(dis).or_default().push(gene);
            }
            line.clear();
        }

        let mut hg = Hypergraph::new();

        for (_d, mut genes) in diseases.into_iter() {
            if genes.len() > 1 && genes.len() <= 10 {
                genes.sort_unstable();
                match hg.get_hyperedge_mut(HxUnsizedRef::new(&genes, &0.0)) {
                    Some(r) => *r.weight += 1.0,
                    None => {
                        hg.add_edge_slice_unchecked(&genes, 1.0);
                    }
                }
            }
        }

        Ok(hg)
    }
}
