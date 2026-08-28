#![allow(dead_code)]
// 2-level representation. First level leverages sorted list sorting, second level is a bitset.
// Should be faster on sparse graphs

use std::{
    fmt::Binary,
    ops::{BitAnd, BitAndAssign, BitOrAssign},
};

use num_traits::PrimInt;

use crate::types::NodeId;

#[derive(Clone)]
pub struct HCBS<T> {
    pub bits: Vec<T>,
    pub offsets: Vec<usize>,
}

pub struct HCBSGraph<T> {
    pub nodes: Vec<HCBS<T>>,
}

impl<T> HCBS<T>
where
    T: PrimInt + BitOrAssign,
{
    fn new() -> Self {
        HCBS {
            bits: Vec::new(),
            offsets: Vec::new(),
        }
    }

    pub fn from(elements: &mut [usize], sort: bool) -> Self {
        if sort {
            elements.sort_unstable();
        }

        let bit_size = size_of::<T>() * 8;

        let mut next = T::zero();
        let mut offset = 0;
        let mut base = 0;
        let mut cbs = Vec::new();
        let mut offsets = Vec::new();

        // println!("Neigboors: {:?}", neigboors);
        for n in elements.iter().cloned() {
            let mut r_n = n - base;
            // println!("n: {}, r_n: {} offset: {}, base: {}", n, r_n, offset, base);
            if r_n >= bit_size {
                if next != T::zero() {
                    cbs.push(next);
                    offsets.push(offset);
                }
                offset += r_n / bit_size;
                base = offset * bit_size;
                next = T::zero();
                r_n = n - base;
            }
            next |= T::one() << r_n;
        }

        if next != T::zero() {
            cbs.push(next);
            offsets.push(offset);
        }

        Self { bits: cbs, offsets }
    }

    /// Optimization: assume that the last element is always the largest, so we can append it
    /// without checks for reallocation
    #[inline(always)]
    pub fn append(&mut self, element: NodeId) {
        let bit_size = (size_of::<T>() * 8) as u32;
        let block_offset = element / bit_size;
        let bit_offset = element % bit_size;
        let new_block = T::one() << bit_offset as usize;

        if self.offsets.is_empty() || *self.offsets.last().unwrap() != block_offset as usize {
            self.bits.push(new_block);
            self.offsets.push(block_offset as usize);
        } else {
            *self.bits.last_mut().unwrap() |= new_block;
        }
    }
}

impl<T> Default for HCBSGraph<T>
where
    T: PrimInt + BitOrAssign + BitAnd<Output = T> + BitAndAssign + Binary,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> HCBSGraph<T>
where
    T: PrimInt + BitOrAssign + BitAnd<Output = T> + BitAndAssign + Binary,
{
    pub fn new() -> Self {
        HCBSGraph { nodes: Vec::new() }
    }

    pub fn with_nodes(n: usize) -> Self {
        HCBSGraph {
            nodes: vec![HCBS::new(); n],
        }
    }

    pub fn from(graph: &mut [Vec<usize>], sort: bool) -> Self {
        let mut rep = Vec::new();
        for neigboors in graph.iter_mut() {
            rep.push(HCBS::from(neigboors, sort));
        }

        Self { nodes: rep }
    }

    #[inline(always)]
    /// Optimization: assume that node v is biggest neighbor of u, so we can append it directly
    /// without checking for reallocation in the underlying HCBS structure.
    pub fn append_neighbor(&mut self, u: NodeId, v: NodeId) {
        self.nodes[u as usize].append(v);
    }

    pub fn count_common_neighbors(&self, u: usize, v: usize) -> usize {
        let mut count = 0;
        let a = &self.nodes[u].offsets;
        let b = &self.nodes[v].offsets;

        let (mut i, mut j) = (0, 0);
        while i < a.len() && j < b.len() {
            if a[i] == b[j] {
                count += (self.nodes[u].bits[i] & self.nodes[v].bits[j]).count_ones() as usize;
                i += 1;
                j += 1;
            } else if a[i] < b[j] {
                i += 1;
            } else {
                j += 1;
            }
        }
        count
    }
}
