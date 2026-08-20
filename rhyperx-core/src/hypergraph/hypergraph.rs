use std::collections::HashSet;

use hashbrown::HashMap;

use crate::{
    error::HypergraphError,
    hyperedge::{HxSizedRef, HxSizedRefMut, HxUnsizedRef, HxUnsizedRefMut, SizedHx, UnsizedHx},
    hyperedge_container::HyperedgeContainer,
    types::NodeId,
};

#[inline(always)]
fn find_dup_sorted<T: NodeId>(nodes: &[T]) -> Option<T> {
    nodes
        .windows(2)
        .find_map(|window| (window[0] == window[1]).then_some(&window[0]))
        .map(|dup| *dup)
}

#[derive(Clone)]
pub struct Hypergraph<T, W, C>
where
    T: NodeId,
    C: HyperedgeContainer<T, W>,
{
    pub(crate) edges: HashMap<usize, C>,
    pub(crate) nodes: HashMap<T, usize>,

    n: usize,
    m: usize,

    _phantom: std::marker::PhantomData<W>,
}

impl<T, W, C> Hypergraph<T, W, C>
where
    T: NodeId,
    C: HyperedgeContainer<T, W>,
{
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            nodes: HashMap::new(),
            n: 0,
            m: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn m(&self) -> usize {
        self.m
    }

    fn update_n(nodes: &mut HashMap<T, usize>, tot_n: &mut usize, edge: &[T], add: bool) {
        let inc = if add { 1 } else { -1 };

        for &n in edge {
            nodes
                .entry(n)
                .and_modify(|degree| {
                    *degree = (*degree as i64 + inc) as usize;
                })
                .or_insert_with(|| {
                    *tot_n += 1;
                    1
                });
        }
    }

    pub fn edges(&self, size: usize) -> Option<&C> {
        self.edges.get(&size)
    }

    pub fn edges_mut(&mut self, size: usize) -> Option<&mut C> {
        self.edges.get_mut(&size)
    }

    pub fn take_edges(&mut self, size: usize) -> Option<C> {
        self.edges.remove(&size)
    }

    // pub fn edges_sized<const N: usize>(&self) -> Option<&C> {
    //     self.edges.get(&N)
    // }
    // pub fn edges_mut_sized<const N: usize>(&mut self) -> Option<&mut C> {
    //     self.edges_mut(N)
    // }
    // pub fn take_edges_sized<const N: usize>(&mut self) -> Option<C> {
    //     self.take_edges(N)
    // }
    // pub fn remove_edge_sized_unchecked<const N: usize>(&mut self, edge: &[T; N]) -> Option<W> {
    //     self.remove_edge_unchecked(edge)
    // }
    // pub fn remove_edge_sized<const N: usize>(
    //     &mut self,
    //     edge: &[T; N],
    // ) -> Result<Option<W>, HypergraphError<T>> {
    //     self.remove_edge(edge)
    // }
    // pub fn remove_edge_unchecked(&mut self, nodes: &[T]) -> Option<W> {
    //     if self.edges.contains_key(&nodes.len()) {
    //         self.update_n(&nodes, false);
    //         self.m -= 1;
    //         let container = unsafe { self.edges.get_mut(&nodes.len()).unwrap_unchecked() };
    //         container.remove(nodes)
    //     } else {
    //         None
    //     }
    // }
    // pub fn has_hyperedge_unchecked(&self, hyperedge: &[T]) -> bool {
    //     match self.edges.get(&hyperedge.len()) {
    //         Some(container) => container.contains(hyperedge),
    //         None => false,
    //     }
    // }
    //
    // pub fn has_hyperedge_sized_unchecked<const N: usize>(&self, hyperedge: &[T; N]) -> bool {
    //     match self.edges.get(&N) {
    //         Some(container) => container.contains(hyperedge),
    //         None => false,
    //     }
    // }

    pub fn add_edge_slice(&mut self, nodes: &mut [T], weight: W) -> Result<bool, HypergraphError> {
        nodes.sort_unstable();
        if let Some(dup) = find_dup_sorted(nodes) {
            return Err(HypergraphError::DuplicateNodes(dup.as_usize()));
        }
        Ok(self.add_edge_slice_unchecked(nodes, weight))
    }

    pub fn add_edge_slice_unchecked(&mut self, nodes: &[T], weight: W) -> bool {
        Self::update_n(&mut self.nodes, &mut self.n, &nodes, true);
        self.m += 1;

        if !self.edges.contains_key(&nodes.len()) {
            self.edges.insert(nodes.len(), C::new(nodes.len()));
        }

        let container = unsafe { self.edges.get_mut(&nodes.len()).unwrap_unchecked() };
        container.insert(nodes, weight)
    }

    pub fn add_edge(&mut self, edge: UnsizedHx<T, W>) -> bool {
        self.add_edge_slice_unchecked(&edge.nodes, edge.weight)
    }

    pub fn add_edge_sized<const N: usize>(&mut self, edge: SizedHx<N, T, W>) -> bool {
        self.add_edge_slice_unchecked(&edge.nodes, edge.weight)
    }

    pub fn remove_edge<WW>(&mut self, edge: HxUnsizedRef<T, WW>) -> Option<W> {
        if self.edges.contains_key(&edge.nodes.len()) {
            Self::update_n(&mut self.nodes, &mut self.n, &edge.nodes, true);
            self.m -= 1;
            let container = unsafe { self.edges.get_mut(&edge.nodes.len()).unwrap_unchecked() };
            container.remove(edge.nodes)
        } else {
            None
        }
    }

    pub fn extend_with_edges<const N: usize>(&mut self, edges: Vec<UnsizedHx<T, W>>) -> usize {
        let mut count = 0;
        for edge in edges {
            count += self.add_edge(edge) as usize;
        }
        count
    }

    pub fn extend_with_edges_sized<const N: usize>(
        &mut self,
        edges: Vec<SizedHx<N, T, W>>,
    ) -> usize {
        let mut count = 0;
        for edge in edges {
            count += self.add_edge_sized(edge) as usize;
        }
        count
    }

    pub fn has_hyperedge<WW>(&self, edge: HxUnsizedRef<T, WW>) -> bool {
        match self.edges.get(&edge.nodes.len()) {
            Some(container) => container.contains(edge.nodes),
            None => false,
        }
    }

    pub fn get_hyperedge_sized<const N: usize, WW>(
        &self,
        hyperedge: HxUnsizedRef<T, WW>,
    ) -> Option<HxSizedRef<'_, N, T, W>> {
        match self.edges.get(&hyperedge.nodes.len()) {
            Some(container) => match container.get(hyperedge.nodes) {
                Some(hyperedge_ref) => Some(hyperedge_ref.into_sized_unchecked()),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_hyperedge_sized_mut<const N: usize, WW>(
        &mut self,
        hyperedge: HxUnsizedRef<'_, T, WW>,
    ) -> Option<HxSizedRefMut<'_, N, T, W>> {
        match self.edges.get_mut(&hyperedge.nodes.len()) {
            Some(container) => match container.get_mut(hyperedge.nodes) {
                Some(hyperedge_ref) => Some(hyperedge_ref.into_sized_unchecked()),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_hyperedge<WW>(
        &self,
        hyperedge: HxUnsizedRef<T, WW>,
    ) -> Option<HxUnsizedRef<'_, T, W>> {
        match self.edges.get(&hyperedge.nodes.len()) {
            Some(container) => match container.get(hyperedge.nodes) {
                Some(hyperedge_ref) => Some(hyperedge_ref),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_hyperedge_mut<WW>(
        &mut self,
        hyperedge: HxUnsizedRef<T, WW>,
    ) -> Option<HxUnsizedRefMut<'_, T, W>> {
        match self.edges.get_mut(&hyperedge.nodes.len()) {
            Some(container) => match container.get_mut(hyperedge.nodes) {
                Some(hyperedge_ref) => Some(hyperedge_ref),
                None => None,
            },
            None => None,
        }
    }

    pub fn modify_hx_with<WW, F>(&mut self, hyperedge: HxUnsizedRef<T, WW>, mut f: F) -> bool
    where
        F: FnMut(HxUnsizedRefMut<T, W>),
    {
        match self.get_hyperedge_mut(hyperedge) {
            Some(hg_ref) => {
                f(hg_ref);
                true
            }
            None => false,
        }
    }

    pub fn remove_hyperedge<WW>(&mut self, hyperedge: HxUnsizedRef<T, WW>) -> Option<W> {
        match self.edges.get_mut(&hyperedge.nodes.len()) {
            Some(container) => container.remove(hyperedge.nodes),
            None => None,
        }
    }

    pub fn take_hyperedge<WW>(
        &mut self,
        hyperedge: HxUnsizedRef<T, WW>,
    ) -> Option<UnsizedHx<T, W>> {
        match self.edges.get_mut(&hyperedge.nodes.len()) {
            Some(container) => match container.remove(hyperedge.nodes) {
                Some(w) => Some(UnsizedHx::new_unchecked(hyperedge.nodes.to_vec(), w)),
                None => None,
            },
            None => None,
        }
    }

    pub fn iter_hg_sizes(&self) -> impl Iterator<Item = usize> {
        let mut sorted_keys = self.edges.keys().copied().collect::<Vec<_>>();
        sorted_keys.sort_unstable();
        sorted_keys.into_iter()
    }

    pub fn iter_edges(&self, size: usize) -> impl Iterator<Item = HxUnsizedRef<'_, T, W>> {
        self.edges
            .get(&size)
            .into_iter()
            .flat_map(|container| container.iter())
    }

    pub fn iter_edges_sized<const N: usize>(
        &self,
        size: usize,
    ) -> impl Iterator<Item = HxSizedRef<'_, N, T, W>> {
        self.edges
            .get(&size)
            .into_iter()
            .flat_map(|container| container.iter())
            .map(|hx_ref| hx_ref.into_sized_unchecked())
    }

    pub fn remove_isolated_nodes(&mut self) -> usize {
        let len = self.nodes.len();
        self.nodes.retain(|_, &mut degree| degree > 0);
        self.n = self.nodes.len();
        len - self.nodes.len()
    }

    pub fn to_unweighted<CC>(self) -> Hypergraph<T, (), CC>
    where
        CC: HyperedgeContainer<T, ()>,
    {
        let mut new_edges = HashMap::new();
        for (order, container) in self.edges {
            let mut bucket = CC::new(order);
            for hx_ref in container.iter() {
                bucket.insert(hx_ref.nodes, ());
            }
            new_edges.insert(order, bucket);
        }

        Hypergraph {
            edges: new_edges,
            nodes: self.nodes,
            n: self.n,
            m: self.m,
            _phantom: std::marker::PhantomData,
        }
    }

    /// the vec is a map to new_node -> old node
    /// the hash map maps the old_node -> new_node
    pub fn normalize_node_ids(&mut self) -> (Vec<T>, HashMap<T, T>) {
        let mut new_to_old = vec![T::zero(); self.n];
        let mut old_to_new = HashMap::new();
        let mut next_id = 0;

        for (_, container) in self.edges.iter_mut() {
            for hx_ref in container.iter_mut() {
                for n in hx_ref.nodes.iter_mut() {
                    *n = match old_to_new.get(n) {
                        Some(&new_id) => new_id,
                        None => {
                            new_to_old[next_id] = *n;
                            old_to_new.insert(*n, T::from_usize(next_id));
                            next_id += 1;
                            T::from_usize(next_id - 1)
                        }
                    }
                }
                hx_ref.nodes.sort_unstable();
            }
        }

        (new_to_old, old_to_new)
    }

    pub fn retain_orders(&mut self, orders: &[usize]) -> usize {
        let mut count = 0;
        let orders_set: HashSet<usize> = orders.iter().cloned().collect();

        self.edges.retain(|&order, container| {
            if orders_set.contains(&order) {
                true
            } else {
                count += container.len();
                for edge_ref in container.iter() {
                    Self::update_n(&mut self.nodes, &mut self.n, &edge_ref.nodes, true);
                }
                false
            }
        });

        count
    }

    pub fn remove_orders(&mut self, orders: &[usize]) -> usize {
        let mut count = 0;

        for order in orders {
            match self.edges.get_mut(order) {
                Some(container) => {
                    count += container.len();
                    for edge_ref in container.iter() {
                        Self::update_n(&mut self.nodes, &mut self.n, &edge_ref.nodes, true);
                    }
                }
                None => {}
            }
        }

        count
    }
}
