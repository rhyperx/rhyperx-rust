use rhyperx_core::motif::CompactMotif;
use std::hint::black_box;

fn main() {
    // Hot path 1: motif enumeration + connectivity filter (fingerprint workload)
    let mut count = 0usize;
    for m in <CompactMotif!(3)>::enum_motifs(2..=3) {
        if black_box(&m).is_connected() {
            count += 1;
        }
    }
    println!("connected order-3 motifs: {}", count);

    // Hot path 2: runtime-indexed bit ops (add_edge / iteration)
    let mut x: CompactMotif!(5) = black_box(Default::default());
    for i in 0..31u32 {
        x.add_edge(black_box(i as usize));
    }
    let mut c = 0usize;
    for e in x.iter_edges_ids() {
        c = c.wrapping_add(black_box(e));
    }
    println!("edge sum: {}", c);

    // Hot path 3: edge insertion by nodes (edge_id_from_bitset path)
    let mut y: CompactMotif!(4) = black_box(Default::default());
    for i in 0..1000u32 {
        let a = (i % 4) as u8;
        let b = ((i + 1) % 4) as u8;
        black_box(y.add_edge_with_nodes_unchecked([a, b]));
    }
    println!("y edges: {}", y.edge_count());
}
