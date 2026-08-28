use hashbrown::{HashMap, HashSet};
use std::hash::Hash;

use crate::{
    misc::neighbors_sorted_list_cloj,
    types::{
        NodeId,
        adj_list::{AdjList, common::Undirected, traits::Incidence},
    },
};

/// Returns all maximal cliques in the undirected graph.
///
/// This is an iterative implementation of the Bron-Kerbosch algorithm with pivoting,
/// translating the provided Python implementation to avoid recursion depth limits.
pub fn find_cliques<W, I: Incidence>(adj: &AdjList<W, Undirected, I>) -> Vec<Vec<NodeId>> {
    let mut cliques = Vec::new();

    if adj.n() == 0 {
        return cliques;
    }

    // Convert adjacency list to HashSets for O(1) lookups and set operations (ignores adj-loops)
    let mut adj_sets: Vec<HashSet<NodeId>> = Vec::with_capacity(adj.n());
    for u in 0..adj.n() {
        let mut set = HashSet::new();
        for n in &adj[u] {
            let v = n.node;
            if v as usize != u {
                set.insert(v);
            }
        }
        adj_sets.push(set);
    }

    // Initialize candidate sets
    let mut cand: HashSet<NodeId> = (0..adj.n() as NodeId).collect();
    let mut subg: HashSet<NodeId> = cand.clone();

    if cand.is_empty() {
        return cliques;
    }

    let mut stack = Vec::new();
    let mut q: Vec<NodeId> = Vec::new();

    // Placeholder for Q[-1] logic in Python
    q.push(0);

    // Find initial pivot
    let u_pivot = *subg
        .iter()
        .max_by_key(|&&u| cand.intersection(&adj_sets[u as usize]).count())
        .unwrap();

    let mut ext_u: Vec<NodeId> = cand
        .difference(&adj_sets[u_pivot as usize])
        .cloned()
        .collect();

    loop {
        if let Some(q_node) = ext_u.pop() {
            cand.remove(&q_node);

            // Q[-1] = q
            if let Some(last) = q.last_mut() {
                *last = q_node;
            }

            let adj_q = &adj_sets[q_node as usize];
            let subg_q: HashSet<NodeId> = subg.intersection(adj_q).cloned().collect();

            if subg_q.is_empty() {
                // Yield Q[:]
                cliques.push(q.clone());
            } else {
                let cand_q: HashSet<NodeId> = cand.intersection(adj_q).cloned().collect();
                if !cand_q.is_empty() {
                    // Push state to stack
                    stack.push((subg, cand, ext_u));
                    q.push(0); // Q.append(None)

                    subg = subg_q;
                    cand = cand_q;

                    // Find new pivot
                    let u_pivot = *subg
                        .iter()
                        .max_by_key(|&&u| cand.intersection(&adj_sets[u as usize]).count())
                        .unwrap();

                    ext_u = cand
                        .difference(&adj_sets[u_pivot as usize])
                        .cloned()
                        .collect();
                }
            }
        } else {
            // Backtrack
            q.pop();
            if let Some((prev_subg, prev_cand, prev_ext)) = stack.pop() {
                subg = prev_subg;
                cand = prev_cand;
                ext_u = prev_ext;
            } else {
                break;
            }
        }
    }

    cliques
}
