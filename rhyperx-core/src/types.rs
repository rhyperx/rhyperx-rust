use std::hash::Hash;

pub trait NodeId: Clone + Copy + Hash + Eq + Ord {
    fn as_usize(&self) -> usize;
    fn from_usize(id: usize) -> Self;
    fn zero() -> Self;
}

pub trait EdgeId: Clone + Copy + Hash + Eq + Ord {
    fn as_usize(&self) -> usize;
    fn from_usize(id: usize) -> Self;
    fn zero() -> Self;
}

macro_rules! impl_node_id_prim_type {
    ($($t:ty),*) => {
        $(
            impl NodeId for $t {
                fn as_usize(&self) -> usize {
                    *self as usize
                }

                fn from_usize(id: usize) -> Self {
                    id as $t
                }

                fn zero() -> Self {
                    0 as $t
                }
            }
        )*
    };
}

macro_rules! impl_edge_id_prim_type {
    ($($t:ty),*) => {
        $(
            impl EdgeId for $t {
                fn as_usize(&self) -> usize {
                    *self as usize
                }

                fn from_usize(id: usize) -> Self {
                    id as $t
                }

                fn zero() -> Self {
                    0 as $t
                }
            }
        )*
    };
}

impl_node_id_prim_type!(u8, u16, u32, u64, usize);
impl_edge_id_prim_type!(u8, u16, u32, u64, usize);
