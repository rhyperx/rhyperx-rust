use std::cmp::max;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut, RangeBounds};

use foldhash::fast::FixedState;
use hashbrown::{HashMap, HashSet, hash_map::Entry};

use super::traits::{AdjConfig, Direction, Neighbor, NeighborContainer};
use crate::check_bounds_debug;
use crate::types::NodeId;

#[derive(Clone)]
#[cfg_attr(
    feature = "serialize",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)
)]
pub struct AdjBase<C: AdjConfig> {
    adj: Vec<C::Container>,
    n: usize,
    m: usize,
    _phantom: PhantomData<C>,
}

impl<C: AdjConfig> AdjBase<C> {
    pub fn new() -> Self {
        Self {
            adj: Vec::new(),
            n: 0,
            m: 0,
            _phantom: PhantomData,
        }
    }

    pub fn with_nodes(n: usize) -> Self {
        Self {
            adj: (0..n).map(|_| C::Container::empty()).collect(),
            n,
            m: 0,
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

    pub fn insert_edge(&mut self, from: C::NodeId, to: C::NodeId, weight: C::Weight)
    where
        C::Weight: Clone,
    {
        check_bounds_debug!(0..self.adj.len(), from, to);

        let inserted = if C::Dir::IS_DIRECTED {
            self.adj[from.as_usize()].insert(to, weight)
        } else {
            let inserted = self.adj[from.as_usize()].insert(to, weight.clone());
            self.adj[to.as_usize()].insert(from, weight);
            inserted
        };
        if inserted {
            self.m += 1;
        }
    }

    pub fn remove_edges_between(&mut self, from: C::NodeId, to: C::NodeId) -> usize {
        check_bounds_debug!(0..self.adj.len(), from, to);

        let removed = if C::Dir::IS_DIRECTED {
            let len_before = self.adj[from.as_usize()].len();
            self.adj[from.as_usize()].retain(|n| *n.node != to);
            len_before - self.adj[from.as_usize()].len()
        } else {
            let len_before = (
                self.adj[from.as_usize()].len(),
                self.adj[to.as_usize()].len(),
            );
            self.adj[from.as_usize()].retain(|n| *n.node != to);
            self.adj[to.as_usize()].retain(|n| *n.node != from);
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
            neighbors.retain(|n| n.node.as_usize() != x);
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
            for n in neighbors.iter_neighbors() {
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
            for n in neighbors.iter_neighbors() {
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
            neighbors.retain(|n| ids.insert(*n.node));
            count += len_before - neighbors.len();
        }
        if !C::Dir::IS_DIRECTED {
            count /= 2;
        }

        count
    }
}

impl<C: AdjConfig> Debug for AdjBase<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "n: {}, m: {}", self.n, self.m)?;
        for (u, neighbors) in self.iter_neighbors().enumerate() {
            let ids: Vec<C::NodeId> = neighbors.iter_neighbors().map(|n| *n.node).collect();
            let _ = writeln!(f, "{}:\t{:?}", u, ids);
        }
        Ok(())
    }
}

impl<C: AdjConfig> Index<C::NodeId> for AdjBase<C> {
    type Output = C::Container;

    fn index(&self, index: C::NodeId) -> &Self::Output {
        &self.adj[index.as_usize()]
    }
}

impl<C: AdjConfig> IndexMut<C::NodeId> for AdjBase<C> {
    fn index_mut(&mut self, index: C::NodeId) -> &mut Self::Output {
        &mut self.adj[index.as_usize()]
    }
}

/// Vec-backed adj list (no edge IDs).
pub type AdjList<N, W, D> = AdjBase<(N, W, D, Vec<Neighbor<N, W>>)>;

/// HashMap-backed adj set (no edge IDs).
pub type AdjSet<N, W, D> = AdjBase<(N, W, D, HashMap<N, W, FixedState>)>;

impl<N: NodeId + Debug, W: Clone, D: Direction> From<AdjSet<N, W, D>> for AdjList<N, W, D> {
    fn from(value: AdjSet<N, W, D>) -> Self {
        let mut rv = Self::with_nodes(value.n());
        for (u, container) in value.adj.into_iter().enumerate() {
            for n in container.into_iter_neighbors() {
                rv.insert_edge(N::from_usize(u), n.node, n.weight);
            }
        }
        rv
    }
}

impl<N: NodeId + Debug, W: Clone, D: Direction> From<AdjList<N, W, D>> for AdjSet<N, W, D> {
    fn from(value: AdjList<N, W, D>) -> Self {
        let mut rv = Self::with_nodes(value.n());
        for (u, container) in value.adj.into_iter().enumerate() {
            for n in container.into_iter_neighbors() {
                rv.insert_edge(N::from_usize(u), n.node, n.weight);
            }
        }
        rv
    }
}

impl<N: NodeId, W, D: Direction> AdjList<N, W, D> {
    pub fn sort_neighbors(&mut self) {
        for u in 0..self.n {
            self.adj[u].sort_unstable();
        }
    }
}
