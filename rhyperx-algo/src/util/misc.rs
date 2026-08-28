use std::{fs, path::Path};

use hashbrown::HashMap;

use crate::motifs::{compressed_motif::CompactMotif, fingerprint::Fingerprint5};

struct FlatBinData<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> FlatBinData<N> {
    pub fn new() -> Self {
        Self { data: [0; N] }
    }

    pub fn from_array(arr: [u8; N]) -> Self {
        Self { data: arr }
    }

    pub fn set_range(&mut self, start: usize, value: u8) {
        let bit_offset = start % 8;
        self.data[start / 8] &= (1 << bit_offset) - 1;
        self.data[start / 8] |= value << bit_offset;

        self.data[start / 8 + 1] &= 0xFF << bit_offset;
        self.data[start / 8 + 1] |= value >> (8 - bit_offset);
    }
}

const CACHE_DIR: &str = "cache";

pub fn save_to_file(
    v: HashMap<Fingerprint5, Vec<CompactMotif<5>>>,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Ensure the directory exists
    let path = Path::new(CACHE_DIR);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    let v: Vec<(usize, Vec<_>)> = v
        .into_iter()
        .enumerate()
        .map(|(i, (_key, items))| {
            // Map the inner items to just the 'container' field
            let containers = items.into_iter().map(|m| m.container).collect();
            (i, containers)
        })
        .collect();

    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&v)?;

    let file_path = path.join(file_name);
    fs::write(file_path, bytes)?;

    Ok(())
}

pub fn load_from_file(
    file_name: &str,
) -> Result<HashMap<Fingerprint5, Vec<CompactMotif<5>>>, Box<dyn std::error::Error>> {
    let file_path = Path::new(CACHE_DIR).join(file_name);

    let bytes = fs::read(file_path)?;
    let deserialized: Vec<(usize, Vec<u32>)> =
        rkyv::from_bytes::<Vec<(usize, Vec<u32>)>, rkyv::rancor::Error>(&bytes)?;

    // Reconstruct the HashMap
    let rv: HashMap<Fingerprint5, Vec<CompactMotif<5>>> = deserialized
        .into_iter()
        .map(|(key, containers)| {
            let motifs = containers
                .into_iter()
                .map(|container| CompactMotif::<5> { container })
                .collect::<Vec<CompactMotif<5>>>();
            (motifs[0].fingerprint(), motifs)
        })
        .collect();

    Ok(rv)
}
