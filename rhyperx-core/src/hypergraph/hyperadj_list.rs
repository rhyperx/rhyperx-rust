use hashbrown::{HashMap, HashSet};
use num_traits::AsPrimitive;

use crate::{
    hyperedge_container::HyperedgeContainer,
    hypergraph::hypergraph::Hypergraph,
    types::{EdgeId, NodeId},
};

#[derive(Clone, Copy, Debug)]
pub struct BucketInfo {
    pub(crate) size: usize,
    pub(crate) position: usize,
}

#[derive(Clone)]
pub struct HyperCSR<N, E, W, NC, EC>
where
    N: NodeId,
    E: EdgeId,
    NC: HyperedgeContainer<N, W>,
    EC: IdContainer<E>,
{
    pub(crate) edges: Hypergraph<N, W, NC>,

    pub(crate) adj: Vec<EC>,

    /// Lookup table; lookup[e] = index of first node of edge e in self.nodes
    // pub(crate) lookup: Vec<>,
    _phantom: std::marker::PhantomData<E>,
}

impl<N, E, W, NC, EC> HyperCSR<N, E, W, NC, EC>
where
    N: NodeId,
    E: EdgeId,
    NC: HyperedgeContainer<N, W>,
    EC: IdContainer<E>,
{
    pub fn new() -> Self {
        Self {
            edges: Hypergraph::new(),
            adj: Vec::new(),
            lookup: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn n(&self) -> usize {
        self.edges.n()
    }

    pub fn m(&self) -> usize {
        self.edges.m()
    }

    pub fn from_hypergraph<CC: HyperedgeContainer<N, W>>(mut hg: Hypergraph<N, W, CC>) -> Self {
        let mut rv = Self::new();
        let mut edge_id = 0;
        let mut edge_pos = 0;

        rv.lookup.reserve(hg.m());
        // rv.sizes.sort_unstable_by_key(|(order, _)| *order);
        rv.m = hg.m();
        rv.n = hg.n();

        // rv.sizes.push((edge_id, hg.edges::<N>().len()));

        for order in hg.iter_hg_sizes() {
            for container in hg.edges(order) {
                let bucket = rv.sizes.get_mut(&size).unwrap();
            }

            rv.lookup.push((edge_pos, N));
            for n in &edge {
                rv.nodes.push(n.as_());
            }

            rv.weights.push(edge.weight);
            edge_id += 1;
            edge_pos += N;
        }
        rv
    }

    pub fn count_by_size(&self, size: usize) -> usize {
        match self.edges.get(&size) {
            Some(bucket) => bucket.len(),
            None => 0,
        }
    }

    pub fn iter_by_size(&self, size: usize) -> impl Iterator<Item = (EdgeId, EdgeRef<'_, W>)> + '_ {
        let (first_id, count, start) = match self.sizes.get(size) {
            Some(&(first_id, count)) if first_id < self.m() => {
                (first_id, count, self.lookup[first_id].0)
            }
            _ => (0, 0, 0), // A count of 0 makes the range (0..0) instantly empty
        };

        (0..count).map(move |number| {
            let edge_id = first_id + number;
            let edge_start = start + size * number;

            let edge_ref = EdgeRef {
                nodes: &self.nodes[edge_start..edge_start + size],
                weight: &self.weights[edge_id],
            };

            (edge_id as EdgeId, edge_ref)
        })
    }

    pub fn get_edge_by_id(&self, edge_id: T) -> EdgeRef<'_, W> {
        let node_start = self.lookup[edge_id as usize].0;
        let edge_size = self.lookup[edge_id as usize].1 as usize;
        EdgeRef {
            nodes: &self.nodes[node_start..node_start + edge_size],
            weight: &self.weights[edge_id as usize],
        }
    }

    pub fn get_edge_by_id_mut(&mut self, edge_id: EdgeId) -> EdgeRefMut<'_, W> {
        let node_start = self.lookup[edge_id as usize].0;
        let edge_size = self.lookup[edge_id as usize].1 as usize;
        EdgeRefMut {
            nodes: &mut self.nodes[node_start..node_start + edge_size],
            weight: &mut self.weights[edge_id as usize],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BucketDescriptor {
    /// start index of the bucket in the flat array
    pub(crate) start: usize,
    /// size of hyperedges the bucket contains
    pub(crate) size: usize,
    /// length of the bucket, i.e. number of hyperedges in the bucket
    pub(crate) len: usize,
}

pub struct IdVecStore<E: EdgeId> {
    /// Flat list of edge ids, sorted by encreasing size
    pub(crate) ids: Vec<E>,

    /// vec[i] = (size, count) where size is the size of the edges in the bucket and count is the
    /// number of edges in the bucket. Sizes are stored in increasing order
    pub(crate) sizes_vec: Vec<BucketDescriptor>,

    /// sizes_map[size] = index of the first edge id with size "size" in the ids vector.
    pub(crate) sizes_map: HashMap<usize, usize>,
}

pub trait IdContainer<E: EdgeId> {
    /// Returns an empty container
    fn empty() -> Self;

    /// Total number of edge ids in the container
    fn len(&self) -> usize;

    /// Total number of edge ids with size "size" in the container
    fn len_by_size(&self, size: usize) -> usize;

    /// Insert an edge id into the bucket of size "size".
    /// Returns true if the edge id was inserted, false if it was already present.
    fn insert_id(&mut self, edge: E, size: usize) -> bool;

    /// Remove an edge id from the container.
    /// Returns true if the edge id was removed, false if it was not present.
    fn remove_id(&mut self, edge_id: E) -> bool;

    /// Iter all edge ids in the container
    fn iter_edge_ids<'a>(&'a self) -> impl Iterator<Item = &'a E>
    where
        E: 'a;

    /// Iter all edge ids with size "size" in the container
    fn iter_edge_ids_by_size<'a>(&'a self, size: usize) -> impl Iterator<Item = &'a E>
    where
        E: 'a;

    /// Optional implementation to optimize performance
    fn reserve(&mut self, _additional: usize) {}

    /// Retain only the edge ids that satisfy the predicate `f`. All other edge ids are removed from
    /// the container.
    fn retain_ids<F>(&mut self, f: F)
    where
        F: FnMut(&E) -> bool;

    /// Retait only the edge ids with size "size" that satisfy the predicate `f`. All other edge ids
    /// are removed from
    fn retain_ids_by_size<F>(&mut self, size: usize, f: F)
    where
        F: FnMut(&E) -> bool;

    // #[inline(always)]
    // fn into_iter_neighbors(self) -> impl Iterator<Item = &mut EdgeId>;
}

impl<E: EdgeId> IdContainer<E> for IdVecStore<E> {
    fn empty() -> Self {
        IdVecStore {
            ids: Vec::new(),
            sizes_vec: Vec::new(),
            sizes_map: HashMap::new(),
        }
    }

    fn len(&self) -> usize {
        self.ids.len()
    }

    fn len_by_size(&self, size: usize) -> usize {
        self.sizes_map
            .get(&size)
            .map_or(0, |&idx| self.sizes_vec[idx].len)
    }

    fn insert_id(&mut self, edge_id: E, size: usize) -> bool {
        if self.sizes_map.get(&size).is_none() {}

        self.ids.push(E::zero());
        // self.sizes_vec.last_mut()

        let mut curr_bucket_idx = self.sizes_vec.len() - 1;
        let mut curr_size = self.sizes_vec[curr_bucket_idx].size;

        while curr_bucket_idx > 0 && curr_size > size {
            let bucket = self.sizes_vec[curr_bucket_idx];

            self.ids[bucket.start + bucket.len] = self.ids[bucket.start];
            self.sizes_vec[curr_bucket_idx].start += 1;
            *self.sizes_map.get_mut(&curr_size).unwrap() += 1;

            curr_bucket_idx -= 1;
            curr_size = self.sizes_vec[curr_bucket_idx].size;
        }

        if curr_bucket_idx == 0 && self.sizes_map.get(&size).is_none() {
            self.ids[bucket.start + bucket.len] = self.ids[bucket.start];
            self.sizes_map.insert(size, 0);
            self.sizes_vec[0] = BucketDescriptor {
                start: 0,
                size,
                len: 1,
            };
        } else {
            let bucket = self.sizes_vec[curr_bucket_idx];
            self.ids[bucket.start + bucket.len] = edge_id;
        }

        // self.ids[]

        true
    }

    fn iter_edge_ids<'a>(&'a self) -> impl Iterator<Item = &'a E>
    where
        E: 'a,
    {
        self.iter()
    }

    fn retain_ids<F>(&mut self, f: F)
    where
        F: FnMut(&E) -> bool,
    {
        self.retain(f);
    }

    fn remove_id(&mut self, edge_id: E) -> bool {
        todo!()
    }

    fn iter_edge_ids_by_size<'a>(&'a self, size: usize) -> impl Iterator<Item = &'a E>
    where
        E: 'a,
    {
        todo!()
    }

    fn retain_ids_by_size<F>(&mut self, size: usize, f: F)
    where
        F: FnMut(&E) -> bool,
    {
        todo!()
    }
}

impl<E: EdgeId> IdContainer<E> for HashSet<E> {
    fn empty() -> Self {
        HashSet::new()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert_id(&mut self, edge: E) -> bool {
        self.insert(edge)
    }

    fn iter_edge_ids<'a>(&'a self) -> impl Iterator<Item = &'a E>
    where
        E: 'a,
    {
        self.iter()
    }

    fn retain_ids<F>(&mut self, f: F)
    where
        F: FnMut(&E) -> bool,
    {
        self.retain(f);
    }
}

// pub trait StaticAdjList<E: EdgeId> {
//     fn get_edge_ids(&self, node_id: usize) -> &[E];
//     fn get_edge_ids_mut(&mut self, node_id: usize) -> &mut [E];
// }
