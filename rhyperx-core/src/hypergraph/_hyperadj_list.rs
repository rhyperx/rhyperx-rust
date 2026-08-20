use foldhash::fast::FixedState;
use hashbrown::{HashMap, HashSet};
use num_traits::AsPrimitive;
use std::{
    fmt::Debug,
    hash::Hash,
    ops::{Index, IndexMut},
};

use crate::{
    misc::OrderAndPos,
    types::{EdgeId, EdgeRef, HyperCSR, Hypergraph, NodeId},
};
#[derive(Clone)]
pub struct HyperAdjListBase<W, C: Container> {
    pub(crate) csr: HyperCSR<W>,
    pub(crate) adj: Vec<C>,

    /// for each vertex, the position in the neighbor list of the first edge with the given size and its size
    pub(crate) sizes: Vec<Vec<(usize, usize)>>,
}

impl<W, C: Container> HyperAdjListBase<W, C> {
    pub fn new() -> Self {
        Self {
            csr: HyperCSR::new(),
            adj: Vec::new(),
            sizes: Vec::new(),
        }
    }

    pub fn n(&self) -> usize {
        self.csr.n()
    }

    pub fn m(&self) -> usize {
        self.csr.m()
    }

    pub fn from_hypergraph_mapped(
        mut hg: Hypergraph<NodeId, W>,
    ) -> (Self, Vec<NodeId>, HashMap<NodeId, usize, FixedState>) {
        let (new_to_old, old_to_new) = hg.normalize_node_ids();

        let adj = Self::from_hypergraph_unmapped(hg);
        (adj, new_to_old, old_to_new)
    }

    /// Like from_hypergraph_mapped, but it assumes that node ids are distributed in the range 0..n
    pub fn from_hypergraph_unmapped<T: Hash + Eq + AsPrimitive<NodeId>>(
        mut hg: Hypergraph<T, W>,
    ) -> Self {
        let mut csr = HyperCSR::new();
        let mut edge_pos = 0;
        let mut edge_id = 0;
        // csr.lookup.resize(hg.m(), (0, 0));
        csr.lookup.reserve(hg.m());
        csr.weights.reserve(hg.m());
        csr.sizes.reserve(11);
        csr.sizes.push((0, 0));
        csr.sizes.push((0, 0));
        csr.m = hg.m();
        csr.n = hg.n();

        let mut degrees = vec![0; hg.n()];
        seq_macro::seq!(N in 2..11{
            csr.sizes.push((edge_id, hg.edges::<N>().len()));
            for edge in hg.into_iter_edges::<N>() {
                csr.lookup.push((edge_pos, N));
                for n in &edge{
                    csr.nodes.push(n.as_());
                    degrees[n.as_() as usize] += 1;
                }
                csr.weights.push(edge.weight);
                edge_id +=1;
                edge_pos += N;
            }
        });

        let mut adj = Vec::with_capacity(hg.n());
        let mut sizes = Vec::with_capacity(hg.n());
        adj.resize_with(hg.n(), C::empty);
        for v in 0..hg.n() {
            let mut node_sizes = Vec::with_capacity(11);
            node_sizes.push((0, 0));
            node_sizes.push((0, 0));
            sizes.push(node_sizes);
            adj[v].reserve(degrees[v]);
        }

        for edge_id in 0..csr.m() {
            let edge = csr.get_edge_by_id(edge_id as NodeId);
            for n in edge.nodes {
                adj[*n as usize].insert_id(edge_id as NodeId);

                let last = sizes[*n as usize].last_mut().unwrap();
                let new_size = (last.0 + last.1, 0);
                while sizes[*n as usize].len() <= edge.nodes.len() {
                    sizes[*n as usize].push(new_size);
                }

                let last = sizes[*n as usize].last_mut().unwrap();
                last.1 += 1;
            }
        }

        Self { csr, adj, sizes }
    }

    /// Returns the set of incident edges ids for a given node.
    pub fn incident_edges(&self, node_id: NodeId) -> &C {
        &self.adj[node_id as usize]
    }

    /// Returns the set of incident edges refs for a given node.
    pub fn iter_incident_edges(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = (EdgeId, EdgeRef<'_, W>)> {
        self.adj[node_id as usize]
            .iter_edge_ids()
            .map(move |&id| (id as EdgeId, self.csr.get_edge_by_id(id as EdgeId)))
    }

    pub fn count_by_size(&self, size: usize) -> usize {
        self.csr.count_by_size(size)
    }

    pub fn iter_by_size(&self, size: usize) -> impl Iterator<Item = (EdgeId, EdgeRef<'_, W>)> {
        // let (start, count) = self.csr.sizes[size];
        // (start..start + count).map(move |id| (id as EdgeId, self.csr.get_edge_by_id(id as EdgeId)))
        self.csr.iter_by_size(size)
    }

    pub fn iter_all_incident_edges(&self) -> impl Iterator<Item = &C> {
        self.adj.iter()
    }

    pub fn iter_all_incident_edges_mut(&mut self) -> impl Iterator<Item = &mut C> {
        self.adj.iter_mut()
    }

    /// Gets the oriented adj list following a provided order of nodes; This means that an hyperedge
    /// having nodes u,v,..., z will be incident only to u, given that u < v < ... < z
    pub fn get_oriented(&self, order_pos: &OrderAndPos) -> Self
    where
        C: Clone,
        W: Clone,
    {
        let OrderAndPos { order: _, pos, .. } = order_pos;
        let mut cached_min = vec![u32::MAX; self.m()];

        let mut adj = self.adj.clone();
        let mut sizes = self.sizes.clone();

        for (u, incidend_edges) in adj.iter_mut().enumerate() {
            incidend_edges.retain_ids(|edge_id| {
                let edge = self.csr.get_edge_by_id(*edge_id);
                let min = if cached_min[*edge_id as usize] == u32::MAX {
                    let min = *edge.nodes.iter().min_by_key(|n| pos[**n as usize]).unwrap();
                    cached_min[*edge_id as usize] = min;
                    min
                } else {
                    cached_min[*edge_id as usize]
                };

                sizes[u][edge.nodes.len()].1 -= (u as NodeId != min) as usize;
                u as NodeId == min
            });
        }

        for (u, _incidend_edges) in adj.iter_mut().enumerate() {
            let mut new_start = 0;
            for (start, count) in sizes[u].iter_mut() {
                *start = new_start;
                new_start += *count;
            }
        }

        HyperAdjListBase {
            csr: self.csr.clone(),
            adj,
            sizes,
        }
    }
}

impl<W> HyperAdjList<W> {
    pub fn iter_incident_by_size(
        &self,
        node: NodeId,
        size: usize,
    ) -> impl Iterator<Item = (EdgeId, EdgeRef<'_, W>)> {
        // 1. Evaluate the conditions upfront.
        // Fallback to 0..0 if anything fails, making the range instantly empty.
        let range = match self
            .sizes
            .get(node as usize)
            .and_then(|sizes| sizes.get(size))
        {
            Some(&(first_id, count)) if first_id < self.m() => first_id..(first_id + count),
            _ => 0..0,
        };

        // 2. Return a direct, single Map iterator with no flattening required.
        range.map(move |number| {
            let edge_id = self.adj[node as usize][number];
            (edge_id, self.csr.get_edge_by_id(edge_id as EdgeId))
        })
    }
}

impl<W, C: Container> Index<usize> for HyperAdjListBase<W, C> {
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        &self.adj[index]
    }
}

impl<W, C: Container> Index<NodeId> for HyperAdjListBase<W, C> {
    type Output = C;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.adj[index as usize]
    }
}

impl<W, C: Container> IndexMut<usize> for HyperAdjListBase<W, C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.adj[index]
    }
}

impl<W, C: Container> IndexMut<NodeId> for HyperAdjListBase<W, C> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.adj[index as usize]
    }
}

impl<W, C: Container> IntoIterator for HyperAdjListBase<W, C> {
    type Item = C;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.adj.into_iter()
    }
}

impl<'a, W, C: Container> IntoIterator for &'a HyperAdjListBase<W, C> {
    type Item = &'a C;
    type IntoIter = std::slice::Iter<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        self.adj.iter()
    }
}

impl<'a, W, C: Container> IntoIterator for &'a mut HyperAdjListBase<W, C> {
    type Item = &'a mut C;
    type IntoIter = std::slice::IterMut<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        self.adj.iter_mut()
    }
}

impl<W, C: Container> Debug for HyperAdjListBase<W, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for n in 0..self.n() {
            writeln!(f, "n: {}", n);
            for (edge_id, edge_ref) in self.iter_incident_edges(n as NodeId) {
                writeln!(f, "\t{} - {:?}", edge_id, edge_ref.nodes);
            }
        }
        Ok(())
    }
}

pub trait Container {
    const SUPPORTS_MULTIEDGES: bool;

    fn empty() -> Self;

    fn len(&self) -> usize;

    /// Returns true if the element was inserted successfully; This means that if
    /// SUPPORTS_MULTIEDGES is true it will always return true; if false, it will return true if no
    /// already exist

    fn insert_id(&mut self, edge: EdgeId) -> bool;

    fn iter_edge_ids(&self) -> impl Iterator<Item = &EdgeId>;

    /// Optional implementation to optimize performance
    fn reserve(&mut self, _additional: usize) {}

    fn retain_ids<F>(&mut self, f: F)
    where
        F: FnMut(&EdgeId) -> bool;

    // #[inline(always)]
    // fn into_iter_neighbors(self) -> impl Iterator<Item = &mut EdgeId>;
}

impl Container for HashSet<NodeId, FixedState> {
    const SUPPORTS_MULTIEDGES: bool = false;

    fn empty() -> Self {
        HashSet::with_hasher(FixedState::default())
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert_id(&mut self, edge: EdgeId) -> bool {
        self.insert(edge)
    }

    fn iter_edge_ids(&self) -> impl Iterator<Item = &EdgeId> {
        self.iter()
    }

    fn retain_ids<F>(&mut self, f: F)
    where
        F: FnMut(&EdgeId) -> bool,
    {
        self.retain(f);
    }
}

impl Container for Vec<NodeId> {
    const SUPPORTS_MULTIEDGES: bool = true;

    fn empty() -> Self {
        Vec::new()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert_id(&mut self, edge: EdgeId) -> bool {
        self.push(edge);
        true
    }

    fn iter_edge_ids(&self) -> impl Iterator<Item = &EdgeId> {
        self.iter()
    }

    fn retain_ids<F>(&mut self, f: F)
    where
        F: FnMut(&EdgeId) -> bool,
    {
        self.retain(f);
    }
}

pub type HyperAdjList<W> = HyperAdjListBase<W, Vec<NodeId>>;
pub type HyperAdjSet<W> = HyperAdjListBase<W, HashSet<NodeId, FixedState>>;
