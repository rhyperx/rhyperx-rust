use super::hyperedge::{HxUnsizedRef, HxUnsizedRefMut};
use crate::{collections::chunked_array_map::non_typed::ChunkedArrayMap, types::NodeId};

pub trait HyperedgeContainer<T, W>
where
    T: NodeId,
{
    fn new(chunk_size: usize) -> Self;

    fn insert(&mut self, nodes: &[T], weight: W) -> bool;

    fn remove(&mut self, nodes: &[T]) -> Option<W>;

    fn get(&self, nodes: &[T]) -> Option<HxUnsizedRef<'_, T, W>>;

    /// # Safety:
    /// It should be noted that modifying the nodes of a hyperedge can lead to internal state corruption
    fn get_mut(&mut self, nodes: &[T]) -> Option<HxUnsizedRefMut<'_, T, W>>;

    fn contains(&self, edge: &[T]) -> bool;

    fn len(&self) -> usize;

    fn iter<'a>(&'a self) -> impl Iterator<Item = HxUnsizedRef<'a, T, W>> + 'a
    where
        T: 'a,
        W: 'a;

    /// # Safety:
    /// It should be noted that modifying the nodes of a hyperedge can lead to internal state corruption
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = HxUnsizedRefMut<'a, T, W>> + 'a
    where
        T: 'a,
        W: 'a;

    /// Returns the nodes as a flat array;
    fn take_flat_nodes(&mut self) -> Vec<T>;

    fn take_weights(&mut self) -> Vec<W>;

    fn retain(&mut self, f: impl FnMut(&HxUnsizedRef<'_, T, W>) -> bool);
}

#[derive(Clone)]
#[cfg_attr(
    feature = "serialize",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)
)]
pub struct HxVecStore<T, W> {
    chunk_size: usize,
    nodes: Vec<T>,
    weights: Vec<W>,
}

#[derive(Clone)]
#[cfg_attr(
    feature = "serialize",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)
)]
pub struct HxSetStore<T, W> {
    container: ChunkedArrayMap<T, W>,
}

impl<T, W> HyperedgeContainer<T, W> for HxVecStore<T, W>
where
    T: NodeId,
    W: PartialEq,
{
    fn new(chunk_size: usize) -> Self {
        HxVecStore {
            chunk_size,
            nodes: Vec::new(),
            weights: Vec::new(),
        }
    }

    fn insert(&mut self, nodes: &[T], weight: W) -> bool {
        for n in nodes.iter().cloned() {
            self.nodes.push(n);
        }
        self.weights.push(weight);
        true
    }

    fn remove(&mut self, nodes: &[T]) -> Option<W> {
        for i in 0..self.weights.len() {
            let n = nodes.len();
            let start = i * n;
            let end = start + n;
            let edge_nodes = &self.nodes[start..end];
            if edge_nodes == nodes {
                for j in 0..n {
                    self.nodes[start + j] = self.nodes[self.nodes.len() - n + j];
                }
                self.nodes.truncate(self.nodes.len() - n);
                return Some(self.weights.swap_remove(i));
            }
        }
        None
    }

    fn get(&self, nodes: &[T]) -> Option<HxUnsizedRef<'_, T, W>> {
        for i in 0..self.weights.len() {
            let n = nodes.len();
            let start = i * n;
            let end = start + n;
            let edge_nodes = &self.nodes[start..end];
            if edge_nodes == nodes {
                let weight_ref = &self.weights[i];
                return Some(HxUnsizedRef::new(
                    edge_nodes.try_into().unwrap(),
                    weight_ref,
                ));
            }
        }
        None
    }

    fn get_mut(&mut self, nodes: &[T]) -> Option<HxUnsizedRefMut<'_, T, W>> {
        for i in 0..self.weights.len() {
            let n = nodes.len();
            let start = i * n;
            let end = start + n;
            let edge_nodes = &self.nodes[start..end];
            if edge_nodes == nodes {
                let nodes_ref = &mut self.nodes[start..end];
                let weight_ref = &mut self.weights[i];
                return Some(HxUnsizedRefMut::new(
                    nodes_ref.try_into().unwrap(),
                    weight_ref,
                ));
            }
        }
        None
    }

    fn contains(&self, nodes: &[T]) -> bool {
        for i in 0..self.weights.len() {
            let n = nodes.len();
            let start = i * n;
            let end = start + n;
            let edge_nodes = &self.nodes[start..end];
            if edge_nodes == nodes {
                return true;
            }
        }
        false
    }

    fn len(&self) -> usize {
        self.weights.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = HxUnsizedRef<'a, T, W>> + 'a
    where
        T: 'a,
        W: 'a,
    {
        self.nodes
            .chunks_exact(self.chunk_size)
            .zip(self.weights.iter())
            .map(|(nodes_slice, weight_ref)| {
                HxUnsizedRef::new(nodes_slice.try_into().unwrap(), weight_ref)
            })
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = HxUnsizedRefMut<'a, T, W>> + 'a
    where
        T: 'a,
        W: 'a,
    {
        self.nodes
            .chunks_exact_mut(self.chunk_size)
            .zip(self.weights.iter_mut())
            .map(|(nodes_slice, weight_ref)| {
                HxUnsizedRefMut::new(nodes_slice.try_into().unwrap(), weight_ref)
            })
    }

    fn retain(&mut self, mut f: impl FnMut(&HxUnsizedRef<'_, T, W>) -> bool) {
        let len = self.weights.len();
        let mut write_idx = 0;
        let n = self.chunk_size;

        for i in 0..len {
            let start = i * n;
            let end = start + n;
            let nodes_slice = &self.nodes[start..end];
            let weight_ref = &self.weights[i];
            let hx_ref = HxUnsizedRef::new(nodes_slice.try_into().unwrap(), weight_ref);

            if f(&hx_ref) {
                if write_idx != i {
                    let write_start = write_idx * n;
                    self.nodes.copy_within(start..end, write_start);
                    self.weights.swap(write_idx, i);
                }
                write_idx += 1;
            }
        }

        // Truncate to new length
        self.nodes.truncate(write_idx * n);
        self.weights.truncate(write_idx);
    }

    fn take_flat_nodes(&mut self) -> Vec<T> {
        std::mem::take(&mut self.nodes)
    }

    fn take_weights(&mut self) -> Vec<W> {
        std::mem::take(&mut self.weights)
    }
}

impl<T, W> HyperedgeContainer<T, W> for HxSetStore<T, W>
where
    T: NodeId,
    W: PartialEq,
{
    fn new(chunk_size: usize) -> Self {
        HxSetStore {
            container: ChunkedArrayMap::new(chunk_size),
        }
    }

    fn insert(&mut self, nodes: &[T], weight: W) -> bool {
        self.container.insert(nodes, weight).is_none()
    }

    fn remove(&mut self, nodes: &[T]) -> Option<W> {
        self.container.remove(nodes)
    }

    fn get(&self, nodes: &[T]) -> Option<HxUnsizedRef<'_, T, W>> {
        match self.container.get_key_value(nodes) {
            Some((nodes, w)) => Some(HxUnsizedRef::new(nodes, w)),
            None => None,
        }
    }

    fn get_mut(&mut self, nodes: &[T]) -> Option<HxUnsizedRefMut<'_, T, W>> {
        match self.container.get_key_value_mut(nodes) {
            Some((nodes, w)) => Some(HxUnsizedRefMut::new(nodes, w)),
            None => None,
        }
    }

    fn contains(&self, nodes: &[T]) -> bool {
        self.container.contains_key(nodes)
    }

    fn len(&self) -> usize {
        self.container.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = HxUnsizedRef<'a, T, W>> + 'a
    where
        T: 'a,
        W: 'a,
    {
        self.container
            .iter()
            .map(|(nodes, weight)| HxUnsizedRef::new(nodes, weight))
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = HxUnsizedRefMut<'a, T, W>> + 'a
    where
        T: 'a,
        W: 'a,
    {
        self.container
            .iter_mut()
            .map(|(nodes, weight)| HxUnsizedRefMut::new(nodes, weight))
    }

    fn retain(&mut self, mut f: impl FnMut(&HxUnsizedRef<'_, T, W>) -> bool) {
        self.container.retain(|nodes, weight| {
            let hx_ref = HxUnsizedRef::new(nodes, weight);
            f(&hx_ref)
        });
    }

    fn take_flat_nodes(&mut self) -> Vec<T> {
        std::mem::take(&mut self.container.keys)
    }

    fn take_weights(&mut self) -> Vec<W> {
        std::mem::take(&mut self.container.values)
    }
}
