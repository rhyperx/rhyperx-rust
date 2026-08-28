#[cfg(feature = "serialize")]
use rkyv::{Archive, Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use crate::{error::HypergraphError, types::NodeId};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Archive, Deserialize, Serialize))]
pub struct SizedHx<const N: usize, T: NodeId, W> {
    pub nodes: [T; N],
    pub weight: W,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Archive, Deserialize, Serialize))]
pub struct UnsizedHx<T: NodeId, W> {
    pub nodes: Vec<T>,
    pub weight: W,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Archive, Deserialize, Serialize))]
pub struct HxSizedRef<'a, const N: usize, T, W> {
    pub nodes: &'a [T; N],
    pub weight: &'a W,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Archive, Deserialize, Serialize))]
pub struct HxSizedRefMut<'a, const N: usize, T, W> {
    pub nodes: &'a mut [T; N],
    pub weight: &'a mut W,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Archive, Deserialize, Serialize))]
pub struct HxUnsizedRef<'a, T, W> {
    pub nodes: &'a [T],
    pub weight: &'a W,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Archive, Deserialize, Serialize))]
pub struct HxUnsizedRefMut<'a, T, W> {
    pub nodes: &'a mut [T],
    pub weight: &'a mut W,
}

// Inherent implementations for SizedHx
impl<const N: usize, T: NodeId, W> SizedHx<N, T, W> {
    pub fn new(mut nodes: [T; N], weight: W) -> Result<Self, HypergraphError>
    where
        T: Ord + Copy,
    {
        nodes.sort_unstable();

        if let Some(&dup) = nodes
            .windows(2)
            .find_map(|w| (w[0] == w[1]).then_some(&w[0]))
        {
            return Err(HypergraphError::DuplicateNodes(dup.as_usize()));
        }

        Ok(Self { nodes, weight })
    }

    pub fn new_unchecked(nodes: [T; N], weight: W) -> Self {
        Self { nodes, weight }
    }
}

// Inherent implementations for UnsizedHx
impl<T: NodeId, W> UnsizedHx<T, W> {
    pub fn new(mut nodes: Vec<T>, weight: W) -> Result<Self, HypergraphError>
    where
        T: Ord + Copy,
    {
        nodes.sort_unstable();

        if let Some(&dup) = nodes
            .windows(2)
            .find_map(|w| (w[0] == w[1]).then_some(&w[0]))
        {
            return Err(HypergraphError::DuplicateNodes(dup.as_usize()));
        }

        Ok(Self { nodes, weight })
    }

    pub fn new_unchecked(nodes: Vec<T>, weight: W) -> Self {
        Self { nodes, weight }
    }
}

impl<'a, const N: usize, T, W> HxSizedRef<'a, N, T, W> {
    pub fn new(nodes: &'a [T; N], weight: &'a W) -> Self {
        Self { nodes, weight }
    }

    pub fn iter_nodes(&self) -> std::slice::Iter<'_, T> {
        self.nodes.iter()
    }

    pub fn as_unsized(self) -> HxUnsizedRef<'a, T, W> {
        HxUnsizedRef {
            nodes: self.nodes,
            weight: self.weight,
        }
    }
}

impl<'a, const N: usize, T, W> HxSizedRefMut<'a, N, T, W> {
    pub fn new(nodes: &'a mut [T; N], weight: &'a mut W) -> Self {
        Self { nodes, weight }
    }

    pub fn iter_nodes(&self) -> std::slice::Iter<'_, T> {
        self.nodes.iter()
    }

    pub fn iter_nodes_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.nodes.iter_mut()
    }

    pub fn as_unsized(self) -> HxUnsizedRefMut<'a, T, W> {
        HxUnsizedRefMut {
            nodes: self.nodes,
            weight: self.weight,
        }
    }
}

// Inherent implementations for UnsizedRef
impl<'a, T, W> HxUnsizedRef<'a, T, W> {
    pub fn new(nodes: &'a [T], weight: &'a W) -> Self {
        Self { nodes, weight }
    }

    pub fn iter_nodes(&self) -> std::slice::Iter<'_, T> {
        self.nodes.iter()
    }

    pub fn try_into_sized<const N: usize>(
        self,
    ) -> Result<HxSizedRef<'a, N, T, W>, HypergraphError> {
        if self.nodes.len() == N {
            let nodes_ref = unsafe { &*(self.nodes.as_ptr() as *const [T; N]) };
            Ok(HxSizedRef {
                nodes: nodes_ref,
                weight: self.weight,
            })
        } else {
            Err(HypergraphError::InvalidHyperedgeSize {
                expected: N,
                got: self.nodes.len(),
            })
        }
    }

    pub fn into_sized_unchecked<const N: usize>(self) -> HxSizedRef<'a, N, T, W> {
        HxSizedRef {
            nodes: self.nodes.try_into().expect(&format!(
                "Unexpected hyperedge size in \"into_sized_unchecked\": expected {}, got {}",
                N,
                self.nodes.len()
            )),
            weight: self.weight,
        }
    }
}

impl<'a, T, W> HxUnsizedRefMut<'a, T, W> {
    pub fn new(nodes: &'a mut [T], weight: &'a mut W) -> Self {
        Self { nodes, weight }
    }

    pub fn iter_nodes(&self) -> std::slice::Iter<'_, T> {
        self.nodes.iter()
    }

    pub fn iter_nodes_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.nodes.iter_mut()
    }

    pub fn try_into_sized<const N: usize>(
        self,
    ) -> Result<HxSizedRefMut<'a, N, T, W>, HypergraphError> {
        if self.nodes.len() == N {
            let nodes_ref = unsafe { &mut *(self.nodes.as_mut_ptr() as *mut [T; N]) };
            Ok(HxSizedRefMut {
                nodes: nodes_ref,
                weight: self.weight,
            })
        } else {
            Err(HypergraphError::InvalidHyperedgeSize {
                expected: N,
                got: self.nodes.len(),
            })
        }
    }

    pub fn into_sized_unchecked<const N: usize>(self) -> HxSizedRefMut<'a, N, T, W> {
        let len = self.nodes.len();
        HxSizedRefMut {
            nodes: self.nodes.try_into().expect(&format!(
                "Unexpected hyperedge size in \"into_sized_unchecked\": expected {}, got {}",
                N, len
            )),
            weight: self.weight,
        }
    }
}

// ToOwned
impl<const N: usize, T: NodeId, W: Clone> HxSizedRef<'_, N, T, W> {
    fn to_owned(&self) -> SizedHx<N, T, W> {
        SizedHx {
            nodes: self.nodes.clone(),
            weight: self.weight.clone(),
        }
    }
}

impl<const N: usize, T: NodeId, W: Clone> HxSizedRefMut<'_, N, T, W> {
    fn to_owned(&self) -> SizedHx<N, T, W> {
        SizedHx {
            nodes: self.nodes.clone(),
            weight: self.weight.clone(),
        }
    }
}

// Hash
impl<const N: usize, T: NodeId, W> Hash for SizedHx<N, T, W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nodes.hash(state);
    }
}

impl<T: NodeId, W> Hash for UnsizedHx<T, W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nodes.hash(state);
    }
}

impl<const N: usize, T: Hash, W> Hash for HxSizedRef<'_, N, T, W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nodes.hash(state);
    }
}

impl<const N: usize, T: Hash, W> Hash for HxSizedRefMut<'_, N, T, W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nodes.hash(state);
    }
}

impl<T: Hash, W> Hash for HxUnsizedRef<'_, T, W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nodes.hash(state);
    }
}

impl<T: Hash, W> Hash for HxUnsizedRefMut<'_, T, W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nodes.hash(state);
    }
}

// PartialEq
impl<const N: usize, T: NodeId, W> PartialEq for SizedHx<N, T, W> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl<T: NodeId, W> PartialEq for UnsizedHx<T, W> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl<const N: usize, T: NodeId, W> PartialEq for HxSizedRef<'_, N, T, W> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl<const N: usize, T: NodeId, W> PartialEq for HxSizedRefMut<'_, N, T, W> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl<T: NodeId, W> PartialEq for HxUnsizedRef<'_, T, W> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl<T: NodeId, W> PartialEq for HxUnsizedRefMut<'_, T, W> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

// Eq
impl<const N: usize, T: NodeId, W> Eq for SizedHx<N, T, W> where T: Eq {}

impl<T: NodeId, W> Eq for UnsizedHx<T, W> where T: Eq {}

impl<const N: usize, T: NodeId, W> Eq for HxSizedRef<'_, N, T, W> where T: Eq {}

impl<const N: usize, T: NodeId, W> Eq for HxSizedRefMut<'_, N, T, W> where T: Eq {}

impl<T: NodeId, W> Eq for HxUnsizedRef<'_, T, W> where T: Eq {}

impl<T: NodeId, W> Eq for HxUnsizedRefMut<'_, T, W> where T: Eq {}

// Into Iter for SizedHx
impl<const N: usize, T: NodeId, W> IntoIterator for SizedHx<N, T, W> {
    type Item = T;
    type IntoIter = std::array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

// Into Iter for UnsizedHx
impl<T: NodeId, W> IntoIterator for UnsizedHx<T, W> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

// Reference Iteration for SizedHx
impl<'a, const N: usize, T: NodeId, W> IntoIterator for &'a SizedHx<N, T, W> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

// Reference Iteration for UnsizedHx
impl<'a, T: NodeId, W> IntoIterator for &'a UnsizedHx<T, W> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

// Mutable Reference Iteration for SizedHx
impl<'a, const N: usize, T: NodeId, W> IntoIterator for &'a mut SizedHx<N, T, W> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter_mut()
    }
}

// Mutable Reference Iteration for UnsizedHx
impl<'a, T: NodeId, W> IntoIterator for &'a mut UnsizedHx<T, W> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter_mut()
    }
}

// Equivalent
impl<'a, T: NodeId, W, const N: usize> hashbrown::Equivalent<SizedHx<N, T, W>> for [T; N] {
    fn equivalent(&self, key: &SizedHx<N, T, W>) -> bool {
        self == &key.nodes
    }
}

impl<'a, T: NodeId, W> hashbrown::Equivalent<UnsizedHx<T, W>> for [T] {
    fn equivalent(&self, key: &UnsizedHx<T, W>) -> bool {
        self == &key.nodes
    }
}
