use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use hashbrown::HashMap;

use rhyperx_core::hypergraph::{HxUnsizedRef, Hypergraph};

use crate::loader::common::Loader;
use crate::loader::error::LoaderError;

use super::{PacsStdUnweightedLoader, PacsStdWeightedLoader};

/// Parse the PACS CSV (columns `ArticleID`, `PACS`, `AuthorDAIS`, `FullName`) into hyperedges:
/// each `ArticleID` yields the set of distinct authors (mapped to first-seen ids).
///
/// Rows with an empty `FullName` or `ArticleID` are dropped.
pub fn parse_pacs_groups<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Vec<Vec<u32>>, LoaderError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut header: Option<Vec<String>> = None;
    let mut author_id: HashMap<String, u32> = HashMap::new();
    let mut groups: HashMap<String, Vec<u32>> = HashMap::new();

    for line in reader.lines() {
        let l = line?;
        let cols: Vec<&str> = l.split(',').collect();

        if header.is_none() {
            header = Some(cols.iter().map(|s| s.trim().to_string()).collect());
            continue;
        }

        let header = header.as_ref().unwrap();
        let article_idx = header.iter().position(|h| h == "ArticleID");
        let fullname_idx = header.iter().position(|h| h == "FullName");

        let (Some(article_idx), Some(fullname_idx)) = (article_idx, fullname_idx) else {
            return Err(LoaderError::MlformedDataset(
                "pacs dataset missing expected columns".into(),
            ));
        };

        if article_idx >= cols.len() || fullname_idx >= cols.len() {
            continue;
        }

        let article = cols[article_idx].trim();
        let fullname = cols[fullname_idx].trim();

        if article.is_empty() || fullname.is_empty() {
            continue;
        }

        let id = match author_id.get(fullname) {
            Some(&id) => id,
            None => {
                let id = author_id.len() as u32;
                author_id.insert(fullname.to_string(), id);
                id
            }
        };
        groups.entry(article.to_string()).or_default().push(id);
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

impl Loader for PacsStdUnweightedLoader {
    type Output = Hypergraph<u32, ()>;

    const VARIANT: &'static str = "uw";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let edges = parse_pacs_groups(&dataset_location)?;

        let mut hg = Hypergraph::new();
        for mut e in edges {
            hg.add_edge_slice(&mut e, ()).expect("Malformed edge");
        }

        Ok(hg)
    }
}

impl Loader for PacsStdWeightedLoader {
    type Output = Hypergraph<u32, f32>;

    const VARIANT: &'static str = "w";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let edges = parse_pacs_groups(&dataset_location)?;

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
