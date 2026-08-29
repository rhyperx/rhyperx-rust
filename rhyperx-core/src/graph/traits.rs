use super::adjacency::{AdjBase, AdjConfig, AdjList, Neighbor, NeighborContainer, NeighborRef};
use super::incidence::{
    IncBase, IncConfig, IncList, IncNeighbor, IncNeighborContainer, IncNeighborRef,
};
use crate::types::{EdgeId, NodeId};

pub trait Direction {
    const IS_DIRECTED: bool;
}

// ── Graph traits ──────────────────────────────────

/// Base methods supported by every graph representation.
pub trait GraphBase {
    type NodeIdType: NodeId;
    type WeightType;
    type Dir: Direction;

    const SUPPORTS_MULTIEDGES: bool;

    fn n(&self) -> usize;
    fn m(&self) -> usize;
}

// TODO: add so that it supports index operation []. Adj[i] should return the container with
// neighbors of node i; indexable by usize. I guess the container type should become part of the trait
/// Per-node neighbor retrieval.
pub trait NeighborRetrieval: GraphBase {
    fn degree(&self, node: Self::NodeIdType) -> usize;

    fn iter_neighbors(&self, node: Self::NodeIdType)
    -> impl Iterator<Item = Self::NodeIdType> + '_;

    fn iter_weighted_neighbors(
        &self,
        node: Self::NodeIdType,
    ) -> impl Iterator<Item = NeighborRef<'_, Self::NodeIdType, Self::WeightType>> + '_;
}

/// A single neighbor entry readable through [`IndexedNeighbors`].
pub trait IndexedNeighborEntry {
    type NodeIdType: NodeId;
    type WeightType;

    fn node(&self) -> Self::NodeIdType;
    fn weight(&self) -> &Self::WeightType;
}

impl<N: NodeId, W> IndexedNeighborEntry for Neighbor<N, W> {
    type NodeIdType = N;
    type WeightType = W;

    fn node(&self) -> N {
        self.node
    }

    fn weight(&self) -> &W {
        &self.weight
    }
}

impl<N: NodeId, W, E> IndexedNeighborEntry for IncNeighbor<N, W, E> {
    type NodeIdType = N;
    type WeightType = W;

    fn node(&self) -> N {
        self.node
    }

    fn weight(&self) -> &W {
        &self.weight
    }
}

/// Per-node neighbor retrieval on index based. Basically Adj/Inc list with vec container
pub trait IndexedNeighbors: NeighborRetrieval {
    /// The type of a single neighbor entry in the container.
    type Neighbor: IndexedNeighborEntry<NodeIdType = Self::NodeIdType, WeightType = Self::WeightType>;

    /// Returns the neighbor container of `node` as a slice.
    fn neighbors(&self, node: Self::NodeIdType) -> &[Self::Neighbor];

    /// Returns the `idx`-th neighbor of `node`.
    fn neighbor_node(&self, node: Self::NodeIdType, idx: usize) -> Self::NodeIdType {
        self.neighbors(node)[idx].node()
    }

    /// Returns a reference to the weight of the `idx`-th neighbor of `node`.
    fn neighbor_weight(&self, node: Self::NodeIdType, idx: usize) -> &Self::WeightType {
        self.neighbors(node)[idx].weight()
    }
}

/// [`IndexedNeighbors`] whose neighbor containers can be mutated in place.
///
/// No sorting-order guarantee is provided: the ordering of a slice is unspecified unless the
/// algorithm itself sorts it (e.g. through [`IndexedNeighborsMut::sort_neighbors_by_key`]).
pub trait IndexedNeighborsMut: IndexedNeighbors {
    /// Returns the neighbor container of `node` as a mutable slice.
    fn neighbors_mut(&mut self, node: Self::NodeIdType) -> &mut [Self::Neighbor];

    /// Sorts the neighbors of `node` by the key extracted from each entry.
    fn sort_neighbors_by_key<K>(
        &mut self,
        node: Self::NodeIdType,
        key: impl FnMut(&Self::NodeIdType, &Self::WeightType) -> K,
    ) where
        K: Ord,
    {
        let mut key = key;
        self.neighbors_mut(node)
            .sort_by_key(|n| key(&n.node(), n.weight()));
    }
}

/// An edge with both endpoints and a reference to its weight.
pub struct EdgeRef<'a, N, W> {
    pub from: N,
    pub to: N,
    pub weight: &'a W,
}

impl<N: Clone, W> Clone for EdgeRef<'_, N, W> {
    fn clone(&self) -> Self {
        Self {
            from: self.from.clone(),
            to: self.to.clone(),
            weight: self.weight,
        }
    }
}

impl<N: Copy, W> Copy for EdgeRef<'_, N, W> {}

/// Whole-graph edge iteration.
pub trait EdgeIteration: GraphBase {
    /// Iterates over logical edges; every parallel edge and self-loop is
    /// yielded exactly once.
    fn iter_edges(
        &self,
    ) -> impl Iterator<Item = EdgeRef<'_, Self::NodeIdType, Self::WeightType>> + '_;
}

/// Graphs whose edges carry unique ids.
pub trait EdgeIdGraph: GraphBase {
    type EdgeIdType: EdgeId;

    fn iter_incident_neighbors(
        &self,
        node: Self::NodeIdType,
    ) -> impl Iterator<
        Item = IncNeighborRef<'_, Self::NodeIdType, Self::WeightType, Self::EdgeIdType>,
    > + '_;
}

/// Edge insertion; returns whether the edge count increased.
pub trait InsertEdge: GraphBase {
    fn insert_edge(
        &mut self,
        from: Self::NodeIdType,
        to: Self::NodeIdType,
        weight: Self::WeightType,
    ) -> bool
    where
        Self::WeightType: Clone;
}

/// Edge insertion in graphs with edge ids; returns the id of the inserted edge.
pub trait InsertEdgeWithId: EdgeIdGraph {
    fn insert_edge(
        &mut self,
        from: Self::NodeIdType,
        to: Self::NodeIdType,
        weight: Self::WeightType,
    ) -> Self::EdgeIdType
    where
        Self::WeightType: Clone;
}

/// Edge removal.
pub trait RemoveEdge: GraphBase {
    /// Removes every edge between `from` and `to`; returns how many were removed.
    fn remove_edges_between(&mut self, from: Self::NodeIdType, to: Self::NodeIdType) -> usize;

    fn remove_self_loops(&mut self) -> usize;
}

/// Multiedge queries and removal.
pub trait MultiedgeOps: GraphBase {
    fn count_multiedges(&self) -> usize;
    fn has_multiedges(&self) -> bool;
    fn remove_multiedges(&mut self) -> usize;
}

// ── AdjBase blanket impls ─────────────────────────

impl<C: AdjConfig> GraphBase for AdjBase<C> {
    type NodeIdType = C::NodeId;
    type WeightType = C::Weight;
    type Dir = C::Dir;

    const SUPPORTS_MULTIEDGES: bool = C::Container::SUPPORTS_MULTIEDGES;

    fn n(&self) -> usize {
        AdjBase::n(self)
    }

    fn m(&self) -> usize {
        AdjBase::m(self)
    }
}

impl<C: AdjConfig> NeighborRetrieval for AdjBase<C> {
    fn degree(&self, node: C::NodeId) -> usize {
        self[node].len()
    }

    fn iter_neighbors(&self, node: C::NodeId) -> impl Iterator<Item = C::NodeId> + '_ {
        self[node].iter_neighbors().map(|n| *n.node)
    }

    fn iter_weighted_neighbors(
        &self,
        node: C::NodeId,
    ) -> impl Iterator<Item = NeighborRef<'_, C::NodeId, C::Weight>> + '_ {
        self[node].iter_neighbors()
    }
}

impl<C: AdjConfig> EdgeIteration for AdjBase<C> {
    fn iter_edges(&self) -> impl Iterator<Item = EdgeRef<'_, C::NodeId, C::Weight>> + '_ {
        self.iter_neighbors()
            .enumerate()
            .flat_map(|(u, container)| {
                let mut self_loops_seen = 0usize;
                container.iter_neighbors().filter_map(move |n| {
                    let v = n.node.as_usize();
                    let emit = if C::Dir::IS_DIRECTED || v > u {
                        true
                    } else if v == u {
                        // Undirected self-loops are stored twice per logical edge,
                        // except after `remove_multiedges` where only one copy is
                        // left: emitting every other occurrence yields exactly one
                        // edge per logical self-loop in both cases.
                        self_loops_seen += 1;
                        self_loops_seen % 2 == 1
                    } else {
                        false
                    };
                    emit.then(|| EdgeRef {
                        from: C::NodeId::from_usize(u),
                        to: *n.node,
                        weight: n.weight,
                    })
                })
            })
    }
}

impl<C: AdjConfig> InsertEdge for AdjBase<C> {
    fn insert_edge(&mut self, from: C::NodeId, to: C::NodeId, weight: C::Weight) -> bool
    where
        Self::WeightType: Clone,
    {
        let m_before = AdjBase::m(self);
        AdjBase::insert_edge(self, from, to, weight);
        AdjBase::m(self) > m_before
    }
}

impl<C: AdjConfig> RemoveEdge for AdjBase<C> {
    fn remove_edges_between(&mut self, from: C::NodeId, to: C::NodeId) -> usize {
        AdjBase::remove_edges_between(self, from, to)
    }

    fn remove_self_loops(&mut self) -> usize {
        AdjBase::remove_self_loops(self)
    }
}

impl<C: AdjConfig> MultiedgeOps for AdjBase<C> {
    fn count_multiedges(&self) -> usize {
        AdjBase::count_multiedges(self)
    }

    fn has_multiedges(&self) -> bool {
        AdjBase::has_multiedges(self)
    }

    fn remove_multiedges(&mut self) -> usize {
        AdjBase::remove_multiedges(self)
    }
}

// ── IncBase blanket impls ──────────────────────────

impl<C: IncConfig> GraphBase for IncBase<C> {
    type NodeIdType = C::NodeId;
    type WeightType = C::Weight;
    type Dir = C::Dir;

    const SUPPORTS_MULTIEDGES: bool = C::Container::SUPPORTS_MULTIEDGES;

    fn n(&self) -> usize {
        IncBase::n(self)
    }

    fn m(&self) -> usize {
        IncBase::m(self)
    }
}

impl<C: IncConfig> NeighborRetrieval for IncBase<C> {
    fn degree(&self, node: C::NodeId) -> usize {
        self[node].len()
    }

    fn iter_neighbors(&self, node: C::NodeId) -> impl Iterator<Item = C::NodeId> + '_ {
        self[node].iter_neighbors_inc().map(|n| *n.node)
    }

    fn iter_weighted_neighbors(
        &self,
        node: C::NodeId,
    ) -> impl Iterator<Item = NeighborRef<'_, C::NodeId, C::Weight>> + '_ {
        self[node].iter_neighbors_inc().map(|n| NeighborRef {
            node: n.node,
            weight: n.weight,
        })
    }
}

impl<C: IncConfig> EdgeIteration for IncBase<C> {
    fn iter_edges(&self) -> impl Iterator<Item = EdgeRef<'_, C::NodeId, C::Weight>> + '_ {
        self.iter_neighbors()
            .enumerate()
            .flat_map(|(u, container)| {
                let mut self_loops_seen = 0usize;
                container.iter_neighbors_inc().filter_map(move |n| {
                    let v = n.node.as_usize();
                    let emit = if C::Dir::IS_DIRECTED || v > u {
                        true
                    } else if v == u {
                        // Undirected self-loops are stored twice per logical edge,
                        // except after `remove_multiedges` where only one copy is
                        // left: emitting every other occurrence yields exactly one
                        // edge per logical self-loop in both cases.
                        self_loops_seen += 1;
                        self_loops_seen % 2 == 1
                    } else {
                        false
                    };
                    emit.then(|| EdgeRef {
                        from: C::NodeId::from_usize(u),
                        to: *n.node,
                        weight: n.weight,
                    })
                })
            })
    }
}

impl<C: IncConfig> EdgeIdGraph for IncBase<C> {
    type EdgeIdType = C::EdgeId;

    fn iter_incident_neighbors(
        &self,
        node: C::NodeId,
    ) -> impl Iterator<Item = IncNeighborRef<'_, C::NodeId, C::Weight, C::EdgeId>> + '_ {
        self[node].iter_neighbors_inc()
    }
}

impl<C: IncConfig> InsertEdgeWithId for IncBase<C> {
    fn insert_edge(&mut self, from: C::NodeId, to: C::NodeId, weight: C::Weight) -> C::EdgeId
    where
        Self::WeightType: Clone,
    {
        IncBase::insert_edge(self, from, to, weight)
    }
}

impl<C: IncConfig> RemoveEdge for IncBase<C> {
    fn remove_edges_between(&mut self, from: C::NodeId, to: C::NodeId) -> usize {
        IncBase::remove_edges_between(self, from, to)
    }

    fn remove_self_loops(&mut self) -> usize {
        IncBase::remove_self_loops(self)
    }
}

impl<C: IncConfig> MultiedgeOps for IncBase<C> {
    fn count_multiedges(&self) -> usize {
        IncBase::count_multiedges(self)
    }

    fn has_multiedges(&self) -> bool {
        IncBase::has_multiedges(self)
    }

    fn remove_multiedges(&mut self) -> usize {
        IncBase::remove_multiedges(self)
    }
}

// ── Vec-backed indexable impls ─────────────────────

impl<N: NodeId, W, D: Direction> IndexedNeighbors for AdjList<N, W, D> {
    type Neighbor = Neighbor<N, W>;

    fn neighbors(&self, node: N) -> &[Neighbor<N, W>] {
        &self[node]
    }
}

impl<N: NodeId, W, D: Direction> IndexedNeighborsMut for AdjList<N, W, D> {
    fn neighbors_mut(&mut self, node: N) -> &mut [Neighbor<N, W>] {
        &mut self[node]
    }
}

impl<N: NodeId, W, D: Direction, E: EdgeId> IndexedNeighbors for IncList<N, W, D, E> {
    type Neighbor = IncNeighbor<N, W, E>;

    fn neighbors(&self, node: N) -> &[IncNeighbor<N, W, E>] {
        &self[node]
    }
}

impl<N: NodeId, W, D: Direction, E: EdgeId> IndexedNeighborsMut for IncList<N, W, D, E> {
    fn neighbors_mut(&mut self, node: N) -> &mut [IncNeighbor<N, W, E>] {
        &mut self[node]
    }
}
