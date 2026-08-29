use rhyperx_core::graph::{AdjList, Undirected};

use crate::misc::cycle::{count_c4, count_c4_no_sort};
use crate::misc::sorting::sort_by_degree;
use crate::triangle::cetc::{cetc, cetc_s};
use crate::triangle::forward::{forward, forward_hashed, forward_hbs};
use crate::triangle::kclist::kclist;

fn test_graph() -> AdjList<u32, (), Undirected> {
    let mut g: AdjList<u32, (), Undirected> = AdjList::with_nodes(5);
    g.insert_edge(0, 1, ());
    g.insert_edge(1, 2, ());
    g.insert_edge(2, 0, ());
    g.insert_edge(2, 3, ());
    g.insert_edge(3, 4, ());
    g.insert_edge(4, 1, ());
    g
}

#[test]
fn triangle_counts_agree() {
    let mut g = test_graph();
    g.sort_neighbors();

    assert_eq!(kclist(&g), 1);
    assert_eq!(cetc(&g), 1);
    assert_eq!(cetc_s(&g), 1);
    assert_eq!(forward(&g, false), 1);
    assert_eq!(forward(&g, true), 1);
    assert_eq!(forward_hashed(&g, None), 1);
    assert_eq!(forward_hbs(&g, false), 1);
    assert_eq!(forward_hbs(&g, true), 1);
}

#[test]
fn cycle_counts() {
    let mut g = test_graph();
    assert_eq!(count_c4(&mut g), 1);

    let mut g2 = test_graph();
    let (order_pos, _) = sort_by_degree(&mut g2, false);
    assert_eq!(count_c4_no_sort(&g2, &order_pos.order), 1);
}
