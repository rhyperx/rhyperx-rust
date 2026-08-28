use hashbrown::HashMap;

use crate::{
    error::HypergraphError,
    hyperedge::{HxSizedRef, HxSizedRefMut, HxUnsizedRef, HxUnsizedRefMut, SizedHx, UnsizedHx},
    types::{EdgeId, NodeId},
};

/// Base methods supported by every hypergraph
pub trait HypergraphBase {
    type NodeIdType: NodeId;
    type WeightType;

    fn n(&self) -> usize;
    fn m(&self) -> usize;

    fn iter_edges(
        &self,
        node: Self::NodeIdType,
    ) -> impl Iterator<Item = HxUnsizedRef<'_, Self::NodeIdType, Self::WeightType>> + '_;

    fn iter_hg_sizes(&self) -> impl Iterator<Item = usize>;

    fn normalize_node_ids(
        &mut self,
    ) -> (
        Vec<Self::NodeIdType>,
        HashMap<Self::NodeIdType, Self::NodeIdType>,
    );

    // fn remove_isolated_nodes(&mut self) -> usize;
}

/// Hypergraphs where edges have unique ids.
pub trait WithEdgeId: HypergraphBase {
    type EdgeIdType: EdgeId;
}

/// Gets all edge ids incident to a node.
pub trait IncidentEdgeIdRetrieval: WithEdgeId {
    fn iter_incident_ids(
        &self,
        node: Self::NodeIdType,
    ) -> impl Iterator<Item = Self::EdgeIdType> + '_;
}

/// Gets an edge reference from its vertices.
pub trait EdgeRetrieval: HypergraphBase {
    fn get_hyperedge<W>(
        &self,
        hyperedge: HxUnsizedRef<Self::NodeIdType, W>,
    ) -> Option<HxUnsizedRef<'_, Self::NodeIdType, Self::WeightType>>;

    fn get_hyperedge_sized<const N: usize, W>(
        &self,
        hyperedge: HxUnsizedRef<Self::NodeIdType, W>,
    ) -> Option<HxSizedRef<'_, N, Self::NodeIdType, Self::WeightType>>;

    fn modify_hx_with<W, F>(&mut self, hyperedge: HxUnsizedRef<Self::NodeIdType, W>, f: F) -> bool
    where
        F: FnMut(HxUnsizedRefMut<Self::NodeIdType, Self::WeightType>);

    fn has_hyperedge<WW>(&self, edge: HxUnsizedRef<Self::NodeIdType, WW>) -> bool;
}

/// Gets a mutable edge reference from its vertices.
pub trait EdgeRetrievalMut: EdgeRetrieval {
    fn get_hyperedge_mut<W>(
        &mut self,
        hyperedge: HxUnsizedRef<Self::NodeIdType, W>,
    ) -> Option<HxUnsizedRefMut<'_, Self::NodeIdType, Self::WeightType>>;

    fn get_hyperedge_sized_mut<const N: usize, W>(
        &mut self,
        hyperedge: HxUnsizedRef<'_, Self::NodeIdType, W>,
    ) -> Option<HxSizedRefMut<'_, N, Self::NodeIdType, Self::WeightType>>;

    fn take_hyperedge<W>(
        &mut self,
        hyperedge: HxUnsizedRef<Self::NodeIdType, W>,
    ) -> Option<UnsizedHx<Self::NodeIdType, Self::WeightType>>;
}

/// Gets an edge reference from its vertices in efficient time complexity.
pub trait FastEdgeRetrieval: EdgeRetrieval {}

/// Gets an edge reference from its vertices in efficient time complexity.
pub trait FastEdgeRetrievalMut: EdgeRetrievalMut {}

/// Gets an edge reference from its id.
pub trait EdgeRetrievalById: WithEdgeId {
    fn get_hyperedge_by_id(
        &self,
        edge_id: Self::EdgeIdType,
    ) -> Option<HxUnsizedRef<'_, Self::NodeIdType, Self::WeightType>>;

    fn get_hyperedge_sized_by_id<const N: usize>(
        &self,
        edge_id: Self::EdgeIdType,
    ) -> Option<HxSizedRef<'_, N, Self::NodeIdType, Self::WeightType>>;
}

/// Gets a mutable edge reference from its id.
pub trait EdgeRetrievalMutById: EdgeRetrievalById {
    fn get_hyperedge_mut_by_id(
        &mut self,
        edge_id: Self::EdgeIdType,
    ) -> Option<HxUnsizedRefMut<'_, Self::NodeIdType, Self::WeightType>>;

    fn get_hyperedge_sized_mut_by_id<const N: usize>(
        &mut self,
        edge_id: Self::EdgeIdType,
    ) -> Option<HxSizedRefMut<'_, N, Self::NodeIdType, Self::WeightType>>;

    fn take_hyperedge_by_id(
        &mut self,
        edge_id: Self::EdgeIdType,
    ) -> Option<UnsizedHx<Self::NodeIdType, Self::WeightType>>;
}

/// Retrieve all hyperedges with a certain size.
pub trait SizedRetrieval: HypergraphBase {
    fn count_by_size(&self, size: usize) -> usize;

    fn iter_edges_by_size(
        &self,
        size: usize,
    ) -> impl Iterator<Item = HxUnsizedRef<'_, Self::NodeIdType, Self::WeightType>>;

    fn iter_edges_sized_by_size<const N: usize>(
        &self,
        size: usize,
    ) -> impl Iterator<Item = HxSizedRef<'_, N, Self::NodeIdType, Self::WeightType>>;
}

/// Efficiently retrieve all hyperedges with a certain size.
pub trait FastSizedRetrieval: SizedRetrieval {}

/// Insert hyperedges.
pub trait InsertEdge: HypergraphBase {
    fn insert_edge(&mut self, edge: UnsizedHx<Self::NodeIdType, Self::WeightType>) -> bool;

    fn insert_edge_sized<const N: usize>(
        &mut self,
        edge: SizedHx<N, Self::NodeIdType, Self::WeightType>,
    ) -> bool;

    fn insert_edge_slice(
        &mut self,
        nodes: &mut [Self::NodeIdType],
        weight: Self::WeightType,
    ) -> Result<bool, HypergraphError>;

    fn insert_edge_slice_unchecked(
        &mut self,
        nodes: &[Self::NodeIdType],
        weight: Self::WeightType,
    ) -> bool;

    fn extend_with_edges<const N: usize>(
        &mut self,
        edges: Vec<UnsizedHx<Self::NodeIdType, Self::WeightType>>,
    ) -> usize;

    fn extend_with_edges_sized<const N: usize>(
        &mut self,
        edges: Vec<SizedHx<N, Self::NodeIdType, Self::WeightType>>,
    ) -> usize;
}

/// Remove hyperedges.
pub trait RemoveEdge: EdgeRetrieval {
    fn remove_edge(
        &mut self,
        edge: UnsizedHx<Self::NodeIdType, Self::WeightType>,
    ) -> Option<UnsizedHx<Self::NodeIdType, Self::WeightType>>;

    fn remove_edge_sized<const N: usize, W>(
        &mut self,
        edge: SizedHx<N, Self::NodeIdType, W>,
    ) -> Option<SizedHx<N, Self::NodeIdType, Self::WeightType>>;

    fn remove_edge_slice(
        &mut self,
        nodes: &[Self::NodeIdType],
    ) -> Result<UnsizedHx<Self::NodeIdType, Self::WeightType>, HypergraphError>;
}

/// Efficiently remove hyperedges.
pub trait FastRemoveEdge: RemoveEdge {}

/// Remove hyperedges by id.
pub trait RemoveEdgeById: EdgeRetrievalById {
    fn remove_edge_by_id(
        &mut self,
        edge_id: Self::EdgeIdType,
    ) -> Option<UnsizedHx<Self::NodeIdType, Self::WeightType>>;
}

/// Remove hyperedges by id efficiently.
pub trait FastRemoveEdgeById: RemoveEdgeById {}
