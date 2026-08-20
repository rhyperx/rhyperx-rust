use foldhash::fast::FixedState;
use hashbrown::{HashMap, HashSet};
use indicatif::ProgressIterator;
use rust_core::{
    iter_hyperedges,
    motifs::{
        compressed_motif::{CompactMotif, CompactMotifConfigurator},
        types::EnumerationStats,
    },
};

pub fn print_static_const<const N: usize>()
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    iter_hyperedges!(4, 1..=4, |edge, edge_size, edge_idx| {
        println!("{}: {:?}", edge_idx, &edge[0..edge_size]);
    });

    println!("Adjacencies:");
    for cm in CompactMotif::<N>::ADJ {
        println!("{:016b}", cm.container);
    }

    println!("Node Map:");
    for cm in CompactMotif::<N>::NODE_MAP {
        println!("{:016b}", cm.nodes);
    }

    println!("Full overlaps:");
    for cm in CompactMotif::<N>::FULL_OVERLAPS {
        println!("{:016b}", cm.container);
    }
    println!("Part overlaps:");
    for cm in CompactMotif::<N>::PART_OVERLAPS {
        println!("{:016b}", cm.container);
    }
    println!("Edge filter bitmask");
    for cm in CompactMotif::<N>::EDGE_FILTER_BITMASK {
        println!("{:016b}", cm.container);
    }

    println!("Relabeling map");
    for cm in CompactMotif::<N>::RELABELING_MAP {
        println!("{:?}", cm);
    }

    println!("Inclusion map");
    for cm in CompactMotif::<N>::INCLUSION_MAP {
        println!("{:016b}", cm.container);
    }
}

pub fn print_hyperedges<const N: usize>() {
    iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
        let mut v = Vec::new();
        for i in 0..edge_size {
            v.push(edge[i]);
        }
        println!("Edge {}: {:?}", edge_idx, v);
    });
}

pub fn compute_all_fingerprints<const N: usize>(
    show_progress: bool,
) -> Result<EnumerationStats, Box<dyn std::error::Error>>
where
    CompactMotif<N>: CompactMotifConfigurator,
{
    let mut map = HashMap::with_hasher(FixedState::default());
    let time = std::time::Instant::now();

    let mut total_count = 0;
    let mut connected_count = 0;
    println!("Enumerating motifs and computing fingerprints...");

    let iter: Box<dyn Iterator<Item = CompactMotif<N>>> = if show_progress {
        Box::new(CompactMotif::<N>::enum_motifs(2..=N).progress())
    } else {
        Box::new(CompactMotif::<N>::enum_motifs(2..=N))
    };
    for m in iter {
        if m.is_connected() {
            let fingerprint = m.fingerprint();
            if !map.contains_key(&fingerprint) {
                map.insert(m.fingerprint(), vec![]);
            }
            map.get_mut(&fingerprint).unwrap().push(m);

            if N != 5 {
                // TODO! remove this once we can compute canonical rep for order 5
                let x = fingerprint.clone();
                let y = fingerprint.clone().into().fingerprint();
                assert!(x == y, "expected {}\ngot      {}", m, fingerprint.into());
            }
            connected_count += 1;
        }
        total_count += 1;
    }
    let elapsed_time = time.elapsed();

    println!("Aggregating results and checking for clashing buckets...");
    let mut clashing_buckets = Vec::new();

    let iter: Box<
        dyn Iterator<
            Item = (
                &<CompactMotif<N> as CompactMotifConfigurator>::FingerprintType,
                &Vec<CompactMotif<N>>,
            ),
        >,
    > = if show_progress {
        Box::new(map.iter().progress())
    } else {
        Box::new(map.iter())
    };
    for (fingerprint, motifs) in iter {
        let mut unique_motifs = HashSet::new();
        for motif in motifs {
            unique_motifs.insert(*motif);
        }

        let mut isomorphism = HashSet::new();
        motifs[0].enum_isomorphism(|iso| {
            isomorphism.insert(iso);
        });

        if isomorphism != unique_motifs {
            clashing_buckets.push((fingerprint, motifs));
        }
    }

    let rv = EnumerationStats {
        total_count,
        connected_count,
        elapsed_time,
        distinct_fingerprints: map.len(),
        clashing_buckets_count: clashing_buckets.len(),
    };

    Ok(rv)
}
