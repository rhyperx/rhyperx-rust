use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use hashbrown::HashMap;

use rhyperx_core::hypergraph::{HxUnsizedRef, Hypergraph};

use crate::loader::common::Loader;
use crate::loader::error::LoaderError;

use super::{JusticeStdUnweightedLoader, JusticeStdWeightedLoader};

/// Parse the SCOTUS CSV (columns `caseId`, `justiceName`, `vote`) into hyperedges: each
/// `(caseId, vote)` group yields the set of voting justices, mapped to first-seen ids.
///
/// Empty or non-integer `vote` values are dropped.
fn parse_justice_groups<P: AsRef<Path>>(path: &P) -> Result<Vec<Vec<u32>>, LoaderError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut header: Option<Vec<String>> = None;
    let mut justice_id: HashMap<String, u32> = HashMap::new();
    let mut groups: HashMap<(String, i32), Vec<u32>> = HashMap::new();

    for line in reader.lines() {
        let l = line?;
        let cols: Vec<&str> = l.split(',').collect();

        if header.is_none() {
            header = Some(cols.iter().map(|s| s.trim().to_string()).collect());
            continue;
        }

        let header = header.as_ref().unwrap();
        let case_idx = header.iter().position(|h| h == "caseId");
        let name_idx = header.iter().position(|h| h == "justiceName");
        let vote_idx = header.iter().position(|h| h == "vote");

        let (Some(case_idx), Some(name_idx), Some(vote_idx)) = (case_idx, name_idx, vote_idx)
        else {
            return Err(LoaderError::MlformedDataset(
                "justice dataset missing expected columns".into(),
            ));
        };

        if case_idx >= cols.len() || name_idx >= cols.len() || vote_idx >= cols.len() {
            continue;
        }

        let case_id = cols[case_idx].trim();
        let justice_name = cols[name_idx].trim();
        let vote = cols[vote_idx].trim();

        if case_id.is_empty() || justice_name.is_empty() || vote.is_empty() {
            continue;
        }
        let Ok(vote) = vote.parse::<i32>() else {
            continue;
        };

        let id = match justice_id.get(justice_name) {
            Some(&id) => id,
            None => {
                let id = justice_id.len() as u32;
                justice_id.insert(justice_name.to_string(), id);
                id
            }
        };
        groups
            .entry((case_id.to_string(), vote))
            .or_insert_with(Vec::new)
            .push(id);
    }

    let mut edges = Vec::new();
    for (_k, mut ids) in groups.into_iter() {
        ids.sort_unstable();
        ids.dedup();
        if ids.len() > 1 && ids.len() <= 10 {
            edges.push(ids);
        }
    }
    Ok(edges)
}

impl Loader for JusticeStdUnweightedLoader {
    type Output = Hypergraph<u32, ()>;

    const VARIANT: &'static str = "uw";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let edges = parse_justice_groups(&dataset_location)?;

        let mut hg = Hypergraph::new();
        for mut e in edges {
            hg.add_edge_slice(&mut e, ()).expect("Malformed edge");
        }

        Ok(hg)
    }
}

impl Loader for JusticeStdWeightedLoader {
    type Output = Hypergraph<u32, f32>;

    const VARIANT: &'static str = "w";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let edges = parse_justice_groups(&dataset_location)?;

        let mut hg = Hypergraph::new();
        for mut e in edges {
            e.sort_unstable();
            match hg.get_hyperedge_mut(HxUnsizedRef::new(&e, &0.0)) {
                Some(r) => *r.weight += 1.0,
                None => {
                    hg.add_edge_slice(&mut e, 1.0).expect("Malformed edge");
                }
            }
        }

        Ok(hg)
    }
}
