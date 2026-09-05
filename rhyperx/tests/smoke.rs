//! Smoke tests for the `rhyperx` facade crate.
//!
//! Each test is feature-gated so the file compiles and runs under every
//! feature combination exercised by `cargo-hack`.

use rhyperx::collections::BinStore;
use rhyperx::compact_motif;
use rhyperx::error::HypergraphError;
use rhyperx::graph::{AdjList, Undirected};
use rhyperx::hypergraph::Hypergraph;
use rhyperx::motif::CompactMotif;
use rhyperx::types::NodeId;

#[test]
fn core_graph_is_re_exported() {
    let mut g: AdjList<u32, (), Undirected> = AdjList::with_nodes(3);
    g.insert_edge(0, 1, ());
    g.insert_edge(1, 2, ());
    assert_eq!(g.m(), 2);
    assert_eq!(g.n(), 3);
}

#[test]
fn core_hypergraph_is_re_exported() {
    let mut hg: Hypergraph<u32, ()> = Hypergraph::new();
    let inserted = hg
        .add_edge_slice(&mut [0, 1, 2], ())
        .expect("edge with distinct nodes must insert");
    assert!(inserted);
    assert_eq!(hg.m(), 1);
}

#[test]
fn core_motif_and_macros_are_re_exported() {
    let motif: CompactMotif!(3) = compact_motif!(3);
    assert_eq!(motif.edge_count(), 0);
    let _error: Option<HypergraphError> = None;
}

#[test]
fn core_collections_and_types_are_re_exported() {
    let bits = BinStore::<u32, 1>::with_elements([1]);
    assert!(bits.get_bit(1));

    fn _requires_node_id<T: NodeId>() {}
    let _ = _requires_node_id::<u32>;
}

#[cfg(feature = "algo")]
#[test]
fn algo_is_re_exported() {
    use rhyperx::algo::misc::sorting::degeneracy_ordering;
    use rhyperx::algo::triangle::forward::forward_hashed;

    let mut g: AdjList<u32, (), Undirected> = AdjList::with_nodes(3);
    g.insert_edge(0, 1, ());
    g.insert_edge(1, 2, ());
    g.insert_edge(2, 0, ());
    g.sort_neighbors();

    assert_eq!(forward_hashed(&g, None), 1);
    let _order = degeneracy_ordering(&g);
}

#[cfg(feature = "io")]
#[test]
fn io_is_re_exported() {
    use rhyperx::io::DatasetLoader;
    use rhyperx::io::loader::DatasetLoaderDispatcher;

    let dispatcher: DatasetLoaderDispatcher = DatasetLoader::builder();
    assert!(!dispatcher.cached);
}
