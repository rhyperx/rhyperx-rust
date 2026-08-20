use foldhash::fast::FixedState;
use hashbrown::HashMap;

use super::traits::{
    AdjConfig, Direction, EdgeIdTrait, Incidence, NeighborContainer, NeighborLike,
};

#[macro_export]
macro_rules! check_bounds_debug {
    ($range:expr, $($node:expr),+ $(,)?) => {
        $(
            debug_assert!(
                $range.contains(&(($node) as usize)),
                "NodeId {} is out of bounds for range {:?}",
                $node,
                $range
            );
        )+
    };
}

pub type NodeId = u32;
pub type EdgeId = u32;

impl EdgeIdTrait for EdgeId {
    const ZERO: Self = 0;

    fn advance(&mut self) {
        *self += 1;
    }
}

impl EdgeIdTrait for () {
    const ZERO: Self = ();

    fn advance(&mut self) {}
}

#[derive(Clone, Copy)]
pub struct Directed;

#[derive(Clone, Copy)]
pub struct Undirected;
impl Direction for Directed {
    const IS_DIRECTED: bool = true;
}
impl Direction for Undirected {
    const IS_DIRECTED: bool = false;
}

#[derive(Clone, Copy)]
pub struct WithIncidence;

#[derive(Clone, Copy)]
pub struct WithoutIncidence;
impl Incidence for WithIncidence {
    type EdgeType = EdgeId;
    const HAS_INCIDENCE: bool = true;
}
impl Incidence for WithoutIncidence {
    type EdgeType = ();
    const HAS_INCIDENCE: bool = false;
}

#[derive(Debug, Clone, Copy)]
pub struct Neighbor<W, I>
where
    I: EdgeIdTrait,
{
    pub node: NodeId,
    pub weight: W,
    pub edge: I,
}

impl<W, I: EdgeIdTrait> Neighbor<W, I> {
    pub fn new(node: NodeId, weight: W, edge: I) -> Self {
        Self { node, weight, edge }
    }
}

impl<W, I: EdgeIdTrait> PartialEq for Neighbor<W, I> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.edge == other.edge
    }
}
impl<W, I: EdgeIdTrait> Eq for Neighbor<W, I> {}

impl<W, I: EdgeIdTrait> PartialOrd for Neighbor<W, I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.node.cmp(&other.node).then(self.edge.cmp(&other.edge)))
    }
}
impl<W, I: EdgeIdTrait> Ord for Neighbor<W, I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub struct NeighborRef<'a, W, I: EdgeIdTrait> {
    pub node: &'a NodeId,
    pub weight: &'a W,
    pub edge: &'a I,
}

pub struct NeighborRefMut<'a, W, I: EdgeIdTrait> {
    pub node: &'a NodeId,
    pub weight: &'a mut W,
    pub edge: &'a I,
}

impl<W, I: EdgeIdTrait> NeighborLike for Neighbor<W, I> {
    type WeightType = W;
    type EdgeType = I;

    fn node(&self) -> NodeId {
        self.node
    }

    fn weight(&self) -> &Self::WeightType {
        &self.weight
    }

    fn weight_mut(&mut self) -> &mut Self::WeightType {
        &mut self.weight
    }

    fn edge(&self) -> Self::EdgeType {
        self.edge
    }
}

impl<W, I: EdgeIdTrait> NeighborContainer for Vec<Neighbor<W, I>> {
    type WeightType = W;
    type EdgeType = I;

    const SUPPORTS_MULTIEDGES: bool = true;

    fn empty() -> Self {
        Vec::new()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert(&mut self, node: NodeId, weight: Self::WeightType, edge: Self::EdgeType) -> bool {
        self.push(Neighbor { node, weight, edge });
        true
    }

    #[inline(always)]
    fn iter_neighbors(
        &self,
    ) -> impl Iterator<Item = NeighborRef<'_, Self::WeightType, Self::EdgeType>> {
        self.iter().map(|neighbor| NeighborRef {
            node: &neighbor.node,
            weight: &neighbor.weight,
            edge: &neighbor.edge,
        })
    }

    fn iter_neighbors_mut(
        &mut self,
    ) -> impl Iterator<Item = NeighborRefMut<'_, Self::WeightType, Self::EdgeType>> {
        self.iter_mut().map(|neighbor| NeighborRefMut {
            node: &neighbor.node,
            weight: &mut neighbor.weight,
            edge: &neighbor.edge,
        })
    }

    fn into_iter_neighbors(
        self,
    ) -> impl Iterator<Item = Neighbor<Self::WeightType, Self::EdgeType>> {
        self.into_iter()
    }

    fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(NeighborRef<Self::WeightType, Self::EdgeType>) -> bool,
    {
        self.retain(|neighbor| {
            let neighbor_ref = NeighborRef {
                node: &neighbor.node,
                weight: &neighbor.weight,
                edge: &neighbor.edge,
            };
            f(neighbor_ref)
        });
    }
}

impl<W, I: EdgeIdTrait> NeighborContainer for HashMap<NodeId, (W, I), FixedState> {
    type WeightType = W;
    type EdgeType = I;

    const SUPPORTS_MULTIEDGES: bool = false;

    fn empty() -> Self {
        HashMap::with_hasher(FixedState::default())
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert(&mut self, node: NodeId, weight: Self::WeightType, edge: Self::EdgeType) -> bool {
        match self.get_mut(&node) {
            Some(n) => {
                n.0 = weight;
                false
            }
            None => {
                self.insert(node, (weight, edge));
                true
            }
        }
    }

    fn iter_neighbors(
        &self,
    ) -> impl Iterator<Item = NeighborRef<'_, Self::WeightType, Self::EdgeType>> {
        self.iter()
            .map(|(node, (weight, edge))| NeighborRef { node, weight, edge })
    }

    fn iter_neighbors_mut(
        &mut self,
    ) -> impl Iterator<Item = NeighborRefMut<'_, Self::WeightType, Self::EdgeType>> {
        self.iter_mut()
            .map(|(node, (weight, edge))| NeighborRefMut { node, weight, edge })
    }

    fn into_iter_neighbors(
        self,
    ) -> impl Iterator<Item = Neighbor<Self::WeightType, Self::EdgeType>> {
        self.into_iter()
            .map(|(node, (weight, edge))| Neighbor { node, weight, edge })
    }

    fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(NeighborRef<Self::WeightType, Self::EdgeType>) -> bool,
    {
        self.retain(|node, (weight, edge)| {
            let neighbor_ref = NeighborRef { node, weight, edge };
            f(neighbor_ref)
        });
    }
}

impl<W, D, I, C> AdjConfig for (W, D, I, C)
where
    D: Direction,
    I: Incidence,
    C: NeighborContainer<WeightType = W, EdgeType = <I as Incidence>::EdgeType>,
{
    type Weight = W;
    type Dir = D;
    type Inc = I;
    type Container = C;
}
