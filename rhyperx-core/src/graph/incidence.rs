use std::cmp::max;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut, RangeBounds};

use foldhash::fast::FixedState;
use hashbrown::{HashMap, HashSet, hash_map::Entry};

use super::traits::{Direction, IncConfig, IncNeighbor, IncNeighborContainer};
use crate::check_bounds_debug;
use crate::types::{EdgeId, NodeId};

#[derive(Clone)]
#[cfg_attr(
    feature = "serialize",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)
)]
pub struct IncBase<C: IncConfig> {
    adj: Vec<C::Container>,
    n: usize,
    m: usize,
    next_edge_id: C::EdgeId,
    _phantom: PhantomData<C>,
}

impl<C: IncConfig> IncBase<C> {
    pub fn new() -> Self {
        Self {
            adj: Vec::new(),
            n: 0,
            m: 0,
            next_edge_id: C::EdgeId::zero(),
            _phantom: PhantomData,
        }
    }

    pub fn with_nodes(n: usize) -> Self {
        Self {
            adj: (0..n).map(|_| C::Container::empty()).collect(),
            n,
            m: 0,
            next_edge_id: C::EdgeId::zero(),
            _phantom: PhantomData,
        }
    }

    pub fn from_edges_mapped(
        edges: Vec<(C::NodeId, C::NodeId, C::Weight)>,
    ) -> (
        Self,
        Vec<C::NodeId>,
        HashMap<C::NodeId, C::NodeId, FixedState>,
    )
    where
        C::Weight: Clone,
    {
        if edges.is_empty() {
            return (
                Self::new(),
                Vec::new(),
                HashMap::with_hasher(FixedState::default()),
            );
        }

        let mut compressed_index: HashMap<C::NodeId, C::NodeId, FixedState> =
            HashMap::with_hasher(FixedState::default());
        let mut curr_number = 0usize;

        for (u, v, _w) in edges.iter() {
            if let Entry::Vacant(e) = compressed_index.entry(*u) {
                e.insert(C::NodeId::from_usize(curr_number));
                curr_number += 1;
            }
            if let Entry::Vacant(e) = compressed_index.entry(*v) {
                e.insert(C::NodeId::from_usize(curr_number));
                curr_number += 1;
            }
        }

        let n = curr_number;
        let mut rv = Self::with_nodes(n);

        for (u, v, w) in edges.into_iter() {
            let u_idx = compressed_index[&u];
            let v_idx = compressed_index[&v];
            rv.insert_edge(u_idx, v_idx, w);
        }

        let mut original_index = Vec::with_capacity(n);
        original_index.resize(n, C::NodeId::zero());
        for (node, &compressed) in compressed_index.iter() {
            original_index[compressed.as_usize()] = *node;
        }

        (rv, original_index, compressed_index)
    }

    pub fn from_edges_unmapped(edges: Vec<(C::NodeId, C::NodeId, C::Weight)>) -> Self
    where
        C::Weight: Clone,
    {
        if edges.is_empty() {
            return Self::new();
        }

        let n = edges.iter().fold(0usize, |acc, (u, v, _w)| {
            max(acc, max(u.as_usize(), v.as_usize()))
        }) + 1;

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

    pub fn insert_edge(&mut self, from: C::NodeId, to: C::NodeId, weight: C::Weight) -> C::EdgeId
    where
        C::Weight: Clone,
    {
        check_bounds_debug!(0..self.adj.len(), from, to);

        let edge = self.next_edge_id;
        let inserted = if C::Dir::IS_DIRECTED {
            self.adj[from.as_usize()].insert_inc(to, weight, edge)
        } else {
            let inserted = self.adj[from.as_usize()].insert_inc(to, weight.clone(), edge);
            self.adj[to.as_usize()].insert_inc(from, weight, edge);
            inserted
        };
        if inserted {
            self.next_edge_id = C::EdgeId::from_usize(self.next_edge_id.as_usize() + 1);
            self.m += 1;
        }
        edge
    }

    pub fn remove_edges_between(&mut self, from: C::NodeId, to: C::NodeId) -> usize {
        check_bounds_debug!(0..self.adj.len(), from, to);

        let removed = if C::Dir::IS_DIRECTED {
            let len_before = self.adj[from.as_usize()].len();
            self.adj[from.as_usize()].retain_inc(|n| *n.node != to);
            len_before - self.adj[from.as_usize()].len()
        } else {
            let len_before = (
                self.adj[from.as_usize()].len(),
                self.adj[to.as_usize()].len(),
            );
            self.adj[from.as_usize()].retain_inc(|n| *n.node != to);
            self.adj[to.as_usize()].retain_inc(|n| *n.node != from);
            let delta1 = len_before.0 - self.adj[from.as_usize()].len();
            let delta2 = len_before.1 - self.adj[to.as_usize()].len();
            debug_assert!(delta1 == delta2);
            delta1
        };

        self.m -= removed;
        removed
    }

    pub fn remove_self_loops(&mut self) -> usize {
        let mut removed = 0;
        for (x, neighbors) in self.adj.iter_mut().enumerate() {
            let len_before = neighbors.len();
            neighbors.retain_inc(|n| n.node.as_usize() != x);
            removed += len_before - neighbors.len();
        }
        if !C::Dir::IS_DIRECTED {
            removed /= 2;
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
        let mut ids: HashSet<C::NodeId, FixedState> = HashSet::with_hasher(FixedState::default());

        for (_x, neighbors) in self.adj.iter().enumerate() {
            ids.clear();
            for n in neighbors.iter_neighbors_inc() {
                ids.insert(*n.node);
            }
            count += neighbors.len() - ids.len();
        }
        if !C::Dir::IS_DIRECTED {
            count /= 2;
        }

        count
    }

    pub fn has_multiedges(&self) -> bool {
        if !C::Container::SUPPORTS_MULTIEDGES {
            return false;
        }

        let mut ids: HashSet<C::NodeId, FixedState> = HashSet::with_hasher(FixedState::default());

        for (_x, neighbors) in self.adj.iter().enumerate() {
            ids.clear();
            for n in neighbors.iter_neighbors_inc() {
                ids.insert(*n.node);
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
        let mut ids: HashSet<C::NodeId, FixedState> = HashSet::with_hasher(FixedState::default());

        for (_x, neighbors) in self.adj.iter_mut().enumerate() {
            ids.clear();
            let len_before = neighbors.len();
            neighbors.retain_inc(|n| ids.insert(*n.node));
            count += len_before - neighbors.len();
        }
        if !C::Dir::IS_DIRECTED {
            count /= 2;
        }

        count
    }
}

impl<C: IncConfig> Debug for IncBase<C>
where
    C::NodeId: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "n: {}, m: {}", self.n, self.m)?;
        for (u, neighbors) in self.iter_neighbors().enumerate() {
            let ids: Vec<C::NodeId> = neighbors.iter_neighbors_inc().map(|n| *n.node).collect();
            let _ = writeln!(f, "{}:\t{:?}", u, ids);
        }
        Ok(())
    }
}

impl<C: IncConfig> Index<C::NodeId> for IncBase<C> {
    type Output = C::Container;

    fn index(&self, index: C::NodeId) -> &Self::Output {
        &self.adj[index.as_usize()]
    }
}

impl<C: IncConfig> IndexMut<C::NodeId> for IncBase<C> {
    fn index_mut(&mut self, index: C::NodeId) -> &mut Self::Output {
        &mut self.adj[index.as_usize()]
    }
}

/// Vec-backed inc list (with edge IDs).
pub type IncList<N, W, D, E> = IncBase<(N, W, D, E, Vec<IncNeighbor<N, W, E>>)>;

/// HashMap-backed inc set (with edge IDs).
pub type IncSet<N, W, D, E> = IncBase<(N, W, D, E, HashMap<N, (W, E), FixedState>)>;

impl<N: NodeId + Debug, W: Clone, D: Direction, E: EdgeId> From<IncSet<N, W, D, E>>
    for IncList<N, W, D, E>
{
    fn from(value: IncSet<N, W, D, E>) -> Self {
        let mut rv = Self::with_nodes(value.n());
        for (u, container) in value.adj.into_iter().enumerate() {
            for n in container.into_iter_neighbors_inc() {
                rv.insert_edge(N::from_usize(u), n.node, n.weight);
            }
        }
        rv
    }
}

impl<N: NodeId + Debug, W: Clone, D: Direction, E: EdgeId> From<IncList<N, W, D, E>>
    for IncSet<N, W, D, E>
{
    fn from(value: IncList<N, W, D, E>) -> Self {
        let mut rv = Self::with_nodes(value.n());
        for (u, container) in value.adj.into_iter().enumerate() {
            for n in container.into_iter_neighbors_inc() {
                rv.insert_edge(N::from_usize(u), n.node, n.weight);
            }
        }
        rv
    }
}

impl<N: NodeId, W, D: Direction, E: EdgeId> IncList<N, W, D, E> {
    pub fn sort_neighbors(&mut self) {
        for u in 0..self.n {
            self.adj[u].sort_unstable();
        }
    }
}
