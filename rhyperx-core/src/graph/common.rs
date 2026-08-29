use std::hash::Hash;

use foldhash::fast::FixedState;
use hashbrown::HashMap;

use super::adjacency::{AdjConfig, Neighbor, NeighborContainer, NeighborRef, NeighborRefMut};
use super::incidence::{
    IncConfig, IncNeighbor, IncNeighborContainer, IncNeighborRef, IncNeighborRefMut,
};
use super::traits::Direction;
use crate::types::{EdgeId, NodeId};

#[macro_export]
macro_rules! check_bounds_debug {
    ($range:expr, $($node:expr),+ $(,)?) => {
        $(
            debug_assert!(
                $range.contains(&($node.as_usize())),
                "NodeId {:?} is out of bounds for range {:?}",
                $node,
                $range
            );
        )+
    };
}

// ── Direction types ───────────────────────────────

#[derive(Clone, Copy)]
pub struct Directed;

#[derive(Clone, Copy)]
pub struct Undirected;

impl Direction for Directed {
    const IS_DIRECTED: bool = true;
}

impl Direction for Undirected {
    const IS_DIRECTED: bool = false;
}

// ── Vec<Neighbor<N, W>> container ─────────────────

impl<N: NodeId, W> NeighborContainer for Vec<Neighbor<N, W>> {
    type NodeType = N;
    type WeightType = W;

    const SUPPORTS_MULTIEDGES: bool = true;

    fn empty() -> Self {
        Vec::new()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert(&mut self, node: N, weight: W) -> bool {
        self.push(Neighbor { node, weight });
        true
    }

    fn iter_neighbors(&self) -> impl Iterator<Item = NeighborRef<'_, N, W>> {
        self.iter().map(|n| NeighborRef {
            node: &n.node,
            weight: &n.weight,
        })
    }

    fn iter_neighbors_mut(&mut self) -> impl Iterator<Item = NeighborRefMut<'_, N, W>> {
        self.iter_mut().map(|n| NeighborRefMut {
            node: &n.node,
            weight: &mut n.weight,
        })
    }

    fn into_iter_neighbors(self) -> impl Iterator<Item = Neighbor<N, W>> {
        self.into_iter()
    }

    fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(NeighborRef<N, W>) -> bool,
    {
        self.retain(|n| {
            f(NeighborRef {
                node: &n.node,
                weight: &n.weight,
            })
        });
    }
}

// ── HashMap<N, W> container ───────────────────────

impl<N: NodeId + Eq + Hash, W> NeighborContainer for HashMap<N, W, FixedState> {
    type NodeType = N;
    type WeightType = W;

    const SUPPORTS_MULTIEDGES: bool = false;

    fn empty() -> Self {
        HashMap::with_hasher(FixedState::default())
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert(&mut self, node: N, weight: W) -> bool {
        use hashbrown::hash_map::Entry;
        match self.entry(node) {
            Entry::Occupied(mut e) => {
                e.insert(weight);
                false
            }
            Entry::Vacant(e) => {
                e.insert(weight);
                true
            }
        }
    }

    fn iter_neighbors(&self) -> impl Iterator<Item = NeighborRef<'_, N, W>> {
        self.iter()
            .map(|(node, weight)| NeighborRef { node, weight })
    }

    fn iter_neighbors_mut(&mut self) -> impl Iterator<Item = NeighborRefMut<'_, N, W>> {
        self.iter_mut()
            .map(|(node, weight)| NeighborRefMut { node, weight })
    }

    fn into_iter_neighbors(self) -> impl Iterator<Item = Neighbor<N, W>> {
        self.into_iter()
            .map(|(node, weight)| Neighbor { node, weight })
    }

    fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(NeighborRef<N, W>) -> bool,
    {
        self.retain(|node, weight| f(NeighborRef { node, weight }));
    }
}

// ── Vec<IncNeighbor<N, W, E>> container ───────────

impl<N: NodeId, W, E: EdgeId> IncNeighborContainer for Vec<IncNeighbor<N, W, E>> {
    type NodeType = N;
    type WeightType = W;
    type EdgeType = E;

    const SUPPORTS_MULTIEDGES: bool = true;

    fn empty() -> Self {
        Vec::new()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert_inc(&mut self, node: N, weight: W, edge: E) -> bool {
        self.push(IncNeighbor { node, weight, edge });
        true
    }

    fn iter_neighbors_inc(&self) -> impl Iterator<Item = IncNeighborRef<'_, N, W, E>> {
        self.iter().map(|n| IncNeighborRef {
            node: &n.node,
            weight: &n.weight,
            edge: &n.edge,
        })
    }

    fn iter_neighbors_inc_mut(&mut self) -> impl Iterator<Item = IncNeighborRefMut<'_, N, W, E>> {
        self.iter_mut().map(|n| IncNeighborRefMut {
            node: &n.node,
            weight: &mut n.weight,
            edge: &n.edge,
        })
    }

    fn into_iter_neighbors_inc(self) -> impl Iterator<Item = IncNeighbor<N, W, E>> {
        self.into_iter()
    }

    fn retain_inc<F>(&mut self, mut f: F)
    where
        F: FnMut(IncNeighborRef<N, W, E>) -> bool,
    {
        self.retain(|n| {
            f(IncNeighborRef {
                node: &n.node,
                weight: &n.weight,
                edge: &n.edge,
            })
        });
    }
}

// ── HashMap<N, (W, E)> container ──────────────────

impl<N: NodeId + Eq + Hash, W, E: EdgeId> IncNeighborContainer for HashMap<N, (W, E), FixedState> {
    type NodeType = N;
    type WeightType = W;
    type EdgeType = E;

    const SUPPORTS_MULTIEDGES: bool = false;

    fn empty() -> Self {
        HashMap::with_hasher(FixedState::default())
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert_inc(&mut self, node: N, weight: W, edge: E) -> bool {
        use hashbrown::hash_map::Entry;
        match self.entry(node) {
            Entry::Occupied(mut e) => {
                e.insert((weight, edge));
                false
            }
            Entry::Vacant(e) => {
                e.insert((weight, edge));
                true
            }
        }
    }

    fn iter_neighbors_inc(&self) -> impl Iterator<Item = IncNeighborRef<'_, N, W, E>> {
        self.iter()
            .map(|(node, (weight, edge))| IncNeighborRef { node, weight, edge })
    }

    fn iter_neighbors_inc_mut(&mut self) -> impl Iterator<Item = IncNeighborRefMut<'_, N, W, E>> {
        self.iter_mut()
            .map(|(node, (weight, edge))| IncNeighborRefMut { node, weight, edge })
    }

    fn into_iter_neighbors_inc(self) -> impl Iterator<Item = IncNeighbor<N, W, E>> {
        self.into_iter()
            .map(|(node, (weight, edge))| IncNeighbor { node, weight, edge })
    }

    fn retain_inc<F>(&mut self, mut f: F)
    where
        F: FnMut(IncNeighborRef<N, W, E>) -> bool,
    {
        self.retain(|node, (weight, edge)| f(IncNeighborRef { node, weight, edge }));
    }
}

// ── Blanket AdjConfig ─────────────────────────────

impl<N, W, D, C> AdjConfig for (N, W, D, C)
where
    N: NodeId,
    D: Direction,
    C: NeighborContainer<WeightType = W, NodeType = N>,
{
    type Weight = W;
    type Dir = D;
    type NodeId = N;
    type Container = C;
}

// ── Blanket IncConfig ─────────────────────────────

impl<N, W, D, E, C> IncConfig for (N, W, D, E, C)
where
    N: NodeId,
    D: Direction,
    E: EdgeId,
    C: IncNeighborContainer<WeightType = W, NodeType = N, EdgeType = E>,
{
    type Weight = W;
    type Dir = D;
    type NodeId = N;
    type EdgeId = E;
    type Container = C;
}
