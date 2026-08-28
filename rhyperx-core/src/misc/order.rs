use crate::types::NodeId;
use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Order<N: NodeId>(Vec<N>);

impl<N: NodeId> Order<N> {
    pub fn new(order: Vec<N>) -> Self {
        Self(order)
    }
    pub fn empty() -> Self {
        Self(Vec::new())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // Encapsulate mutation and iteration
    pub fn reverse(&mut self) {
        self.0.reverse();
    }
    pub fn iter(&self) -> std::slice::Iter<'_, N> {
        self.0.iter()
    }
}

impl<N: NodeId> Index<usize> for Order<N> {
    type Output = N;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<N: NodeId> IndexMut<usize> for Order<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pos(Vec<usize>);

impl Pos {
    pub fn new(pos: Vec<usize>) -> Self {
        Self(pos)
    }
    pub fn empty() -> Self {
        Self(Vec::new())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn set(&mut self, index: usize, value: usize) {
        self.0[index] = value;
    }
}

impl Index<usize> for Pos {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Pos {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

pub trait OrderType {
    const ORDER_TYPE: &'static str;
}

pub struct Ascending;

pub struct Descending;

pub struct Other;

impl OrderType for Ascending {
    const ORDER_TYPE: &'static str = "Ascending";
}

impl OrderType for Descending {
    const ORDER_TYPE: &'static str = "Descending";
}

impl OrderType for Other {
    const ORDER_TYPE: &'static str = "Other";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderAndPos<N: NodeId, T: OrderType> {
    pub order: Order<N>,
    pub pos: Pos,
    _marker: PhantomData<T>,
}

impl<N: NodeId, T: OrderType> OrderAndPos<N, T> {
    pub fn new(order: Order<N>, pos: Pos) -> Self {
        Self {
            order,
            pos,
            _marker: PhantomData,
        }
    }

    pub fn empty() -> Self {
        Self {
            order: Order::empty(),
            pos: Pos::empty(),
            _marker: PhantomData,
        }
    }

    // Common reversal helper internal to the struct. Does not change the OrderType, so it should
    // not be used directly by users. Use the `reversed` method instead.
    fn reverse_internal(&mut self) {
        self.order.reverse();
        for (p, v) in self.order.iter().enumerate() {
            self.pos.set(v.as_usize(), p);
        }
    }
}

impl<N: NodeId> OrderAndPos<N, Ascending> {
    pub fn reversed(mut self) -> OrderAndPos<N, Descending> {
        self.reverse_internal();
        OrderAndPos::new(self.order, self.pos)
    }
}

impl<N: NodeId> OrderAndPos<N, Descending> {
    pub fn reversed(mut self) -> OrderAndPos<N, Ascending> {
        self.reverse_internal();
        OrderAndPos::new(self.order, self.pos)
    }
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum OrderOrPos {
//     Order(Order),
//     Pos(Pos),
// }
//
// impl OrderOrPos {
//     pub fn get_order(&self) -> Cow<'_, [NodeId]> {
//         match self {
//             OrderOrPos::Order(order) => Cow::Borrowed(order),
//             OrderOrPos::Pos(pos) => {
//                 let mut rv = vec![0; pos.len()];
//                 for (i, &p) in pos.iter().enumerate() {
//                     rv[p] = i as NodeId;
//                 }
//                 Cow::Owned(rv)
//             }
//         }
//     }
//     pub fn get_pos(&self) -> Cow<'_, [usize]> {
//         match self {
//             OrderOrPos::Order(order) => {
//                 let mut rv = vec![0; order.len()];
//                 for (i, &node) in order.iter().enumerate() {
//                     rv[node as usize] = i;
//                 }
//                 Cow::Owned(rv)
//             }
//             OrderOrPos::Pos(pos) => Cow::Borrowed(pos),
//         }
//     }
// }
