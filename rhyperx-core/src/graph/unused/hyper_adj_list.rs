use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Index, IndexMut, Sub},
    slice::SliceIndex,
};

use foldhash::fast::FixedState;
use hashbrown::HashMap;
use num_traits::{AsPrimitive, PrimInt};
use polars::prelude::first;
use seq_macro::seq;

use crate::graph::{Hx, NodeId, StaticEdgeSet, StaticEdgeSetAccessor};

pub struct HyperAdjList<T, W>
where
    T: Clone + Hash + Eq + AsPrimitive<usize> + PrimInt,
{
    pub adj: Vec<StaticEdgeSet<T, W>>,
    pub n: usize,
    pub m: usize,
}

impl<T, W> HyperAdjList<T, W>
where
    T: Clone + Hash + Eq + AsPrimitive<usize> + PrimInt,
{
    pub fn new() -> Self {
        Self {
            adj: Vec::new(),
            n: 0,
            m: 0,
        }
    }

    pub fn extend_with_edges<const N: usize>(&mut self, edges: Vec<Hx<N, T, W>>) -> usize
    where
        T: Clone + Hash + Eq + Display,
        W: Clone,
        StaticEdgeSet<T, W>: StaticEdgeSetAccessor<N, T, W>,
    {
        let mut count = 0;
        for edge in edges {
            count += self.add_edge(edge) as usize;
        }
        count
        // StaticEdgeSet<T, W>: StaticEdgeSetAccessor<N, T, W>,
        // Vec<StaticEdgeSet<T, W>>: IndexMut<T, Output = StaticEdgeSet<T, W>>,
    }

    pub fn from_edges_mapped(
        edges: Vec<(Vec<T>, W)>,
    ) -> (HyperAdjList<T, W>, Vec<T>, HashMap<T, T, FixedState>)
    where
        T: Ord + Sub + Display + Debug,
        W: Clone,
    {
        if edges.is_empty() {
            return (
                Self::new(),
                Vec::new(),
                HashMap::with_hasher(FixedState::default()),
            );
        }

        let first_element = edges[0].0.first().unwrap();
        let (mut min, mut max) = (first_element, first_element);

        for (edge, _) in edges.iter() {
            let first = edges[0].0.first().unwrap();

            let (curr_min, curr_max) = edge
                .iter()
                .fold((first, first), |(min, max), e| (min.min(e), max.max(e)));
            min = min.min(curr_min);
            max = max.max(curr_max);
        }

        let n = max.as_() - min.as_() + 1;

        let mut original_to_adj = HashMap::with_hasher(FixedState::default());
        let mut adj_to_original = vec![T::zero(); n];

        seq!(N in 2..11 {
            let mut bucket_~N: Vec<Hx<N, T, W>> = Vec::new();
        });

        let mut idx: T = T::zero();
        for (edge, weight) in edges.into_iter() {
            for &node in &edge {
                if !original_to_adj.contains_key(&node) {
                    original_to_adj.insert(node, idx);
                    adj_to_original[idx.as_()] = node;
                    idx = idx + T::one();
                }
            }

            let mapped_edge: Vec<T> = edge
                .into_iter()
                .map(|node| original_to_adj[&node])
                .collect();
            seq!(N in 2..11 {
                match mapped_edge.len() {
                    #(N => {
                        let mut array = [T::zero(); N];
                        array.copy_from_slice(mapped_edge.as_slice());
                        bucket_~N.push(Hx::new(array, weight).expect("Clique found with duplicate node"))
                    },)*
                    _ => (),
                }
            });
        }

        let mut hg = Self::new();
        hg.resize(n);

        seq!(N in 2..11 {
            hg.extend_with_edges(bucket_~N);
        });

        (hg, adj_to_original, original_to_adj)
    }

    pub fn from_edges_unmapped(edges: Vec<(Vec<T>, W)>) -> Self
    where
        T: Ord + Sub + Display + Debug,
        W: Clone,
    {
        if edges.is_empty() {
            return Self::new();
        }

        // Find the maximum node ID to determine array size
        let first_element = edges[0].0.first().unwrap();
        let mut max = first_element;

        for (edge, _) in edges.iter() {
            let curr_max = edge.iter().max().unwrap_or(first_element);
            max = max.max(curr_max);
        }

        let n = max.as_() + 1;

        seq!(N in 2..11 {
            let mut bucket_~N: Vec<Hx<N, T, W>> = Vec::new();
        });

        for (edge, weight) in edges.into_iter() {
            seq!(N in 2..11 {
                match edge.len() {
                    #(N => {
                        let mut array = [T::zero(); N];
                        for (i, val) in edge.iter().enumerate() {
                            array[i] = val.clone();
                        }
                        bucket_~N.push(Hx::new(array, weight).expect("Clique found with duplicate node"))
                    },)*
                    _ => (),
                }
            });
        }

        let mut hg = Self::with_nodes(n);

        seq!(N in 2..11 {
            hg.extend_with_edges(bucket_~N);
        });

        hg
    }

    pub fn resize(&mut self, new_n: usize) {
        if new_n > self.n {
            self.adj.resize_with(new_n, StaticEdgeSet::new);
            self.n = new_n;
        }
    }

    pub fn add_edge<const N: usize>(&mut self, edge: Hx<N, T, W>) -> bool
    where
        T: Clone + Hash + Eq,
        W: Clone,
        StaticEdgeSet<T, W>: StaticEdgeSetAccessor<N, T, W>,
    {
        // skips insertion entirely, since it assumes that an edge will exist as neighbor of ANY of
        // its nodes
        let node = edge.nodes[0].clone();
        let bucket = self[node].get_bucket_mut::<N>();
        if bucket.contains(&edge) {
            return false;
        }
        for node in edge.nodes.iter() {
            let bucket = self[node.clone()].get_bucket_mut::<N>();
            bucket.insert(edge.clone());
        }
        true
    }

    pub fn with_nodes(n: usize) -> Self {
        let mut adj = Vec::new();
        adj.resize_with(n, StaticEdgeSet::new);
        Self { adj, n, m: 0 }
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn m(&self) -> usize {
        self.m
    }

    pub fn neighbors<I>(&self, node: I) -> Vec<(Vec<T>, W)>
    where
        Self: Index<I, Output = StaticEdgeSet<T, W>>,
        I: Clone,
        T: Clone,
        W: Clone,
    {
        let mut rv = Vec::new();
        seq!(N in 2..11 {
            for edge in self[node.clone()].get_bucket::<N>().iter().cloned(){
                rv.push((edge.nodes.into(), edge.weight));
            }
        });
        rv
    }

    pub fn neighbors_unweighted<I>(&self, node: I) -> Vec<Vec<T>>
    where
        Self: Index<I, Output = StaticEdgeSet<T, W>>,
        I: Clone,
        T: Clone,
    {
        let mut rv = Vec::new();
        seq!(N in 2..11 {
            for edge in self[node.clone()].get_bucket::<N>().iter(){
                rv.push(edge.nodes.into());
            }
        });
        rv
    }
}

impl<I, T, W> Index<I> for HyperAdjList<T, W>
where
    I: AsPrimitive<usize>,
    T: Clone + Hash + Eq + AsPrimitive<usize> + PrimInt,
{
    type Output = StaticEdgeSet<T, W>;

    fn index(&self, index: I) -> &Self::Output {
        &self.adj[index.as_()]
    }
}

impl<I, T, W> IndexMut<I> for HyperAdjList<T, W>
where
    I: AsPrimitive<usize>,
    T: Clone + Hash + Eq + AsPrimitive<usize> + PrimInt,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.adj[index.as_()]
    }
}

impl<T, W> Default for HyperAdjList<T, W>
where
    T: Clone + Hash + Eq + AsPrimitive<usize> + PrimInt,
    Vec<StaticEdgeSet<T, W>>: IndexMut<T, Output = StaticEdgeSet<T, W>>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, W> HyperAdjList<T, W>
where
    T: Clone + Hash + Eq + AsPrimitive<usize> + PrimInt,
{
    /// Returns an iterator over the underlying adjacency list sets.
    pub fn iter(&self) -> std::slice::Iter<'_, StaticEdgeSet<T, W>> {
        self.adj.iter()
    }

    /// Returns a mutable iterator over the underlying adjacency list sets.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, StaticEdgeSet<T, W>> {
        self.adj.iter_mut()
    }
}

impl<'a, T, W> IntoIterator for &'a HyperAdjList<T, W>
where
    T: Clone + Hash + Eq + AsPrimitive<usize> + PrimInt,
{
    type Item = &'a StaticEdgeSet<T, W>;
    type IntoIter = std::slice::Iter<'a, StaticEdgeSet<T, W>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, W> IntoIterator for &'a mut HyperAdjList<T, W>
where
    T: Clone + Hash + Eq + AsPrimitive<usize> + PrimInt,
{
    type Item = &'a mut StaticEdgeSet<T, W>;
    type IntoIter = std::slice::IterMut<'a, StaticEdgeSet<T, W>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, W> IntoIterator for HyperAdjList<T, W>
where
    T: Clone + Hash + Eq + AsPrimitive<usize> + PrimInt,
{
    type Item = StaticEdgeSet<T, W>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.adj.into_iter()
    }
}
