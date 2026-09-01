use std::fs::read_to_string;

use hashbrown::HashMap;

use rhyperx_core::hypergraph::{HxUnsizedRef, Hypergraph};

use crate::loader::common::Loader;
use crate::loader::error::LoaderError;

use super::{WikiStdUnweightedLoader, WikiStdWeightedLoader};

impl Loader for WikiStdUnweightedLoader {
    type Output = Hypergraph<u32, ()>;

    const VARIANT: &'static str = "uw";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let contents = read_to_string(dataset_location)?;
        let mut votes: HashMap<String, Vec<String>> = HashMap::new();
        let mut hg = Hypergraph::new();

        for line in contents.lines() {
            let l = line.trim();
            if l.is_empty() {
                // flush votes
                for (_k, v) in votes.drain() {
                    let mut uids: Vec<u32> = v
                        .into_iter()
                        .filter_map(|s| s.parse::<u32>().ok())
                        .collect();
                    if uids.len() > 1 && uids.len() <= 10 {
                        hg.add_edge_slice(&mut uids, ())
                            .expect("wiki: found malformed hyperedge");
                    }
                }
                continue;
            }
            if !l.starts_with('V') {
                continue;
            }
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }
            let vote = parts[1].to_string();
            let u_id = parts[2].to_string();
            votes.entry(vote).or_default().push(u_id);
        }

        Ok(hg)
    }
}

impl Loader for WikiStdWeightedLoader {
    type Output = Hypergraph<u32, f32>;

    const VARIANT: &'static str = "w";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let contents = read_to_string(dataset_location)?;
        let mut votes: HashMap<String, Vec<String>> = HashMap::new();
        let mut hg = Hypergraph::new();

        for line in contents.lines() {
            let l = line.trim();
            if l.is_empty() {
                for (_k, v) in votes.drain() {
                    let mut uids: Vec<u32> = v
                        .into_iter()
                        .filter_map(|s| s.parse::<u32>().ok())
                        .collect();
                    if uids.len() > 1 && uids.len() <= 10 {
                        uids.sort_unstable();
                        match hg.get_hyperedge_mut(HxUnsizedRef::new(&uids, &0.0)) {
                            Some(r) => *r.weight += 1.0,
                            None => {
                                hg.add_edge_slice(&mut uids, 1.0)
                                    .expect("wiki: found malformed hyperedge");
                            }
                        }
                    }
                }
                continue;
            }
            if !l.starts_with('V') {
                continue;
            }
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }
            let vote = parts[1].to_string();
            let u_id = parts[2].to_string();
            votes.entry(vote).or_default().push(u_id);
        }

        Ok(hg)
    }
}
