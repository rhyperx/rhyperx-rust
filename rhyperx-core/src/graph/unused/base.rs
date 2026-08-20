use std::ops::{Index, IndexMut, RangeBounds};

use crate::types::{NodeId, adjacency::ListNeighbor};
use foldhash::fast::FixedState;
use hashbrown::HashMap;

pub trait Directionality {}

pub struct Directed;
impl Directionality for Directed {}

pub struct Undirected;
impl Directionality for Undirected {}

// Methods requiring different implementation based on directionality
pub trait DirectionalityDependent<W> {
    fn add_edge(&mut self, u: NodeId, v: NodeId, w: W)
    where
        W: Clone;

    fn remove_edges_between(&mut self, u: NodeId, v: NodeId);
}

// Methods that can be implemented in a directionality-independent way
pub trait AdjacencyBase<W>:
    DirectionalityDependent<W> + Index<usize> + Index<NodeId> + IndexMut<usize> + IndexMut<NodeId>
where
    Self: Sized,
{
    type NeighborContainer;

    fn new() -> Self;

    fn with_nodes(n: usize) -> Self;

    fn from_edges_mapped(
        edges: Vec<(NodeId, NodeId, W)>,
    ) -> (Self, Vec<NodeId>, HashMap<NodeId, NodeId, FixedState>)
    where
        W: Clone;

    fn from_edges_unmapped(edges: Vec<(NodeId, NodeId, W)>) -> Self
    where
        W: Clone;

    fn n(&self) -> usize;

    fn m(&self) -> usize;

    fn remove_self_loops(&mut self) -> usize;

    fn iter_neighbors<'a>(&'a self) -> impl Iterator<Item = &'a Self::NeighborContainer> + 'a
    where
        W: 'a;

    fn iter_neighbors_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut Self::NeighborContainer> + 'a
    where
        W: 'a;

    fn drain_neighbors(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> impl Iterator<Item = Self::NeighborContainer> + '_;
}

// Implemented by both adjacency list and incidence list
pub trait AdjacencyList<W>:
    AdjacencyBase<W>
{
    fn remove_multiedges(&mut self) -> usize;

    fn sort_neighbors(&mut self);
}

// Implemented by both adjacency set and incidence set
pub trait AdjacencySet<W>:
    AdjacencyBase<W>
{
}
