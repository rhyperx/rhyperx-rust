use pyo3::{pyclass, pymethods};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rkyv::{Archive, Deserialize, Serialize};
use std::{fs::File, io::Write, ops::Index, path::Path, time::Instant};

use crate::graph::{AdjList, UnweightedAdjList, types::NodeId};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[gen_stub_pyclass(module = "rust_core._core.graph")]
#[pyclass(skip_from_py_object)]
pub struct FlatAdjList {
    offsets: Vec<usize>,
    edges: Vec<NodeId>,
}

impl Index<usize> for FlatAdjList {
    type Output = [NodeId];

    fn index(&self, index: usize) -> &Self::Output {
        let start = self.offsets[index];
        let end = self.offsets[index + 1];
        &self.edges[start..end]
    }
}

impl Default for FlatAdjList {
    fn default() -> Self {
        Self::new()
    }
}

#[gen_stub_pymethods(module = "rust_core._core.graph")]
#[pymethods]
impl FlatAdjList {
    #[new]
    pub fn new() -> Self {
        Self {
            offsets: vec![0],
            edges: vec![],
        }
    }

    #[staticmethod]
    pub fn from_edges(edges: Vec<(NodeId, NodeId)>, _directed: bool) -> Self {
        let (mut adj_list, _, _) = UnweightedAdjList::from_edges_mapped(edges);
        adj_list.remove_self_loops();
        adj_list.make_undirected();

        let mut rv = Self {
            offsets: vec![0; adj_list.n() + 1],
            edges: vec![0; adj_list.m()],
        };

        for (i, neighbors) in adj_list.iter_neighbors().enumerate() {
            rv.offsets[i + 1] = rv.offsets[i] + neighbors.len();
            let neighbors = neighbors.iter().map(|&(v, _)| v).collect::<Vec<_>>();
            rv.edges[rv.offsets[i]..rv.offsets[i + 1]].copy_from_slice(&neighbors);
        }

        rv
    }

    pub fn n(&self) -> usize {
        self.offsets.len() - 1
    }

    pub fn m(&self) -> usize {
        self.edges.len()
    }
}

impl FlatAdjList {
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self)?;
        // .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut file = File::create(path)?;
        file.write_all(&bytes)?;

        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<FlatAdjList, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;

        let start = Instant::now();
        let archived = rkyv::access::<ArchivedFlatAdjList, rkyv::rancor::Error>(&bytes[..])?;
        // let archived = unsafe { rkyv::access_unchecked::<ArchivedFlatAdjList>(&bytes[..]) }; // Faster
        // println!("Loading ArchivedFlatAdjList {:?}", start.elapsed());

        let start = Instant::now();
        let rv = rkyv::deserialize::<FlatAdjList, rkyv::rancor::Error>(archived)?;
        // println!("Loading FlatAdjList{:?}", start.elapsed());

        Ok(rv)
    }
}
