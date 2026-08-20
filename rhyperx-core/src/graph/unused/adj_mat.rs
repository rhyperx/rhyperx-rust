use std::cmp::max;

use bit_set::BitSet;
use num_traits::AsPrimitive;
use pyo3::{pyclass, pymethods};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rkyv::{Archive, Deserialize, Serialize};

#[gen_stub_pyclass(module = "rust_core._core.graph")]
#[pyclass(from_py_object)]
pub struct AdjMat {
    pub mat: Vec<BitSet>,
}

impl Default for AdjMat {
    fn default() -> Self {
        Self::new()
    }
}

#[gen_stub_pymethods(module = "rust_core._core.graph")]
#[pymethods]
impl AdjMat {
    #[new]
    pub fn new() -> Self {
        Self { mat: Vec::new() }
    }
    pub fn with_nodes(n: usize) -> Self {
        let mut mat = Vec::new();
        mat.resize(n, BitSet::with_capacity(n));
        Self { mat }
    }

    pub fn from_edges<T: AsPrimitive<usize> + num_traits::Zero + Ord>(
        edges: &[(T, T)],
        directed: bool,
    ) -> Self {
        let mut mat = Vec::new();

        let n = edges
            .iter()
            .fold(T::zero(), |acc, (u, v)| max(acc, max(*u, *v)))
            .as_()
            + 1;

        mat.resize(n, BitSet::with_capacity(n));
        for (u, v) in edges {
            let u = u.as_();
            let v = v.as_();
            mat[u].insert(v);
            if !directed {
                mat[u].insert(u);
            }
        }
        Self { mat }
    }

    pub fn n(&self) -> usize {
        self.mat.len()
    }

    pub fn m(&self) -> usize {
        self.mat.len()
    }

    #[inline(always)]
    pub fn add_edge(&mut self, u: usize, v: usize) {
        self.mat[u].insert(v);
    }

    #[inline(always)]
    pub fn extend(&mut self, edges: Vec<(usize, usize)>) {
        for (u, v) in edges {
            self.mat[u].insert(v);
        }
    }
}
