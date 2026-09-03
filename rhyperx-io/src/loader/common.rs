#![allow(unused)]
use super::error::*;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::DatasetLoaderDispatcherAttr;
use rhyperx_algo::misc::clique::find_cliques;
use rhyperx_core::graph::{AdjList, Undirected};
use rhyperx_core::types::NodeId;
use serde::Deserialize;
use std::collections::HashMap;

use rhyperx_core::serialize::traits::{DumpCacheToFile, LoadFromCacheDeserialized};

/// Read a whitespace-separated file of integers (one per line).
pub fn read_ints_from_file<P: AsRef<Path>>(path: &P) -> Result<Vec<u32>, LoaderError> {
    let s = fs::read_to_string(path)?;
    let mut v = Vec::new();
    for line in s.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Ok(x) = t.parse::<u32>() {
            v.push(x);
        }
    }
    Ok(v)
}

/// Build the `{name}-nverts.txt` / `{name}-simplices.txt` paths for a dataset stored either as a
/// directory or as a file.
fn nverts_simplices_paths(base: &Path) -> (PathBuf, PathBuf) {
    let nverts_path = if base.is_dir() {
        base.join(format!(
            "{}-nverts.txt",
            base.file_name().and_then(|n| n.to_str()).unwrap_or("")
        ))
    } else {
        base.with_extension("-nverts.txt")
    };
    let simplices_path = if base.is_dir() {
        base.join(format!(
            "{}-simplices.txt",
            base.file_name().and_then(|n| n.to_str()).unwrap_or("")
        ))
    } else {
        base.with_extension("-simplices.txt")
    };
    (nverts_path, simplices_path)
}

/// Read the `nverts`/`simplices` pair describing variable-size hyperedges.
pub fn read_nverts_simplices(base: &Path) -> Result<(Vec<u32>, Vec<u32>), LoaderError> {
    let (nverts_path, simplices_path) = nverts_simplices_paths(base);
    let v = read_ints_from_file(&nverts_path)?;
    let s = read_ints_from_file(&simplices_path)?;
    Ok((v, s))
}

/// Slice `simplices` into groups of sizes given by `nverts`.
pub fn build_edges_from_nverts_simplices(nverts: Vec<u32>, simplices: &[u32]) -> Vec<Vec<u32>> {
    let mut edges = Vec::new();
    let mut s_idx = 0usize;

    for i in nverts.into_iter() {
        let mut e: Vec<u32> = Vec::new();
        for _ in 0..i {
            if s_idx >= simplices.len() {
                break;
            }
            e.push(simplices[s_idx]);
            s_idx += 1;
        }
        if e.len() > 1 && e.len() <= 10 {
            edges.push(e);
        }
    }

    edges
}

/// Build the maximal cliques of the graph induced by `edges`, mapped back to the original node
/// IDs, keeping only cliques of size in `[2, 10]`.
pub fn build_clique_hyperedges(edges: Vec<(u32, u32)>, remove_self_loops: bool) -> Vec<Vec<u32>> {
    let (mut adj_list, original_index, _compressed_index) =
        AdjList::<u32, (), Undirected>::from_edges_mapped(
            edges.into_iter().map(|(u, v)| (u, v, ())).collect(),
        );
    if remove_self_loops {
        adj_list.remove_self_loops();
    }
    adj_list.remove_multiedges();

    find_cliques(&adj_list)
        .into_iter()
        .filter(|c| c.len() >= 2 && c.len() <= 10)
        .map(|clique| {
            clique
                .into_iter()
                .map(|node| original_index[node.as_usize()])
                .collect()
        })
        .collect()
}

/// Struct to hold the dataset information specified in dataset.toml
#[derive(Deserialize, Debug)]
pub struct DatasetConfig {
    pub cache_dir: Option<PathBuf>,
    #[serde(flatten)]
    pub datasets: HashMap<String, DatasetDescriptor>,
}

#[derive(Deserialize, Debug)]
pub struct DatasetDescriptor {
    pub path: PathBuf,
    pub alias: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub description: Option<String>,
}

pub fn parse_datasets_descriptor() -> Result<DatasetConfig, Box<dyn std::error::Error>> {
    let path_str = std::env::var("DATASETS_TOML")?;
    let toml_str = fs::read_to_string(path_str)?;
    let config: DatasetConfig = toml::from_str(&toml_str)?;
    Ok(config)
}

fn hash_file_metadata<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let metadata = fs::metadata(path)?;
    let size = metadata.len();
    let mtime = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut hasher = DefaultHasher::new();

    size.hash(&mut hasher);
    mtime.hash(&mut hasher);

    Ok(hasher.finish())
}

pub fn get_cache_file<P1, P2>(
    dataset_location: &P1,
    cache_dir: &P2,
    name: &str,
    extension: &str,
) -> io::Result<PathBuf>
where
    P1: AsRef<Path> + ?Sized,
    P2: AsRef<Path> + ?Sized,
{
    let hash = hash_file_metadata(dataset_location)?;
    Ok(PathBuf::from(cache_dir.as_ref())
        .join(format!("{}_{:016x}", name, hash))
        .with_extension(extension))
}

pub trait DatasetInfo {
    /// The name of the dataset, used for logging and cache file naming.
    const NAME: &'static str;
    /// The folder or file path where the raw dataset is located.
    /// loader will always read from the raw dataset file.
    fn dataset_location(&self) -> PathBuf;
    /// The directory where the cache file should be stored. If `None`, caching is disabled and the
    /// loader will always read from the raw dataset file.
    ///This parameter is set from the dataset.toml file
    fn cache_dir(&self) -> Option<PathBuf>;
    /// Returns a string that should uniquely identify the dataset. Implementation is generated
    /// through the `#[loader]` macro, and is used to determine the cache file name.
    fn cache_hash(&self, length: usize) -> String;
}

pub fn hash_to_len<T: Hash>(x: T, length: usize) -> String {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    let hash_val = hasher.finish();

    let hash_string = format!("{:016x}", hash_val);

    if length >= hash_string.len() {
        format!("{:0>width$}", hash_string, width = length)
    } else {
        hash_string[..length].to_string()
    }
}

pub trait Loader
where
    Self: DatasetLoaderDispatcherAttr + Hash,
    Self::Output: DumpCacheToFile + LoadFromCacheDeserialized,
{
    /// A description of the method used to load the dataset, for caching purposes
    const VARIANT: &'static str;

    type Output;

    // #[allow(clippy::wrong_self_convention)]
    fn load(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.get_dataset_location();
        let cache_dir = self.get_cache_dir();

        if !dataset_location.exists() {
            log::error!("Invalid dataset location '{}'", dataset_location.display());
            return Err(LoaderError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Invalid dataset location",
            )));
        }
        if cache_dir.is_none() {
            return self.from_file();
        }

        if !*self.get_cached() {
            return self.from_file();
        }

        let cache_dir = cache_dir.as_ref().unwrap();

        let cache_file = PathBuf::from(&cache_dir)
            .join(format!(
                "{}_{}_{}",
                self.get_name(),
                Self::VARIANT,
                hash_to_len(self, 16)
            ))
            .with_extension("bin");

        if cache_file.exists() {
            match <Self::Output as LoadFromCacheDeserialized>::load_deserialized(&cache_file) {
                Ok(hg) => Ok(hg),
                Err(_err) => {
                    log::warn!(
                        "Cache file {} is corrupted. Falling back to uncached loading.",
                        cache_file.display()
                    );
                    let rv = self.from_file()?;
                    if let Err(e) = rv.save_to_file(&cache_file) {
                        log::error!(
                            "Failed to save hypergraph to cache file {}: {}",
                            cache_file.display(),
                            e
                        );
                    }
                    Ok(rv)
                }
            }
        } else {
            log::info!(
                "Loading hypergraph from source and caching to {}...",
                cache_file.display()
            );
            let rv = self.from_file()?;

            if let Err(e) = rv.save_to_file(&cache_file) {
                log::error!(
                    "Failed to save hypergraph to cache file {}: {}",
                    cache_file.display(),
                    e
                );
            }
            Ok(rv)
        }
    }

    // #[allow(clippy::wrong_self_convention)]
    fn from_file(&self) -> Result<Self::Output, LoaderError>;
}

#[inline(always)]
pub fn parse_u32(chars: &[u8]) -> u32 {
    let mut rv = 0;
    let mut base = 1;
    for c in chars.iter().rev() {
        rv += (c - b'0') as u32 * base;
        base *= 10;
    }
    rv
}
