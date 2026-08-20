use crate::{hyperedge::HxUnsizedRef, types::{EdgeId, NodeId}};

pub trait DynamicHypergraph: StaticHypergraphBase {}

/// Gets efficiently all edges incident to a node.
pub trait StaticHypergraphBase {
    fn n(&self) -> usize;
    fn m(&self) -> usize;
}

/// Gets efficiently all edges id incident to a node.
pub trait IncidentEdgeIdRetrival {
    type NodeIdType: NodeId;
    type EdgeIdType: EdgeId;

    fn iter_incident_ids(
        &self,
        node: Self::NodeIdType,
    ) -> impl Iterator<Item = Self::EdgeIdType> + '_;
}

/// Gets efficiently all edges incident to a node.
pub trait IncidentEdgeRetrival {
    type NodeType: NodeId;
    type Weight: EdgeWeight;

    fn iter_incident_edges(
        &self,
        node: Self::NodeType,
    ) -> impl Iterator<Item = HxUnsizedRef<Self::NodeType, Self::>> + '_;
    // ...
}

// pub trait AddEdge<E> {
//     fn add_edge(&mut self, edge: E);
// }

// pub trait GetHxBySize {
//     fn edges(&self, size: usize) -> Option<&C> {}
// }

pub trait HypergraphConf<N, E> {
    const X: usize;
}
