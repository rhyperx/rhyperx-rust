use crate::types;

pub trait Direction {
    const IS_DIRECTED: bool;
}

/// Configuration for adjacency lists without edge IDs.
pub trait AdjConfig {
    type Weight;
    type Dir: Direction;
    type NodeId: types::NodeId;
    type Container: NeighborContainer<WeightType = Self::Weight, NodeType = Self::NodeId>;
}

/// Configuration for adjacency lists with edge IDs.
pub trait IncConfig {
    type Weight;
    type Dir: Direction;
    type NodeId: types::NodeId;
    type EdgeId: types::EdgeId;
    type Container: IncNeighborContainer<
            WeightType = Self::Weight,
            NodeType = Self::NodeId,
            EdgeType = Self::EdgeId,
        >;
}

// ── Neighbor (no edge ID) ─────────────────────────

#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    feature = "serialize",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)
)]
pub struct Neighbor<N, W> {
    pub node: N,
    pub weight: W,
}

impl<N: Eq, W> PartialEq for Neighbor<N, W> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<N: Eq, W> Eq for Neighbor<N, W> {}

impl<N: Ord, W> PartialOrd for Neighbor<N, W> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.node.cmp(&other.node))
    }
}

impl<N: Ord, W> Ord for Neighbor<N, W> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.node.cmp(&other.node)
    }
}

// ── IncNeighbor (with edge ID) ────────────────────

#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    feature = "serialize",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)
)]
pub struct IncNeighbor<N, W, E> {
    pub node: N,
    pub weight: W,
    pub edge: E,
}

impl<N: Eq, W, E: Eq> PartialEq for IncNeighbor<N, W, E> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.edge == other.edge
    }
}

impl<N: Eq, W, E: Eq> Eq for IncNeighbor<N, W, E> {}

impl<N: Ord, W, E: Ord> PartialOrd for IncNeighbor<N, W, E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.node.cmp(&other.node).then(self.edge.cmp(&other.edge)))
    }
}

impl<N: Ord, W, E: Ord> Ord for IncNeighbor<N, W, E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

// ── Ref types for non-inc containers ──────────────

pub struct NeighborRef<'a, N, W> {
    pub node: &'a N,
    pub weight: &'a W,
}

pub struct NeighborRefMut<'a, N, W> {
    pub node: &'a N,
    pub weight: &'a mut W,
}

impl<'a, N, W> Clone for NeighborRef<'a, N, W> {
    fn clone(&self) -> Self {
        Self {
            node: self.node,
            weight: self.weight,
        }
    }
}

impl<'a, N, W> Copy for NeighborRef<'a, N, W> {}

// ── Ref types for inc containers ──────────────────

pub struct IncNeighborRef<'a, N, W, E> {
    pub node: &'a N,
    pub weight: &'a W,
    pub edge: &'a E,
}

pub struct IncNeighborRefMut<'a, N, W, E> {
    pub node: &'a N,
    pub weight: &'a mut W,
    pub edge: &'a E,
}

impl<'a, N, W, E> Clone for IncNeighborRef<'a, N, W, E> {
    fn clone(&self) -> Self {
        Self {
            node: self.node,
            weight: self.weight,
            edge: self.edge,
        }
    }
}

impl<'a, N, W, E> Copy for IncNeighborRef<'a, N, W, E> {}

// ── NeighborContainer (no edge IDs) ───────────────

pub trait NeighborContainer {
    type WeightType;
    type NodeType: types::NodeId;

    const SUPPORTS_MULTIEDGES: bool;

    fn empty() -> Self;
    fn len(&self) -> usize;

    fn insert(&mut self, node: Self::NodeType, weight: Self::WeightType) -> bool;

    fn iter_neighbors(
        &self,
    ) -> impl Iterator<Item = NeighborRef<'_, Self::NodeType, Self::WeightType>>;

    fn iter_neighbors_mut(
        &mut self,
    ) -> impl Iterator<Item = NeighborRefMut<'_, Self::NodeType, Self::WeightType>>;

    fn into_iter_neighbors(
        self,
    ) -> impl Iterator<Item = Neighbor<Self::NodeType, Self::WeightType>>;

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(NeighborRef<Self::NodeType, Self::WeightType>) -> bool;
}

// ── IncNeighborContainer (with edge IDs) ──────────

pub trait IncNeighborContainer {
    type WeightType;
    type NodeType: types::NodeId;
    type EdgeType: types::EdgeId;

    const SUPPORTS_MULTIEDGES: bool;

    fn empty() -> Self;
    fn len(&self) -> usize;

    fn insert_inc(
        &mut self,
        node: Self::NodeType,
        weight: Self::WeightType,
        edge: Self::EdgeType,
    ) -> bool;

    fn iter_neighbors_inc(
        &self,
    ) -> impl Iterator<Item = IncNeighborRef<'_, Self::NodeType, Self::WeightType, Self::EdgeType>>;

    fn iter_neighbors_inc_mut(
        &mut self,
    ) -> impl Iterator<Item = IncNeighborRefMut<'_, Self::NodeType, Self::WeightType, Self::EdgeType>>;

    fn into_iter_neighbors_inc(
        self,
    ) -> impl Iterator<Item = IncNeighbor<Self::NodeType, Self::WeightType, Self::EdgeType>>;

    fn retain_inc<F>(&mut self, f: F)
    where
        F: FnMut(IncNeighborRef<Self::NodeType, Self::WeightType, Self::EdgeType>) -> bool;
}
