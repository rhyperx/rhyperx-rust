
use super::common::{Directed, Neighbor, NeighborRef, NeighborRefMut, NodeId, Undirected};

pub trait Direction {
    const IS_DIRECTED: bool;
}

pub trait Incidence {
    type EdgeType: EdgeIdTrait;
    const HAS_INCIDENCE: bool;
}

pub trait EdgeIdTrait: Copy + Clone + Eq + Ord {
    const ZERO: Self;
    fn advance(&mut self);
}

pub trait AdjConfig {
    type Weight;
    type Dir: Direction;
    type Inc: Incidence;
    type Container: NeighborContainer<WeightType = Self::Weight, EdgeType = <Self::Inc as Incidence>::EdgeType>;
}

pub trait NeighborLike {
    type WeightType;
    type EdgeType;

    fn node(&self) -> NodeId;
    fn weight(&self) -> &Self::WeightType;
    fn weight_mut(&mut self) -> &mut Self::WeightType;
    fn edge(&self) -> Self::EdgeType;
}

pub trait DirectionalInsert<C: NeighborContainer> {
    fn insert_edge(
        adj: &mut [C],
        from: NodeId,
        to: NodeId,
        weight: C::WeightType,
        edge: C::EdgeType,
    ) -> bool;
}

impl<C: NeighborContainer> DirectionalInsert<C> for Directed {
    fn insert_edge(
        adj: &mut [C],
        from: NodeId,
        to: NodeId,
        weight: C::WeightType,
        edge: C::EdgeType,
    ) -> bool {
        adj[from as usize].insert(to, weight, edge)
    }
}

impl<C: NeighborContainer> DirectionalInsert<C> for Undirected
where
    C::WeightType: Clone,
{
    fn insert_edge(
        adj: &mut [C],
        from: NodeId,
        to: NodeId,
        weight: C::WeightType,
        edge: C::EdgeType,
    ) -> bool {
        let contained = adj[from as usize].insert(to, weight.clone(), edge);
        adj[to as usize].insert(from, weight, edge);
        contained
    }
}

pub trait NeighborContainer {
    type WeightType;
    type EdgeType: EdgeIdTrait;

    const SUPPORTS_MULTIEDGES: bool;

    fn empty() -> Self;

    fn len(&self) -> usize;

    /// Returns true if the element was inserted successfully; This means that if
    /// SUPPORTS_MULTIEDGES is true it will always return true; if false, it will return true if no
    /// already exist
    fn insert(&mut self, node: NodeId, weight: Self::WeightType, edge: Self::EdgeType) -> bool;

    
    fn iter_neighbors(
        &self,
    ) -> impl Iterator<Item = NeighborRef<'_, Self::WeightType, Self::EdgeType>>;

    fn iter_neighbors_mut(
        &mut self,
    ) -> impl Iterator<Item = NeighborRefMut<'_, Self::WeightType, Self::EdgeType>>;

    fn into_iter_neighbors(
        self,
    ) -> impl Iterator<Item = Neighbor<Self::WeightType, Self::EdgeType>>;

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(NeighborRef<Self::WeightType, Self::EdgeType>) -> bool;
}
