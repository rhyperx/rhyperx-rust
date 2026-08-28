use std::cmp::max;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut, RangeBounds};

use foldhash::fast::FixedState;
use hashbrown::{HashMap, HashSet, hash_map::Entry};

use super::common::NodeId;
use super::traits::{
    AdjConfig, Direction, DirectionalInsert, EdgeIdTrait, Incidence, NeighborContainer,
};
use crate::check_bounds_debug;
use crate::graph::adj_list::Neighbor;

#[derive(Clone)]
pub struct AdjListBase<C: AdjConfig> {
    adj: Vec<C::Container>,
    n: usize,
    m: usize,
    next_edge_id: <C::Inc as Incidence>::EdgeType,
    _phantom: PhantomData<C>,
}

impl<C: AdjConfig> AdjListBase<C> {
    pub fn new() -> Self {
        Self {
            adj: Vec::new(),
            n: 0,
            m: 0,
            next_edge_id: <C::Inc as Incidence>::EdgeType::ZERO,
            _phantom: PhantomData,
        }
    }

    pub fn with_nodes(n: usize) -> Self {
        Self {
            adj: (0..n).map(|_| C::Container::empty()).collect(),
            n,
            m: 0,
            next_edge_id: <C::Inc as Incidence>::EdgeType::ZERO,
            _phantom: PhantomData,
        }
    }

    pub fn from_edges_mapped(
        edges: Vec<(NodeId, NodeId, C::Weight)>,
    ) -> (Self, Vec<NodeId>, HashMap<NodeId, NodeId, FixedState>)
    where
        C::Dir: DirectionalInsert<C::Container>,
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

        for (u, v, w) in edges.into_iter() {
            let u_idx = compressed_index[&u];
            let v_idx = compressed_index[&v];
            rv.insert_edge(u_idx, v_idx, w);
        }

        let mut original_index = Vec::with_capacity(n);
        original_index.resize(n, 0);
        for (node, &compressed) in compressed_index.iter() {
            original_index[compressed as usize] = *node;
        }

        (rv, original_index, compressed_index)
    }

    pub fn from_edges_unmapped(edges: Vec<(NodeId, NodeId, C::Weight)>) -> Self
    where
        C::Dir: DirectionalInsert<C::Container>,
    {
        if edges.is_empty() {
            return Self::new();
        }

        let n = (edges
            .iter()
            .fold(0, |acc, (u, v, _w)| max(acc, max(*u, *v)))
            + 1) as usize;

        let mut rv = Self::with_nodes(n);
        rv.m = edges.len();

        for (u, v, w) in edges.into_iter() {
            rv.insert_edge(u, v, w);
        }

        rv
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn m(&self) -> usize {
        self.m
    }

    pub fn insert_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        weight: C::Weight,
    ) -> <C::Inc as Incidence>::EdgeType
    where
        C::Dir: DirectionalInsert<C::Container>,
    {
        check_bounds_debug!(0..self.adj.len(), from, to);

        let edge = self.next_edge_id;
        let inserted = <C::Dir as DirectionalInsert<C::Container>>::insert_edge(
            &mut self.adj,
            from,
            to,
            weight,
            edge,
        );
        if inserted {
            self.next_edge_id.advance();
            self.m += 1;
        }
        edge
    }

    pub fn remove_edges_between(&mut self, from: NodeId, to: NodeId) -> usize {
        check_bounds_debug!(0..self.adj.len(), from, to);

        let mut removed = if C::Dir::IS_DIRECTED {
            let len_before = self[from].len();
            self[from].retain(|n| *n.node != to);
            len_before - self[from].len()
        } else {
            let len_before = (self[from].len(), self[to].len());
            self[from].retain(|n| *n.node != to);
            self[to].retain(|n| *n.node != from);
            let delta1 = len_before.0 - self[from].len();
            let delta2 = len_before.1 - self[to].len();
            debug_assert!(delta1 == delta2);
            delta1
        };

        if !C::Container::SUPPORTS_MULTIEDGES && (from == to) {
            removed = 1;
        } else if C::Container::SUPPORTS_MULTIEDGES && !C::Dir::IS_DIRECTED && (from == to) {
            removed /= 2;
        }

        self.m -= removed;

        removed
    }

    pub fn remove_self_loops(&mut self) -> usize {
        let mut removed = 0;
        for (x, neighbors) in self.adj.iter_mut().enumerate() {
            let len_before = neighbors.len();
            neighbors.retain(|e| *e.node != x as NodeId);
            removed += len_before - neighbors.len();
        }
        if !C::Dir::IS_DIRECTED && C::Container::SUPPORTS_MULTIEDGES {
            removed /= 2
        }
        self.m -= removed;
        removed
    }

    pub fn iter_neighbors(&self) -> impl Iterator<Item = &C::Container> {
        self.adj.iter()
    }

    pub fn iter_neighbors_mut(&mut self) -> impl Iterator<Item = &mut C::Container> {
        self.adj.iter_mut()
    }

    pub fn drain_neighbors(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> impl Iterator<Item = C::Container> {
        self.adj.drain(range)
    }

    pub fn into_iter_neighbors(self) -> impl Iterator<Item = C::Container> {
        self.adj.into_iter()
    }

    pub fn count_multiedges(&self) -> usize {
        if !C::Container::SUPPORTS_MULTIEDGES {
            return 0;
        }

        let mut count = 0;
        let mut ids: HashSet<NodeId, FixedState> = HashSet::with_hasher(FixedState::default());

        for (_x, neighbors) in self.adj.iter().enumerate() {
            ids.clear();
            for neighbor in neighbors.iter_neighbors() {
                ids.insert(*neighbor.node);
            }
            count += neighbors.len() - ids.len();
        }
        if !C::Dir::IS_DIRECTED {
            count /= 2
        }

        count
    }

    pub fn has_multiedges(&self) -> bool {
        if !C::Container::SUPPORTS_MULTIEDGES {
            return false;
        }

        let mut ids: HashSet<NodeId, FixedState> = HashSet::with_hasher(FixedState::default());

        for (_x, neighbors) in self.adj.iter().enumerate() {
            ids.clear();
            for neighbor in neighbors.iter_neighbors() {
                ids.insert(*neighbor.node);
            }
            if neighbors.len() != ids.len() {
                return true;
            }
        }
        false
    }

    pub fn remove_multiedges(&mut self) -> usize {
        if !C::Container::SUPPORTS_MULTIEDGES {
            return 0;
        }

        let mut count = 0;
        let mut ids: HashSet<NodeId, FixedState> = HashSet::with_hasher(FixedState::default());

        for (_x, neighbors) in self.adj.iter_mut().enumerate() {
            ids.clear();
            let len_before = neighbors.len();
            neighbors.retain(|n| ids.insert(*n.node));
            count += len_before - neighbors.len();
        }
        if !C::Dir::IS_DIRECTED {
            count /= 2
        }

        count
    }
}

impl<C: AdjConfig> Debug for AdjListBase<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "n: {}, m: {}", self.n, self.m)?;
        for (u, neighbors) in self.iter_neighbors().enumerate() {
            let ids: Vec<NodeId> = neighbors.iter_neighbors().map(|v| *v.node).collect();
            writeln!(f, "{}:\t{:?}", u, ids);
        }
        Ok(())
    }
}

impl<C: AdjConfig> Index<usize> for AdjListBase<C> {
    type Output = C::Container;

    fn index(&self, index: usize) -> &Self::Output {
        &self.adj[index]
    }
}

impl<C: AdjConfig> Index<NodeId> for AdjListBase<C> {
    type Output = C::Container;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.adj[index as usize]
    }
}

impl<C: AdjConfig> IndexMut<usize> for AdjListBase<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.adj[index]
    }
}

impl<C: AdjConfig> IndexMut<NodeId> for AdjListBase<C> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.adj[index as usize]
    }
}

#[allow(type_alias_bounds)]
pub type AdjList<W, D: Direction, I: Incidence> =
    AdjListBase<(W, D, I, Vec<Neighbor<W, I::EdgeType>>)>;

#[allow(type_alias_bounds)]
pub type AdjSet<W, D: Direction, I: Incidence> =
    AdjListBase<(W, D, I, HashMap<NodeId, (W, I::EdgeType), FixedState>)>;

// Convertion traits from HashMap container to Vec and vice versa
impl<W, D, I> From<AdjSet<W, D, I>> for AdjList<W, D, I>
where
    I: Incidence,
    D: Direction + DirectionalInsert<Vec<Neighbor<W, I::EdgeType>>>,
{
    fn from(value: AdjSet<W, D, I>) -> Self {
        let mut rv = Self::with_nodes(value.n());

        for (u, neighbors) in value.into_iter_neighbors().enumerate() {
            for neighbor in neighbors.into_iter_neighbors() {
                rv.insert_edge(u as NodeId, neighbor.node, neighbor.weight);
            }
        }

        rv
    }
}

impl<W, D, I1, I2> From<AdjList<W, D, I1>> for AdjSet<W, D, I2>
where
    I1: Incidence,
    I2: Incidence,
    D: Direction + DirectionalInsert<HashMap<NodeId, (W, I2::EdgeType), FixedState>>,
{
    fn from(value: AdjList<W, D, I1>) -> Self {
        let mut rv = Self::with_nodes(value.n());

        for (u, neighbors) in value.into_iter_neighbors().enumerate() {
            for neighbor in neighbors.into_iter_neighbors() {
                rv.insert_edge(u as NodeId, neighbor.node, neighbor.weight);
            }
        }

        rv
    }
}

//List specific methods
impl<W, D: Direction, I: Incidence> AdjList<W, D, I> {
    pub fn sort_neighbors(&mut self) {
        let n = self.n();
        for u in 0..n {
            self[u].sort_unstable();
        }
    }
}
