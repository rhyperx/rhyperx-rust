use crate::{
    hypergraph::hyperedge::HxUnsizedRef,
    hypergraph::hyperedge_container::HyperedgeContainer,
    hypergraph::hypergraph::Hypergraph,
    misc::order::{OrderAndPos, OrderType},
    types::{EdgeId, NodeId},
};

/// Stores information about where edges live inside the csr representation
#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    feature = "serialize",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)
)]
pub struct GlobalBucketInfo {
    /// Number of edges in the bucket
    pub(crate) len: usize,

    /// Order of the edges the bucket contains
    pub(crate) order: usize,

    /// Position of the first node of the bucket in the csr representation
    pub(crate) first_node_pos: usize,

    /// First edge id contained in the bucket
    pub(crate) first_id: usize,
}

/// Stores information about buckets vertex-wise
#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    feature = "serialize",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)
)]
pub struct VertexBucketInfo {
    /// Number of edges ids in the bucket
    pub(crate) len: usize,

    /// Order of the edges relative to the ids in the bucket
    pub(crate) order: usize,

    /// Position of the first id of the bucket in flat id list
    pub(crate) position: usize,
}

/// Stores information about incident edges for each vertex. The edge ids are stored in increasing
/// order of their order
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialize",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)
)]
pub struct VertexInfo<E: EdgeId> {
    /// The incident edge ids for the vertex, sorted by increasing order
    pub(crate) edge_ids: Vec<E>,

    /// Infos about the edge ids incident to the vertex
    pub(crate) bucket_infos: Vec<VertexBucketInfo>,
}

/// Adj list with immutable topology i.e. only weight can be modified. That is the fastest and most memory efficient out of the 3 structs. This is most likely the best fit to submit to complex algorithms
#[derive(Clone)]
#[cfg_attr(
    feature = "serialize",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)
)]
pub struct StaticAdjList<N, E, W>
where
    N: NodeId,
    E: EdgeId,
{
    /// Flat representation of every edge. Edges are stored in encreasing size order
    pub(self) csr: Vec<N>,

    /// Edge weights. Weights are stored in the same order as edges in the csr
    pub(self) weights: Vec<W>,

    ///Stores a map of edge_id to the position of its first vertex in the csr
    pub(self) lookup: Vec<usize>,

    /// Stores bucket infos. Items are sorted by bucket_info.size in increasing order. That is
    /// because distinc hyperedge sizes are usually a small number, so binary search becomes
    /// more efficient than set lookup
    pub(self) bucket_infos: Vec<GlobalBucketInfo>,

    /// Stores for each node the list of edge ids incident to it. Edge id are sorte in encreasing order
    pub(crate) adj: Vec<VertexInfo<E>>,
}

impl<N, E, W> StaticAdjList<N, E, W>
where
    N: NodeId,
    E: EdgeId,
{
    /// Creates a new empty static adjacency list. Usually end users should construct the adj list
    /// starting from a graph directly, given the inherently static nature of the adj list
    pub fn new() -> Self {
        Self {
            csr: Vec::new(),
            weights: Vec::new(),
            lookup: Vec::new(),
            adj: Vec::new(),
            bucket_infos: Vec::new(),
        }
    }

    /// Constructs the static adjacency list from a hypergraph. The hypergraph is consumed and its
    /// edges are removed.
    pub fn from_hypergraph_unmapped<C: HyperedgeContainer<N, W>>(
        mut hg: Hypergraph<N, W, C>,
    ) -> Self {
        let mut rv = Self::new();

        let mut orders: Vec<usize> = hg.iter_hg_sizes().collect();
        orders.sort_unstable();

        let csr_size: usize = orders
            .iter()
            .map(|&order| order * hg.edges_count(order))
            .sum();
        let m = hg.m();
        let n = hg.n();

        rv.csr.reserve(csr_size);
        rv.weights.reserve(m);
        rv.lookup.reserve(m);
        rv.bucket_infos.reserve(orders.len());
        rv.adj.resize_with(n, || VertexInfo {
            edge_ids: Vec::new(),
            bucket_infos: Vec::new(),
        });

        let mut next_bucket_start = 0;
        for order in orders {
            let mut container = hg.edges.remove(&order).unwrap();
            let flat_nodes = container.take_flat_nodes();
            let weights = container.take_weights();
            let edges_count = weights.len();

            rv.bucket_infos.push(GlobalBucketInfo {
                len: edges_count,
                order,
                first_node_pos: next_bucket_start,
                first_id: rv.lookup.len(),
            });

            next_bucket_start += edges_count * order;

            for (w, edge_nodes) in weights.into_iter().zip(flat_nodes.chunks_exact(order)) {
                let next_edge_id = rv.lookup.len();
                rv.lookup.push(rv.csr.len());
                rv.weights.push(w);

                for &next_node in edge_nodes {
                    let v_info = &mut rv.adj[next_node.as_usize()];
                    let curr_ids_count = v_info.edge_ids.len();

                    match v_info.bucket_infos.last_mut() {
                        Some(last) if last.order == order => {
                            last.len += 1;
                        }
                        _ => {
                            v_info.bucket_infos.push(VertexBucketInfo {
                                len: 1,
                                order,
                                position: curr_ids_count, // FIXED: removed the `- 1`
                            });
                        }
                    }

                    v_info.edge_ids.push(E::from_usize(next_edge_id));
                    rv.csr.push(next_node);
                }
            }
        }

        rv
    }

    #[inline(always)]
    pub fn n(&self) -> usize {
        self.adj.len()
    }

    #[inline(always)]
    pub fn m(&self) -> usize {
        self.weights.len()
    }

    /// complexity: O(K) where K is the number of different hyperedge sizes in the graph
    ///
    /// # Panics
    ///
    /// Panics if `edge_id` invalid (i.e., `edge_id.as_usize()` is greater than or
    /// equal to the total number of edges), as edges are numbered from 0 to m-1.
    #[inline(always)]
    pub fn get_edge_by_id_unchecked(&self, edge_id: E) -> HxUnsizedRef<'_, N, W> {
        let id = edge_id.as_usize();
        let node_start = self.lookup[id];

        // Since distinct hyperedge orders are typically very small,
        // a linear scan over bucket_infos is faster than binary search (fully unrolled by LLVM).
        let mut edge_order = 0;
        for info in &self.bucket_infos {
            if id < info.first_node_pos + info.len {
                edge_order = info.order;
                break;
            }
        }

        HxUnsizedRef {
            nodes: &self.csr[node_start..node_start + edge_order],
            weight: &self.weights[id],
        }
    }

    /// complexity: O(1)
    ///
    /// # Panics
    ///
    /// Panics if `edge_id` invalid (i.e., `edge_id.as_usize()` is greater than or
    /// equal to the total number of edges), as edges are numbered from 0 to m-1.
    #[inline(always)]
    pub fn get_edge_by_id_and_size_unchecked(
        &self,
        edge_id: E,
        size: usize,
    ) -> HxUnsizedRef<'_, N, W> {
        let id = edge_id.as_usize();
        let node_start = self.lookup[id];

        HxUnsizedRef {
            nodes: &self.csr[node_start..node_start + size],
            weight: &self.weights[id],
        }
    }

    /// Returns an iterator over all edge IDs of a given size.
    ///
    /// Edges ids are stored in increasing order of their IDs
    /// Returns an iterator of edge IDs for a given size. The iterator is empty if there are no edges of the given size.
    pub fn edge_ids_by_size(&self, order: usize) -> impl Iterator<Item = E> + '_ {
        let range = self
            .bucket_infos
            .iter()
            .find(|bucket| bucket.order == order)
            .map(|bucket| bucket.first_id..(bucket.first_id + bucket.len))
            .unwrap_or(0..0);

        range.map(E::from_usize)
    }

    /// Returns an iterator over incident edge IDs for the given node.
    ///
    /// Edges ids are stored in increasing order of their IDs
    pub fn incident_edge_ids(&self, node_id: N) -> impl Iterator<Item = E> + '_ {
        self.adj[node_id.as_usize()].edge_ids.iter().cloned()
    }

    /// Returns an iterator over incident edge IDs of the fiven size for the given node.
    ///
    /// Edges ids are stored in increasing order of their IDs
    pub fn incident_edge_ids_by_size(
        &self,
        node_id: N,
        order: usize,
    ) -> impl Iterator<Item = E> + '_ {
        let (start, end) = self.adj[node_id.as_usize()]
            .bucket_infos
            .iter()
            .find(|bucket| bucket.order == order)
            .map(|bucket| (bucket.position, bucket.position + bucket.len))
            .unwrap_or((0, 0));

        self.adj[node_id.as_usize()].edge_ids[start..end]
            .iter()
            .cloned()
    }

    /// Counts the total number of edges of a specific size.
    pub fn count_by_size(&self, order: usize) -> usize {
        match self
            .bucket_infos
            .iter()
            .find(|bucket| bucket.order == order)
        {
            Some(info) => info.len,
            None => 0,
        }
    }

    /// Counts the total number of incident edges for a given node.
    pub fn count_incident(&self, node_id: N) -> usize {
        self.adj[node_id.as_usize()].edge_ids.len()
    }

    /// Counts the total number of incident edges of a specific size for a given node.
    pub fn count_incident_by_size(&self, node_id: N, order: usize) -> usize {
        self.adj[node_id.as_usize()]
            .bucket_infos
            .iter()
            .find(|bucket| bucket.order == order)
            .map(|bucket| bucket.len)
            .unwrap_or(0)
    }

    /// Iterates over all edges
    pub fn iter_edges(&self) -> GlobalEdgesIterator<'_, N, E, W> {
        GlobalEdgesIterator::new(self)
    }

    /// Iterates over all edges of a specific size.
    pub fn iter_by_size(
        &self,
        size: usize,
    ) -> impl Iterator<Item = (E, HxUnsizedRef<'_, N, W>)> + '_ {
        let bucket = self.bucket_infos.iter().find(|bucket| bucket.order == size);
        let (first_id, len) = match bucket {
            Some(bucket) => (bucket.first_id, bucket.len),
            None => (0, 0),
        };

        (0..len).map(move |i| {
            let edge_id = E::from_usize(first_id + i);
            (
                edge_id,
                self.get_edge_by_id_and_size_unchecked(edge_id, size),
            )
        })
    }

    /// Returns an iterator over incident edges (ID and reference) for a given node.
    pub fn iter_incident_edges(&self, node_id: N) -> IncidentEdgesIterator<'_, N, E, W> {
        IncidentEdgesIterator::new(self, node_id)
    }

    /// Returns an iterator over incident edges (ID and reference) with the specified size for a given node.
    pub fn iter_incident_edges_by_size(
        &self,
        node_id: N,
        size: usize,
    ) -> impl Iterator<Item = (E, HxUnsizedRef<'_, N, W>)> + '_ {
        let ids = &self.adj[node_id.as_usize()].edge_ids;
        let bucket = self.adj[node_id.as_usize()]
            .bucket_infos
            .iter()
            .find(|bucket| bucket.order == size);

        let id_range = match bucket {
            Some(bucket) => &ids[bucket.position..bucket.position + bucket.len],
            None => &[],
        };

        id_range.iter().map(move |&edge_id| {
            (
                edge_id,
                self.get_edge_by_id_and_size_unchecked(edge_id, size),
            )
        })
    }

    /// Gets the oriented adjacency list following a provided node ordering.
    /// An hyperedge with nodes u, v, ..., z will be incident only to its minimum node.
    pub fn orient<O: OrderType>(&mut self, order: &OrderAndPos<N, O>) {
        let mut minimums = vec![usize::MAX; self.m()];
        for (edge_id, edge) in self.iter_edges() {
            minimums[edge_id.as_usize()] = edge
                .nodes
                .iter()
                .min_by_key(|n| order.pos[n.as_usize()])
                .unwrap()
                .as_usize();
        }

        for (node_id, v_info) in self.adj.iter_mut().enumerate() {
            let mut ids_retain_idx = 0;
            let mut ids_read_idx = 0;

            let mut buckets_retain_idx = 0;
            let mut buckets_read_idx = 0;

            let mut idx_in_bucket = 0;

            let mut retained_in_bucket_count = 0;

            while ids_read_idx < v_info.edge_ids.len() {
                let edge_id = v_info.edge_ids[ids_read_idx].as_usize();

                // retain
                if minimums[edge_id] == node_id {
                    v_info.edge_ids[ids_retain_idx] = v_info.edge_ids[ids_read_idx];
                    retained_in_bucket_count += 1;
                    ids_retain_idx += 1;
                }

                idx_in_bucket += 1;
                if idx_in_bucket == v_info.bucket_infos[buckets_read_idx].len {
                    if retained_in_bucket_count > 0 {
                        v_info.bucket_infos[buckets_retain_idx] = VertexBucketInfo {
                            len: retained_in_bucket_count,
                            order: v_info.bucket_infos[buckets_read_idx].order,
                            position: ids_retain_idx - retained_in_bucket_count,
                        };
                        buckets_retain_idx += 1;
                    }

                    buckets_read_idx += 1;
                    retained_in_bucket_count = 0;
                    idx_in_bucket = 0;
                }

                ids_read_idx += 1;
            }

            v_info.edge_ids.truncate(ids_retain_idx);
            v_info.bucket_infos.truncate(buckets_retain_idx);
        }
    }
}

/// Iterator over all edges in the graph
pub struct GlobalEdgesIterator<'a, N, E, W>
where
    N: NodeId,
    E: EdgeId,
{
    /// The adj list
    adj_list: &'a StaticAdjList<N, E, W>,
    /// The index of the edge in bucket currently iterated
    bucket_index: usize,
    /// The index of the edge in bucket currently iterated
    edge_index_in_bucket: usize,
    /// The number of edge in the currently iterated bucket
    edge_index: usize,
}

impl<'a, N, E, W> GlobalEdgesIterator<'a, N, E, W>
where
    N: NodeId,
    E: EdgeId,
{
    pub fn new(adj_list: &'a StaticAdjList<N, E, W>) -> Self {
        Self {
            adj_list,
            bucket_index: 0,
            edge_index_in_bucket: 0,
            edge_index: 0,
        }
    }
}

impl<'a, N, E, W> Iterator for GlobalEdgesIterator<'a, N, E, W>
where
    N: NodeId,
    E: EdgeId,
{
    type Item = (E, HxUnsizedRef<'a, N, W>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.edge_index == self.adj_list.m() {
            return None;
        }

        let curr_bucket = &self.adj_list.bucket_infos[self.bucket_index];
        let order = curr_bucket.order;
        let next_id = E::from_usize(curr_bucket.first_id + self.edge_index_in_bucket);
        let edge_ref = self
            .adj_list
            .get_edge_by_id_and_size_unchecked(next_id, order);

        self.edge_index += 1;
        self.edge_index_in_bucket += 1;
        if self.edge_index_in_bucket == curr_bucket.len {
            self.bucket_index += 1;
            self.edge_index_in_bucket = 0;
        }

        Some((next_id, edge_ref))
    }
}

/// Iterator over all edges incident to a vertex
pub struct IncidentEdgesIterator<'a, N, E, W>
where
    N: NodeId,
    E: EdgeId,
{
    /// The adj list
    adj_list: &'a StaticAdjList<N, E, W>,
    /// The node for which we are iterating incident edges
    node: N,
    /// The index of the edge in bucket currently iterated
    bucket_index: usize,
    /// The index of the edge in bucket currently iterated
    edge_index_in_bucket: usize,
    /// The number of edge in the currently iterated bucket
    edge_index: usize,
}

impl<'a, N, E, W> IncidentEdgesIterator<'a, N, E, W>
where
    N: NodeId,
    E: EdgeId,
{
    pub fn new(adj_list: &'a StaticAdjList<N, E, W>, node: N) -> Self {
        Self {
            adj_list,
            node,
            bucket_index: 0,
            edge_index_in_bucket: 0,
            edge_index: 0,
        }
    }
}

impl<'a, N, E, W> Iterator for IncidentEdgesIterator<'a, N, E, W>
where
    N: NodeId,
    E: EdgeId,
{
    type Item = (E, HxUnsizedRef<'a, N, W>);

    fn next(&mut self) -> Option<Self::Item> {
        let bucket_infos = &self.adj_list.adj[self.node.as_usize()].bucket_infos;
        let ids = &self.adj_list.adj[self.node.as_usize()].edge_ids;

        if self.edge_index == ids.len() {
            return None;
        }

        let curr_bucket = &bucket_infos[self.bucket_index];
        let order = curr_bucket.order;
        let next_id = ids[self.edge_index];
        let edge_ref = self
            .adj_list
            .get_edge_by_id_and_size_unchecked(next_id, order);

        self.edge_index += 1;
        self.edge_index_in_bucket += 1;
        if self.edge_index_in_bucket == curr_bucket.len {
            self.bucket_index += 1;
            self.edge_index_in_bucket = 0;
        }

        Some((next_id, edge_ref))
    }
}

// pub trait StaticAdjList<E: EdgeId> {
//     fn get_edge_ids(&self, node_id: usize) -> &[E];
//     fn get_edge_ids_mut(&mut self, node_id: usize) -> &mut [E];
// }
//
//
// #[derive(Clone)]
// pub struct HyperCSR<N, E, W, NC, EC>
// where
//     N: NodeId,
//     E: EdgeId,
//     NC: HyperedgeContainer<N, W>,
//     EC: IdContainer<E>,
// {
//     pub(crate) edges: Hypergraph<N, W, NC>,
//
//     pub(crate) adj: Vec<EC>,
//
//     /// Lookup table; lookup[e] = index of first node of edge e in self.nodes
//     // pub(crate) lookup: Vec<>,
//     _phantom: std::marker::PhantomData<E>,
// }

// impl<N, E, W, NC, EC> HyperCSR<N, E, W, NC, EC>
// where
//     N: NodeId,
//     E: EdgeId,
//     NC: HyperedgeContainer<N, W>,
//     EC: IdContainer<E>,
// {
//     pub fn new() -> Self {
//         Self {
//             edges: Hypergraph::new(),
//             adj: Vec::new(),
//             lookup: Vec::new(),
//             _phantom: std::marker::PhantomData,
//         }
//     }
//
//     pub fn n(&self) -> usize {
//         self.edges.n()
//     }
//
//     pub fn m(&self) -> usize {
//         self.edges.m()
//     }
//
//     pub fn from_hypergraph<CC: HyperedgeContainer<N, W>>(mut hg: Hypergraph<N, W, CC>) -> Self {
//         let mut rv = Self::new();
//         let mut edge_id = 0;
//         let mut edge_pos = 0;
//
//         rv.lookup.reserve(hg.m());
//         // rv.sizes.sort_unstable_by_key(|(order, _)| *order);
//         rv.m = hg.m();
//         rv.n = hg.n();
//
//         // rv.sizes.push((edge_id, hg.edges::<N>().len()));
//
//         for order in hg.iter_hg_sizes() {
//             for container in hg.edges(order) {
//                 let bucket = rv.sizes.get_mut(&size).unwrap();
//             }
//
//             rv.lookup.push((edge_pos, N));
//             for n in &edge {
//                 rv.nodes.push(n.as_());
//             }
//
//             rv.weights.push(edge.weight);
//             edge_id += 1;
//             edge_pos += N;
//         }
//         rv
//     }
//
//     pub fn count_by_size(&self, size: usize) -> usize {
//         match self.edges.get(&size) {
//             Some(bucket) => bucket.len(),
//             None => 0,
//         }
//     }
//
//     pub fn iter_by_size(&self, size: usize) -> impl Iterator<Item = (EdgeId, EdgeRef<'_, W>)> + '_ {
//         let (first_id, count, start) = match self.sizes.get(size) {
//             Some(&(first_id, count)) if first_id < self.m() => {
//                 (first_id, count, self.lookup[first_id].0)
//             }
//             _ => (0, 0, 0), // A count of 0 makes the range (0..0) instantly empty
//         };
//
//         (0..count).map(move |number| {
//             let edge_id = first_id + number;
//             let edge_start = start + size * number;
//
//             let edge_ref = EdgeRef {
//                 nodes: &self.nodes[edge_start..edge_start + size],
//                 weight: &self.weights[edge_id],
//             };
//
//             (edge_id as EdgeId, edge_ref)
//         })
//     }
//
//     pub fn get_edge_by_id(&self, edge_id: T) -> EdgeRef<'_, W> {
//         let node_start = self.lookup[edge_id as usize].0;
//         let edge_size = self.lookup[edge_id as usize].1 as usize;
//         EdgeRef {
//             nodes: &self.nodes[node_start..node_start + edge_size],
//             weight: &self.weights[edge_id as usize],
//         }
//     }
//
//     pub fn get_edge_by_id_mut(&mut self, edge_id: EdgeId) -> EdgeRefMut<'_, W> {
//         let node_start = self.lookup[edge_id as usize].0;
//         let edge_size = self.lookup[edge_id as usize].1 as usize;
//         EdgeRefMut {
//             nodes: &mut self.nodes[node_start..node_start + edge_size],
//             weight: &mut self.weights[edge_id as usize],
//         }
//     }
// }
//
// #[derive(Clone, Copy, Debug)]
// pub struct BucketDescriptor {
//     /// start index of the bucket in the flat array
//     pub(crate) start: usize,
//     /// size of hyperedges the bucket contains
//     pub(crate) size: usize,
//     /// length of the bucket, i.e. number of hyperedges in the bucket
//     pub(crate) len: usize,
// }
//
// pub struct IdVecStore<E: EdgeId> {
//     /// Flat list of edge ids, sorted by encreasing size
//     pub(crate) ids: Vec<E>,
//
//     /// vec[i] = (size, count) where size is the size of the edges in the bucket and count is the
//     /// number of edges in the bucket. Sizes are stored in increasing order
//     pub(crate) sizes_vec: Vec<BucketDescriptor>,
//
//     /// sizes_map[size] = index of the first edge id with size "size" in the ids vector.
//     pub(crate) sizes_map: HashMap<usize, usize>,
// }
//
// pub trait IdContainer<E: EdgeId> {
//     /// Returns an empty container
//     fn empty() -> Self;
//
//     /// Total number of edge ids in the container
//     fn len(&self) -> usize;
//
//     /// Total number of edge ids with size "size" in the container
//     fn len_by_size(&self, size: usize) -> usize;
//
//     /// Insert an edge id into the bucket of size "size".
//     /// Returns true if the edge id was inserted, false if it was already present.
//     fn insert_id(&mut self, edge: E, size: usize) -> bool;
//
//     /// Remove an edge id from the container.
//     /// Returns true if the edge id was removed, false if it was not present.
//     fn remove_id(&mut self, edge_id: E) -> bool;
//
//     /// Iter all edge ids in the container
//     fn iter_edge_ids<'a>(&'a self) -> impl Iterator<Item = &'a E>
//     where
//         E: 'a;
//
//     /// Iter all edge ids with size "size" in the container
//     fn iter_edge_ids_by_size<'a>(&'a self, size: usize) -> impl Iterator<Item = &'a E>
//     where
//         E: 'a;
//
//     /// Optional implementation to optimize performance
//     fn reserve(&mut self, _additional: usize) {}
//
//     /// Retain only the edge ids that satisfy the predicate `f`. All other edge ids are removed from
//     /// the container.
//     fn retain_ids<F>(&mut self, f: F)
//     where
//         F: FnMut(&E) -> bool;
//
//     /// Retait only the edge ids with size "size" that satisfy the predicate `f`. All other edge ids
//     /// are removed from
//     fn retain_ids_by_size<F>(&mut self, size: usize, f: F)
//     where
//         F: FnMut(&E) -> bool;
//
//     // #[inline(always)]
//     // fn into_iter_neighbors(self) -> impl Iterator<Item = &mut EdgeId>;
// }
//
// impl<E: EdgeId> IdContainer<E> for IdVecStore<E> {
//     fn empty() -> Self {
//         IdVecStore {
//             ids: Vec::new(),
//             sizes_vec: Vec::new(),
//             sizes_map: HashMap::new(),
//         }
//     }
//
//     fn len(&self) -> usize {
//         self.ids.len()
//     }
//
//     fn len_by_size(&self, size: usize) -> usize {
//         self.sizes_map
//             .get(&size)
//             .map_or(0, |&idx| self.sizes_vec[idx].len)
//     }
//
//     fn insert_id(&mut self, edge_id: E, size: usize) -> bool {
//         if self.sizes_map.get(&size).is_none() {}
//
//         self.ids.push(E::zero());
//         // self.sizes_vec.last_mut()
//
//         let mut curr_bucket_idx = self.sizes_vec.len() - 1;
//         let mut curr_size = self.sizes_vec[curr_bucket_idx].size;
//
//         while curr_bucket_idx > 0 && curr_size > size {
//             let bucket = self.sizes_vec[curr_bucket_idx];
//
//             self.ids[bucket.start + bucket.len] = self.ids[bucket.start];
//             self.sizes_vec[curr_bucket_idx].start += 1;
//             *self.sizes_map.get_mut(&curr_size).unwrap() += 1;
//
//             curr_bucket_idx -= 1;
//             curr_size = self.sizes_vec[curr_bucket_idx].size;
//         }
//
//         if curr_bucket_idx == 0 && self.sizes_map.get(&size).is_none() {
//             self.ids[bucket.start + bucket.len] = self.ids[bucket.start];
//             self.sizes_map.insert(size, 0);
//             self.sizes_vec[0] = BucketDescriptor {
//                 start: 0,
//                 size,
//                 len: 1,
//             };
//         } else {
//             let bucket = self.sizes_vec[curr_bucket_idx];
//             self.ids[bucket.start + bucket.len] = edge_id;
//         }
//
//         // self.ids[]
//
//         true
//     }
//
//     fn iter_edge_ids<'a>(&'a self) -> impl Iterator<Item = &'a E>
//     where
//         E: 'a,
//     {
//         self.iter()
//     }
//
//     fn retain_ids<F>(&mut self, f: F)
//     where
//         F: FnMut(&E) -> bool,
//     {
//         self.retain(f);
//     }
//
//     fn remove_id(&mut self, edge_id: E) -> bool {
//         todo!()
//     }
//
//     fn iter_edge_ids_by_size<'a>(&'a self, size: usize) -> impl Iterator<Item = &'a E>
//     where
//         E: 'a,
//     {
//         todo!()
//     }
//
//     fn retain_ids_by_size<F>(&mut self, size: usize, f: F)
//     where
//         F: FnMut(&E) -> bool,
//     {
//         todo!()
//     }
// }
//
// impl<E: EdgeId> IdContainer<E> for HashSet<E> {
//     fn empty() -> Self {
//         HashSet::new()
//     }
//
//     fn len(&self) -> usize {
//         self.len()
//     }
//
//     fn insert_id(&mut self, edge: E) -> bool {
//         self.insert(edge)
//     }
//
//     fn iter_edge_ids<'a>(&'a self) -> impl Iterator<Item = &'a E>
//     where
//         E: 'a,
//     {
//         self.iter()
//     }
//
//     fn retain_ids<F>(&mut self, f: F)
//     where
//         F: FnMut(&E) -> bool,
//     {
//         self.retain(f);
//     }
// }
