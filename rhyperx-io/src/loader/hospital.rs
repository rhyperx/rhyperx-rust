use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use hashbrown::HashMap;

use rhyperx_core::hypergraph::{HxUnsizedRef, Hypergraph};

use crate::loader::common::{Loader, build_clique_hyperedges};
use crate::loader::error::LoaderError;

use super::{HospitalStdUnweightedLoader, HospitalStdWeightedLoader};

const TIME_OFFSET: i32 = 140;

impl Loader for HospitalStdUnweightedLoader {
    type Output = Hypergraph<u32, ()>;

    const VARIANT: &'static str = "uw";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let file = File::open(dataset_location)?;
        let reader = BufReader::new(file);

        let mut time_edges: HashMap<usize, Vec<(u32, u32)>> = HashMap::new();

        for line in reader.lines() {
            let l = line?;
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() >= 3 {
                let t_raw: i32 = parts[0].parse().unwrap_or(0);
                let a = parts[1].parse().unwrap_or(0);
                let b = parts[2].parse().unwrap_or(0);
                let t = t_raw - TIME_OFFSET;
                time_edges.entry(t as usize).or_default().push((a, b));
            }
        }

        let mut hg = Hypergraph::new();

        for (_t, edge_list) in time_edges.into_iter() {
            for mut clique in build_clique_hyperedges(edge_list, true) {
                hg.add_edge_slice(&mut clique, ())
                    .expect("Clique found with duplicate node");
            }
        }

        Ok(hg)
    }
}

impl Loader for HospitalStdWeightedLoader {
    type Output = Hypergraph<u32, f32>;

    const VARIANT: &'static str = "w";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let file = File::open(dataset_location)?;
        let reader = BufReader::new(file);

        let mut time_edges: HashMap<usize, Vec<(u32, u32)>> = HashMap::new();

        for line in reader.lines() {
            let l = line?;
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() >= 3 {
                let t_raw: i32 = parts[0].parse().unwrap_or(0);
                let a = parts[1].parse().unwrap_or(0);
                let b = parts[2].parse().unwrap_or(0);
                let t = t_raw - TIME_OFFSET;
                time_edges.entry(t as usize).or_default().push((a, b));
            }
        }

        let mut hg = Hypergraph::new();

        for (_t, edge_list) in time_edges.into_iter() {
            for mut clique in build_clique_hyperedges(edge_list, true) {
                clique.sort_unstable();
                match hg.get_hyperedge_mut(HxUnsizedRef::new(&clique, &0.0)) {
                    Some(r) => *r.weight += 1.0,
                    None => {
                        hg.add_edge_slice(&mut clique, 1.0)
                            .expect("Clique found with duplicate node");
                    }
                }
            }
        }

        Ok(hg)
    }
}
