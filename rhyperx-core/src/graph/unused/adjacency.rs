use foldhash::fast::FixedState;
use hashbrown::hash_map::Entry;
use hashbrown::{HashMap, HashSet};
use rust_core_macros::hoist_mod;
use std::cmp::max;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut, Range, RangeBounds};

use duplicate::duplicate_item;
use rkyv::{Archive, Deserialize, Serialize};

use super::base::{
    AdjacencyBase, AdjacencyList, AdjacencySet, Directed, Directionality, DirectionalityDependent,
    Undirected,
};
use crate::misc::serialize::DumpCacheToFile;
use crate::types::{EdgeId, NodeId, NodeWeight};

pub trait AdjBase<W> {
    fn add_edge(&mut self, u: NodeId, v: NodeId, w: W)
    where
        W: Clone;
    fn remove_edges_between(&mut self, u: NodeId, v: NodeId);
}

/// A neighbor node ID, used for storing in a vec.
///
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct ListNeighbor<N, W> {
    /// The node ID
    pub node: N,
    /// The weight of the edge connecting to this neighbor.
    pub weight: W,
}

impl<N, W> ListNeighbor<N, W> {
    pub fn new(node: N, weight: W) -> Self {
        Self { node, weight }
    }
}

/// An adjacency list where each node's neighbors are stored in a Vec.
/// No edge IDs are stored (unlike the incidence list).
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct AdjList<W, D = Undirected>
where
    D: Directionality,
{
    adj: Vec<Vec<ListNeighbor<NodeId, W>>>,
    n: usize,
    m: usize,

    _directionality_marker: PhantomData<D>,
}

/// An adjacency set where each node's neighbors are stored in a HashMap.
/// The value stored is just the weight `W` (no edge ID).
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct AdjSet<W, D = Undirected>
where
    D: Directionality,
{
    adj: Vec<HashMap<NodeId, W, FixedState>>,
    n: usize,
    m: usize,

    _directionality_marker: PhantomData<D>,
}

// Index / IndexMut implementations (shared by both structs)
impl<W, D> Index<usize> for AdjList<W, D>
where
    D: Directionality,
{
    type Output = Vec<ListNeighbor<NodeId, W>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.adj[index]
    }
}

impl<W, D> IndexMut<usize> for AdjList<W, D>
where
    D: Directionality,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.adj[index]
    }
}

impl<W, D> Index<NodeId> for AdjList<W, D>
where
    D: Directionality,
{
    type Output = Vec<ListNeighbor<NodeId, W>>;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.adj[index as usize]
    }
}

impl<W, D> IndexMut<NodeId> for AdjList<W, D>
where
    D: Directionality,
{
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.adj[index as usize]
    }
}

impl<W, D> Default for AdjList<W, D>
where
    D: Directionality,
{
    fn default() -> Self {
        Self {
            adj: Vec::new(),
            n: 0,
            m: 0,
            _directionality_marker: PhantomData,
        }
    }
}

impl<W, D> Index<usize> for AdjSet<W, D>
where
    D: Directionality,
{
    type Output = HashMap<NodeId, W, FixedState>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.adj[index]
    }
}

impl<W, D> IndexMut<usize> for AdjSet<W, D>
where
    D: Directionality,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.adj[index]
    }
}

impl<W, D> Index<NodeId> for AdjSet<W, D>
where
    D: Directionality,
{
    type Output = HashMap<NodeId, W, FixedState>;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.adj[index as usize]
    }
}

impl<W, D> IndexMut<NodeId> for AdjSet<W, D>
where
    D: Directionality,
{
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.adj[index as usize]
    }
}

impl<W, D> Default for AdjSet<W, D>
where
    D: Directionality,
{
    fn default() -> Self {
        Self {
            adj: Vec::new(),
            n: 0,
            m: 0,
            _directionality_marker: PhantomData,
        }
    }
}

// Directionality dependent implementations for AdjList and AdjSet
impl<W> DirectionalityDependent<W> for AdjList<W, Undirected> {
    #[inline(always)]
    fn add_edge(&mut self, u: NodeId, v: NodeId, w: W)
    where
        W: Clone,
    {
        self.adj[u as usize].push(ListNeighbor::new(v, w.clone()));
        self.adj[v as usize].push(ListNeighbor::new(u, w));
        self.m += 1;
    }

    #[inline(always)]
    fn remove_edges_between(&mut self, u: NodeId, v: NodeId) {
        self.adj[u as usize].retain(|e| e.node != v);
        self.adj[v as usize].retain(|e| e.node != u);
    }
}

impl<W> DirectionalityDependent<W> for AdjList<W, Directed> {
    #[inline(always)]
    fn add_edge(&mut self, u: NodeId, v: NodeId, w: W)
    where
        W: Clone,
    {
        self.adj[u as usize].push(ListNeighbor::new(v, w));
        self.m += 1;
    }

    #[inline(always)]
    fn remove_edges_between(&mut self, u: NodeId, v: NodeId) {
        self.adj[u as usize].retain(|e| e.node != v);
    }
}

impl<W> DirectionalityDependent<W> for AdjSet<W, Undirected> {
    #[inline(always)]
    fn add_edge(&mut self, u: NodeId, v: NodeId, w: W)
    where
        W: Clone,
    {
        self.adj[u as usize].insert(v, w.clone());
        self.adj[v as usize].insert(u, w);
        self.m += 1;
    }

    #[inline(always)]
    fn remove_edges_between(&mut self, u: NodeId, v: NodeId) {
        self.adj[u as usize].remove(&v);
        self.adj[v as usize].remove(&u);
    }
}

impl<W> DirectionalityDependent<W> for AdjSet<W, Directed> {
    #[inline(always)]
    fn add_edge(&mut self, u: NodeId, v: NodeId, w: W)
    where
        W: Clone,
    {
        self.adj[u as usize].insert(v, w);
        self.m += 1;
    }

    #[inline(always)]
    fn remove_edges_between(&mut self, u: NodeId, v: NodeId) {
        self.adj[u as usize].remove(&v);
    }
}

// AdjacencyBase implementation for AdjList and AdjSet
impl<W, D> AdjacencyBase<W> for AdjList<W, D>
where
    D: Directionality,
    Self: DirectionalityDependent<W>,
{
    type NeighborContainer = Vec<ListNeighbor<NodeId, W>>;

    fn new() -> Self {
        Self::default()
    }

    fn with_nodes(n: usize) -> Self {
        Self {
            adj: (0..n).map(|_| Vec::new()).collect(),
            n,
            m: 0,
            _directionality_marker: PhantomData,
        }
    }

    fn from_edges_mapped(
        edges: Vec<(NodeId, NodeId, W)>,
    ) -> (
        AdjList<W, D>,
        Vec<NodeId>,
        HashMap<NodeId, NodeId, FixedState>,
    )
    where
        W: Clone,
    {
        if edges.is_empty() {
            return (
                Self::new(),
                Vec::new(),
                HashMap::with_hasher(FixedState::default()),
            );
        }

        // Step 1: assign compact IDs
        let mut compressed_index: HashMap<NodeId, NodeId, FixedState> =
            HashMap::with_hasher(FixedState::default());
        let mut curr_number = 0;

        for (u, v, _w) in edges.iter() {
            if let Entry::Vacant(e) = compressed_index.entry(*u) {
                e.insert(curr_number);
                curr_number += 1;
            }
            if let Entry::Vacant(e) = compressed_index.entry(*v) {
                e.insert(curr_number);
                curr_number += 1;
            }
        }

        let n = curr_number as usize;

        // Step 2: compute adjacency sizes
        let mut sizes = vec![0; n];

        for (u, v, _) in edges.iter() {
            let u_idx = compressed_index[u];
            let v_idx = compressed_index[v];
            sizes[u_idx as usize] += 1;
            sizes[v_idx as usize] += 1;
        }

        // Step 3: allocate graph
        let mut rv = Self::with_nodes(n);

        for (u, adj) in rv.adj.iter_mut().enumerate() {
            adj.reserve(sizes[u]);
        }

        // Step 4: fill adjacency
        for (u, v, w) in edges.iter() {
            let u_idx = compressed_index[u];
            let v_idx = compressed_index[v];

            rv.add_edge(u_idx, v_idx, w.clone());
        }

        rv.m = edges.len();
        let mut original_index = Vec::with_capacity(n);
        original_index.resize(n, 0);
        for (node, &compressed) in compressed_index.iter() {
            original_index[compressed as usize] = *node;
        }

        (rv, original_index, compressed_index)
    }

    fn from_edges_unmapped(edges: Vec<(NodeId, NodeId, W)>) -> Self
    where
        W: Clone,
    {
        if edges.is_empty() {
            return Self::new();
        }

        let n = (edges
            .iter()
            .fold(0, |acc, (u, v, _w)| max(acc, max(*u, *v)))
            + 1) as usize;

        let mut sizes = vec![0; n];

        for (u, _, _) in edges.iter() {
            sizes[*u as usize] += 1;
        }

        let mut rv = Self::with_nodes(n);

        for (u, adj) in rv.adj.iter_mut().enumerate() {
            adj.reserve(sizes[u]);
        }

        for (u, v, w) in edges.iter() {
            rv.add_edge(*u, *v, w.clone());
        }

        rv.m = edges.len();
        rv
    }

    fn n(&self) -> usize {
        self.n
    }

    fn m(&self) -> usize {
        self.m
    }

    fn remove_self_loops(&mut self) -> usize {
        let mut removed = 0;
        for (x, neighbors) in self.adj.iter_mut().enumerate() {
            neighbors.retain(|neighbor| {
                if neighbor.node as usize == x {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
        self.m -= removed;
        removed
    }

    fn iter_neighbors<'a>(&'a self) -> impl Iterator<Item = &'a Vec<ListNeighbor<NodeId, W>>> + 'a
    where
        W: 'a,
    {
        self.adj.iter()
    }

    fn iter_neighbors_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut Vec<ListNeighbor<NodeId, W>>> + 'a
    where
        W: 'a,
    {
        self.adj.iter_mut()
    }

    fn drain_neighbors(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> impl Iterator<Item = Vec<ListNeighbor<NodeId, W>>> + '_ {
        self.adj.drain(range)
    }
}

impl<W, D> AdjacencyBase<W> for AdjSet<W, D>
where
    D: Directionality,
    Self: DirectionalityDependent<W>,
{
    type NeighborContainer = HashMap<NodeId, W, FixedState>;

    fn new() -> Self {
        Self::default()
    }

    fn with_nodes(n: usize) -> Self {
        Self {
            adj: (0..n)
                .map(|_| HashMap::with_hasher(FixedState::default()))
                .collect(),
            n,
            m: 0,
            _directionality_marker: PhantomData,
        }
    }

    fn from_edges_mapped(
        edges: Vec<(NodeId, NodeId, W)>,
    ) -> (
        AdjSet<W, D>,
        Vec<NodeId>,
        HashMap<NodeId, NodeId, FixedState>,
    )
    where
        W: Clone,
    {
        if edges.is_empty() {
            return (
                Self::new(),
                Vec::new(),
                HashMap::with_hasher(FixedState::default()),
            );
        }

        let mut compressed_index: HashMap<NodeId, NodeId, FixedState> =
            HashMap::with_hasher(FixedState::default());
        let mut curr_number = 0;

        for (u, v, _w) in edges.iter() {
            if let Entry::Vacant(e) = compressed_index.entry(*u) {
                e.insert(curr_number);
                curr_number += 1;
            }
            if let Entry::Vacant(e) = compressed_index.entry(*v) {
                e.insert(curr_number);
                curr_number += 1;
            }
        }

        let n = curr_number as usize;

        let mut rv = Self::with_nodes(n);

        for (u, v, w) in edges.iter() {
            let u_idx = compressed_index[u];
            let v_idx = compressed_index[v];
            rv.add_edge(u_idx, v_idx, w.clone());
        }

        rv.m = edges.len();

        let mut original_index = Vec::with_capacity(n);
        original_index.resize(n, 0);
        for (node, &compressed) in compressed_index.iter() {
            original_index[compressed as usize] = *node;
        }

        (rv, original_index, compressed_index)
    }

    fn from_edges_unmapped(edges: Vec<(NodeId, NodeId, W)>) -> Self
    where
        W: Clone,
    {
        if edges.is_empty() {
            return Self::new();
        }

        let n = (edges
            .iter()
            .fold(0, |acc, (u, v, _w)| max(acc, max(*u, *v)))
            + 1) as usize;

        let mut rv = Self::with_nodes(n);

        for (u, v, w) in edges.iter() {
            rv.add_edge(*u, *v, w.clone());
        }

        rv.m = edges.len();
        rv
    }

    fn n(&self) -> usize {
        self.n
    }

    fn m(&self) -> usize {
        self.m
    }

    fn remove_self_loops(&mut self) -> usize {
        let mut removed = 0;
        for (x, neighbors) in self.adj.iter_mut().enumerate() {
            if neighbors.remove(&(x as NodeId)).is_some() {
                removed += 1;
            }
        }
        self.m -= removed;
        removed
    }

    fn iter_neighbors<'a>(&'a self) -> impl Iterator<Item = &'a HashMap<NodeId, W, FixedState>> + 'a
    where
        W: 'a,
    {
        self.adj.iter()
    }

    fn iter_neighbors_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut HashMap<NodeId, W, FixedState>> + 'a
    where
        W: 'a,
    {
        self.adj.iter_mut()
    }

    fn drain_neighbors(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> impl Iterator<Item = HashMap<NodeId, W, FixedState>> + '_ {
        self.adj.drain(range)
    }
}

// List/Set methods
impl<W, D> AdjacencyList<W> for AdjList<W, D>
where
    D: Directionality,
    W: Clone,
    Self: DirectionalityDependent<W>,
{
    fn sort_neighbors(&mut self) {
        let n = self.n();
        let mut rv = vec![Vec::new(); n];

        for u in 0..n {
            rv[u].reserve(self[u].len());
        }

        for u in 0..n {
            for neighbor in self.adj[u].drain(..) {
                rv[neighbor.node as usize].push(ListNeighbor::new(u as NodeId, neighbor.weight));
            }
        }
        self.adj = rv;
    }

    fn remove_multiedges(&mut self) -> usize {
        let mut removed = 0;
        let mut present = HashSet::new();

        for neighbors in self.adj.iter_mut() {
            present.clear();
            neighbors.retain(|neighbor| {
                if !present.insert(neighbor.node) {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
        self.m -= removed;
        removed
    }
}

impl<W, D> AdjacencySet<W> for AdjSet<W, D>
where
    W: Clone,
    D: Directionality,
    Self: DirectionalityDependent<W>,
{
}

// Inherent methods

impl<W> AdjList<W, Undirected> {
    pub fn into_directed(mut self) -> AdjList<W, Directed> {
        AdjList {
            adj: self.adj,
            n: self.n,
            m: self.m,
            _directionality_marker: PhantomData,
        }
    }
}

impl<W> AdjList<W, Directed> {
    /// Makes the directed graph undirected by adding an edge (v,u) for each edge (u,v). If
    /// `allow_multiedges` is set to `false`, it will remove any multiedges that may arise from this
    /// operation.
    pub fn into_undirected(mut self, allow_multiedges: bool) -> AdjList<W, Undirected>
    where
        W: Clone,
    {
        let mut rv = AdjList::<W, Undirected>::with_nodes(self.n());
        for (x, neighbors) in self.drain_neighbors(..).enumerate() {
            rv.adj[x].reserve(neighbors.len());

            for neighbor in neighbors.into_iter() {
                rv.add_edge(x as NodeId, neighbor.node, neighbor.weight);
            }
        }

        if !allow_multiedges {
            rv.remove_multiedges();
        }

        rv
    }
}

impl<W> AdjSet<W, Undirected> {
    pub fn into_directed(mut self) -> AdjSet<W, Directed> {
        AdjSet {
            adj: self.adj,
            n: self.n,
            m: self.m,
            _directionality_marker: PhantomData,
        }
    }
}

impl<W> AdjSet<W, Directed> {
    /// Convert from directed to undirected by adding the reverse edges.
    pub fn into_undirected(mut self) -> AdjSet<W, Undirected>
    where
        W: Clone,
    {
        let mut rv = AdjSet::<W, Undirected>::with_nodes(self.n());
        for (x, neighbors) in self.drain_neighbors(..).enumerate() {
            for (v, w) in neighbors.into_iter() {
                rv.add_edge(x as NodeId, v, w);
            }
        }
        rv
    }
}

// Python bindings
#[cfg(feature = "bindings")]
#[hoist_mod]
mod bindings {
    use pyo3::{FromPyObject, PyRef, pyclass, pymethods};
    use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
    use pyo3_stub_gen::{PyStubType, impl_stub_type};

    // ------------------------------------------------------------------------
    // AdjList bindings
    // ------------------------------------------------------------------------

    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
    #[gen_stub_pyclass(module = "rust_core._core.graph")]
    #[pyclass(from_py_object)]
    pub struct UnweightedAdjList(pub AdjList<()>);

    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
    #[gen_stub_pyclass(module = "rust_core._core.graph")]
    #[pyclass(from_py_object)]
    pub struct WeightedAdjList(pub AdjList<NodeWeight>);

    #[duplicate_item(
    adj_type            weight_type  tuple_edge_type;
    [UnweightedAdjList] [()]         [(NodeId, NodeId)];
    [WeightedAdjList]   [NodeWeight] [(NodeId, NodeId, NodeWeight)];
    )]
    #[gen_stub_pymethods(module = "rust_core._core.graph")]
    #[pymethods]
    impl adj_type {
        #[new]
        pub fn new() -> Self {
            Self(AdjList::new())
        }

        #[staticmethod]
        pub fn with_nodes(n: usize) -> Self {
            Self(AdjList::with_nodes(n))
        }

        #[staticmethod]
        pub fn from_edges_mapped(
            edges: Vec<tuple_edge_type>,
        ) -> (adj_type, Vec<NodeId>, HashMap<NodeId, NodeId, FixedState>) {
            let edges = Self::normalize_edge_list(edges);

            let (adj, m1, m2) = AdjList::<weight_type>::from_edges_mapped(edges);
            (Self(adj), m1, m2)
        }

        #[staticmethod]
        pub fn from_edges_unmapped(edges: Vec<tuple_edge_type>) -> Self {
            let edges = Self::normalize_edge_list(edges);
            Self(AdjList::from_edges_unmapped(edges))
        }

        pub fn remove_multiedges(&mut self) -> usize {
            self.0.remove_multiedges()
        }

        pub fn remove_self_loops(&mut self) -> usize {
            self.0.remove_self_loops()
        }

        /// Efficiently sorts each adjacency list in-place.
        /// Time Complexity: O(n + m)
        /// Space Complexity: O(n + m)
        pub fn sort_neighbors(&mut self) {
            self.0.sort_neighbors()
        }
    }

    impl UnweightedAdjList {
        pub(crate) fn normalize_edge_list(
            edges: Vec<(NodeId, NodeId)>,
        ) -> Vec<(NodeId, NodeId, ())> {
            edges.into_iter().map(|(u, v)| (u, v, ())).collect()
        }
    }

    impl WeightedAdjList {
        pub(crate) fn normalize_edge_list(
            edges: Vec<(NodeId, NodeId, NodeWeight)>,
        ) -> Vec<(NodeId, NodeId, NodeWeight)> {
            edges
        }
    }

    #[derive(FromPyObject)]
    pub enum PyAdjList<'py> {
        Weighted(PyRef<'py, WeightedAdjList>),
        Unweighted(PyRef<'py, UnweightedAdjList>),
    }

    impl_stub_type!(PyAdjList<'_> = WeightedAdjList | UnweightedAdjList);

    impl Deref for UnweightedAdjList {
        type Target = AdjList<()>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for UnweightedAdjList {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl Deref for WeightedAdjList {
        type Target = AdjList<NodeWeight>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for WeightedAdjList {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    // ------------------------------------------------------------------------
    // AdjSet bindings (same functionality, HashMap‑backed)
    // ------------------------------------------------------------------------

    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
    #[gen_stub_pyclass(module = "rust_core._core.graph")]
    #[pyclass(from_py_object)]
    pub struct UnweightedAdjSet(pub AdjSet<()>);

    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
    #[gen_stub_pyclass(module = "rust_core._core.graph")]
    #[pyclass(from_py_object)]
    pub struct WeightedAdjSet(pub AdjSet<NodeWeight>);

    #[duplicate_item(
    adj_type            weight_type  tuple_edge_type;
    [UnweightedAdjSet] [()]         [(NodeId, NodeId)];
    [WeightedAdjSet]   [NodeWeight] [(NodeId, NodeId, NodeWeight)];
    )]
    #[gen_stub_pymethods(module = "rust_core._core.graph")]
    #[pymethods]
    impl adj_type {
        #[new]
        pub fn new() -> Self {
            Self(AdjSet::new())
        }

        #[staticmethod]
        pub fn with_nodes(n: usize) -> Self {
            Self(AdjSet::with_nodes(n))
        }

        #[staticmethod]
        pub fn from_edges_mapped(
            edges: Vec<tuple_edge_type>,
        ) -> (adj_type, Vec<NodeId>, HashMap<NodeId, NodeId, FixedState>) {
            let edges = Self::normalize_edge_list(edges);
            let (adj, m1, m2) = AdjSet::<weight_type>::from_edges_mapped(edges);
            (Self(adj), m1, m2)
        }

        #[staticmethod]
        pub fn from_edges_unmapped(edges: Vec<tuple_edge_type>) -> Self {
            let edges = Self::normalize_edge_list(edges);
            Self(AdjSet::from_edges_unmapped(edges))
        }

        pub fn remove_self_loops(&mut self) -> usize {
            self.0.remove_self_loops()
        }

        /// No‑op for AdjSet because the underlying HashMap already makes
        /// multi‑edges impossible, and ordering is irrelevant.
        pub fn sort_neighbors(&mut self) {
            // no‑op
        }
    }

    impl UnweightedAdjSet {
        pub(crate) fn normalize_edge_list(
            edges: Vec<(NodeId, NodeId)>,
        ) -> Vec<(NodeId, NodeId, ())> {
            edges.into_iter().map(|(u, v)| (u, v, ())).collect()
        }
    }

    impl WeightedAdjSet {
        pub(crate) fn normalize_edge_list(
            edges: Vec<(NodeId, NodeId, NodeWeight)>,
        ) -> Vec<(NodeId, NodeId, NodeWeight)> {
            edges
        }
    }

    #[derive(FromPyObject)]
    pub enum PyAdjSet<'py> {
        Weighted(PyRef<'py, WeightedAdjSet>),
        Unweighted(PyRef<'py, UnweightedAdjSet>),
    }

    impl_stub_type!(PyAdjSet<'_> = WeightedAdjSet | UnweightedAdjSet);

    impl Deref for UnweightedAdjSet {
        type Target = AdjSet<()>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for UnweightedAdjSet {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl Deref for WeightedAdjSet {
        type Target = AdjSet<NodeWeight>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for WeightedAdjSet {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}
