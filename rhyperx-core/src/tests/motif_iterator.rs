use crate::compact_motif;
use crate::motif::CompactMotif;

#[test]
fn enum_motifs_full_range() {
    let motifs: Vec<CompactMotif!(3)> = <CompactMotif!(3)>::enum_motifs(1..=3).collect();
    assert_eq!(motifs.len(), 1 << 7);
    // Every yielded edge id must be in the valid [0, 7) range.
    for m in &motifs {
        for e in m.iter_edges_ids() {
            assert!((0..7).contains(&e));
        }
    }
    // The last motif contains every edge.
    assert_eq!(motifs.last().unwrap().edge_count(), 7);
}

#[test]
fn enum_motifs_restricted_range() {
    // Only edges of size 2 or 3 among 3 nodes: 3 + 1 = 4 edges.
    let motifs: Vec<CompactMotif!(3)> = <CompactMotif!(3)>::enum_motifs(2..=3).collect();
    assert_eq!(motifs.len(), 1 << 4);
    // No edge id outside the [3, 6] range may appear.
    for m in &motifs {
        for e in m.iter_edges_ids() {
            assert!((3..=6).contains(&e));
        }
    }
    // The last motif contains every size-2 and size-3 edge.
    assert_eq!(motifs.last().unwrap().edge_count(), 4);
}

#[test]
fn enum_motifs_empty() {
    let m: CompactMotif!(3) = compact_motif!(3);
    assert!(m.is_empty());
}
