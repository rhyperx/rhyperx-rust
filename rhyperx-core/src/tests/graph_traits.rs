use crate::graph::{
    AdjList, AdjSet, Directed, EdgeIdGraph, EdgeIteration, GraphBase, IncList, IncSet, InsertEdge,
    InsertEdgeWithId, MultiedgeOps, NeighborRetrieval, RemoveEdge, Undirected,
};
use crate::types::NodeId;

#[test]
fn iter_edges_directed_adj_list() {
    let mut g: AdjList<u32, u32, Directed> = AdjList::with_nodes(3);
    g.insert_edge(0, 1, 10);
    g.insert_edge(1, 2, 20);
    g.insert_edge(2, 0, 30);
    g.insert_edge(1, 1, 40);
    g.insert_edge(0, 1, 11);

    let mut edges: Vec<(u32, u32, u32)> =
        g.iter_edges().map(|e| (e.from, e.to, *e.weight)).collect();
    edges.sort_unstable();
    assert_eq!(
        edges,
        vec![(0, 1, 10), (0, 1, 11), (1, 1, 40), (1, 2, 20), (2, 0, 30)]
    );
}

#[test]
fn iter_edges_undirected_adj_list() {
    let mut g: AdjList<u32, u32, Undirected> = AdjList::with_nodes(3);
    g.insert_edge(0, 1, 10);
    g.insert_edge(1, 2, 20);
    g.insert_edge(1, 1, 40);
    g.insert_edge(0, 1, 11);

    assert_eq!(GraphBase::m(&g), 4);
    let mut edges: Vec<(u32, u32, u32)> =
        g.iter_edges().map(|e| (e.from, e.to, *e.weight)).collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![(0, 1, 10), (0, 1, 11), (1, 1, 40), (1, 2, 20)]);
}

#[test]
fn iter_edges_undirected_adj_set() {
    let mut g: AdjSet<u32, u32, Undirected> = AdjSet::with_nodes(3);
    g.insert_edge(0, 2, 10);
    g.insert_edge(1, 1, 40);
    g.insert_edge(0, 2, 11);

    assert_eq!(GraphBase::m(&g), 2);
    let mut edges: Vec<(u32, u32)> = g.iter_edges().map(|e| (e.from, e.to)).collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![(0, 2), (1, 1)]);
}

#[test]
fn iter_edges_undirected_after_remove_multiedges() {
    let mut g: AdjList<u32, (), Undirected> = AdjList::with_nodes(2);
    g.insert_edge(0, 0, ());
    g.insert_edge(0, 0, ());
    assert_eq!(GraphBase::m(&g), 2);

    MultiedgeOps::remove_multiedges(&mut g);
    assert_eq!(GraphBase::m(&g), 1);

    let edges: Vec<(u32, u32)> = g.iter_edges().map(|e| (e.from, e.to)).collect();
    assert_eq!(edges, vec![(0, 0)]);
}

#[test]
fn iter_edges_directed_inc_list() {
    let mut g: IncList<u32, u32, Directed, u32> = IncList::with_nodes(2);
    g.insert_edge(0, 1, 10);
    g.insert_edge(1, 0, 20);
    g.insert_edge(1, 1, 30);

    let mut edges: Vec<(u32, u32, u32)> =
        g.iter_edges().map(|e| (e.from, e.to, *e.weight)).collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![(0, 1, 10), (1, 0, 20), (1, 1, 30)]);
}

#[test]
fn iter_edges_undirected_inc_list() {
    let mut g: IncList<u32, u32, Undirected, u32> = IncList::with_nodes(3);
    g.insert_edge(0, 1, 10);
    g.insert_edge(1, 2, 20);
    g.insert_edge(1, 1, 30);
    g.insert_edge(0, 1, 11);

    let mut edges: Vec<(u32, u32, u32)> =
        g.iter_edges().map(|e| (e.from, e.to, *e.weight)).collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![(0, 1, 10), (0, 1, 11), (1, 1, 30), (1, 2, 20)]);
}

#[test]
fn iter_edges_undirected_inc_set() {
    let mut g: IncSet<u32, u32, Undirected, u32> = IncSet::with_nodes(2);
    g.insert_edge(0, 1, 10);
    g.insert_edge(0, 0, 30);

    assert_eq!(GraphBase::m(&g), 2);
    let mut edges: Vec<(u32, u32)> = g.iter_edges().map(|e| (e.from, e.to)).collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![(0, 0), (0, 1)]);
}

#[test]
fn neighbor_retrieval() {
    let mut g: AdjList<u32, u32, Undirected> = AdjList::with_nodes(3);
    g.insert_edge(0, 1, 10);
    g.insert_edge(0, 2, 20);

    assert_eq!(NeighborRetrieval::degree(&g, 0), 2);
    assert_eq!(NeighborRetrieval::degree(&g, 1), 1);
    let mut ns: Vec<u32> = NeighborRetrieval::iter_neighbors(&g, 0).collect();
    ns.sort_unstable();
    assert_eq!(ns, vec![1, 2]);
    let mut ws: Vec<(u32, u32)> = g
        .iter_weighted_neighbors(0)
        .map(|n| (*n.node, *n.weight))
        .collect();
    ws.sort_unstable();
    assert_eq!(ws, vec![(1, 10), (2, 20)]);

    let mut i: IncList<u32, u32, Undirected, u32> = IncList::with_nodes(3);
    i.insert_edge(0, 1, 10);
    i.insert_edge(0, 2, 20);

    assert_eq!(NeighborRetrieval::degree(&i, 0), 2);
    let mut ins: Vec<u32> = NeighborRetrieval::iter_neighbors(&i, 0).collect();
    ins.sort_unstable();
    assert_eq!(ins, vec![1, 2]);
    let mut iws: Vec<(u32, u32)> = i
        .iter_weighted_neighbors(0)
        .map(|n| (*n.node, *n.weight))
        .collect();
    iws.sort_unstable();
    assert_eq!(iws, vec![(1, 10), (2, 20)]);
}

#[test]
fn insert_remove_multiedge_ops() {
    let mut s: AdjSet<u32, u32, Undirected> = AdjSet::with_nodes(2);
    assert!(InsertEdge::insert_edge(&mut s, 0, 1, 5));
    assert!(!InsertEdge::insert_edge(&mut s, 0, 1, 6));
    assert_eq!(GraphBase::m(&s), 1);

    let mut i: IncList<u32, u32, Undirected, u32> = IncList::with_nodes(2);
    let e0 = InsertEdgeWithId::insert_edge(&mut i, 0, 1, 10);
    let e1 = InsertEdgeWithId::insert_edge(&mut i, 1, 1, 20);
    assert_eq!(e0, 0);
    assert_eq!(e1, 1);
    assert_eq!(GraphBase::m(&i), 2);
    let ids: Vec<u32> = EdgeIdGraph::iter_incident_neighbors(&i, 0)
        .map(|n| *n.edge)
        .collect();
    assert_eq!(ids, vec![e0]);
    assert_eq!(RemoveEdge::remove_edges_between(&mut i, 0, 1), 1);
    assert_eq!(RemoveEdge::remove_self_loops(&mut i), 1);
    assert_eq!(GraphBase::m(&i), 0);

    let mut g: AdjList<u32, u32, Undirected> = AdjList::with_nodes(2);
    g.insert_edge(0, 1, 10);
    g.insert_edge(0, 1, 11);
    assert_eq!(MultiedgeOps::count_multiedges(&g), 1);
    assert!(MultiedgeOps::has_multiedges(&g));
    assert_eq!(MultiedgeOps::remove_multiedges(&mut g), 1);
    assert!(!MultiedgeOps::has_multiedges(&g));
    assert_eq!(GraphBase::m(&g), 1);
}

fn count_edges<G: EdgeIteration>(g: &G) -> usize {
    g.iter_edges().count()
}

fn total_degree<G: NeighborRetrieval<Dir = Undirected>>(g: &G) -> usize {
    (0..g.n())
        .map(|u| g.degree(G::NodeIdType::from_usize(u)))
        .sum()
}

#[test]
fn generic_over_representations() {
    let mut a: AdjList<u32, u32, Undirected> = AdjList::with_nodes(3);
    a.insert_edge(0, 1, 1);
    a.insert_edge(1, 2, 2);
    a.insert_edge(2, 2, 3);

    let mut i: IncList<u32, u32, Undirected, u32> = IncList::with_nodes(3);
    i.insert_edge(0, 1, 1);
    i.insert_edge(1, 2, 2);
    i.insert_edge(2, 2, 3);

    assert_eq!(count_edges(&a), 3);
    assert_eq!(count_edges(&i), 3);
    assert_eq!(total_degree(&a), 6);
    assert_eq!(total_degree(&i), 6);
}
