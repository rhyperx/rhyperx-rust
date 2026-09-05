use std::hash::Hash;

use foldhash::fast::FixedState;
use hashbrown::{HashMap, HashSet};
use indicatif::ProgressIterator;
use rhyperx::algo::motifs::types::EnumerationStats;
use rhyperx::motif::fingerprint::{Fingerprint, Fingerprintable};
use rhyperx::{CompactMotif, iter_hyperedges};

pub fn print_hyperedges<const N: usize>() {
    iter_hyperedges!(N, 1..=N, |edge, edge_size, edge_idx| {
        let mut v = Vec::new();
        for i in 0..edge_size {
            v.push(edge[i]);
        }
        println!("Edge {}: {:?}", edge_idx, v);
    });
}

/// Abstraction over the supported motif orders, so that fingerprint enumeration
/// stays generic while each order expands to a distinct concrete type via the
/// `CompactMotif!` macro.
pub trait MotifFamily: Copy + Eq + Hash + Fingerprintable {
    fn enum_motifs() -> Box<dyn Iterator<Item = Self>>;
    fn enum_motifs_len() -> u64;
    fn is_connected(&self) -> bool;
    fn enum_isomorphism<F: FnMut(Self)>(&self, f: F);

    /// Whether the canonical representative can be computed for this order.
    /// Order 5 fingerprints have no canonical rep yet.
    fn canonical_rep_implemented() -> bool {
        true
    }
}

impl MotifFamily for CompactMotif!(3) {
    fn enum_motifs() -> Box<dyn Iterator<Item = Self>> {
        Box::new(<CompactMotif!(3)>::enum_motifs(2..=3))
    }

    fn enum_motifs_len() -> u64 {
        2u64.pow(4)
    }

    fn is_connected(&self) -> bool {
        self.is_connected()
    }

    fn enum_isomorphism<F: FnMut(Self)>(&self, f: F) {
        self.enum_isomorphism(f)
    }
}

impl MotifFamily for CompactMotif!(4) {
    fn enum_motifs() -> Box<dyn Iterator<Item = Self>> {
        Box::new(<CompactMotif!(4)>::enum_motifs(2..=4))
    }

    fn enum_motifs_len() -> u64 {
        2u64.pow(11)
    }

    fn is_connected(&self) -> bool {
        self.is_connected()
    }

    fn enum_isomorphism<F: FnMut(Self)>(&self, f: F) {
        self.enum_isomorphism(f)
    }
}

impl MotifFamily for CompactMotif!(5) {
    fn enum_motifs() -> Box<dyn Iterator<Item = Self>> {
        Box::new(<CompactMotif!(5)>::enum_motifs(2..=5))
    }

    fn enum_motifs_len() -> u64 {
        2u64.pow(26)
    }

    fn is_connected(&self) -> bool {
        self.is_connected()
    }

    fn enum_isomorphism<F: FnMut(Self)>(&self, f: F) {
        self.enum_isomorphism(f)
    }

    fn canonical_rep_implemented() -> bool {
        false
    }
}

/// Enumerate every motif up to a given order and verify the fingerprint
/// invariant (distinct fingerprints partition the isomorphism classes).
pub fn compute_all_fingerprints<T>(
    show_progress: bool,
) -> Result<EnumerationStats, Box<dyn std::error::Error>>
where
    T: MotifFamily,
    <T as Fingerprintable>::FingerprintType: Fingerprint<MotifType = T>,
{
    let mut map: HashMap<<T as Fingerprintable>::FingerprintType, Vec<T>, FixedState> =
        HashMap::with_hasher(FixedState::default());
    let time = std::time::Instant::now();

    let mut total_count = 0;
    let mut connected_count = 0;
    println!("Enumerating motifs and computing fingerprints...");

    let iter: Box<dyn Iterator<Item = T>> = if show_progress {
        Box::new(T::enum_motifs().progress_count(T::enum_motifs_len()))
    } else {
        T::enum_motifs()
    };
    for m in iter {
        if m.is_connected() {
            let fingerprint = m.fingerprint();
            map.entry(fingerprint).or_default().push(m);

            if T::canonical_rep_implemented() {
                let canonical = fingerprint.get_canonical_rep().fingerprint();
                assert!(
                    fingerprint == canonical,
                    "fingerprint not stable under canonical relabeling"
                );
            }
            connected_count += 1;
        }
        total_count += 1;
    }
    let elapsed_time = time.elapsed();

    println!("Aggregating results and checking for clashing buckets...");
    let mut clashing_buckets = 0;

    let iter: Box<dyn Iterator<Item = &<T as Fingerprintable>::FingerprintType>> = if show_progress
    {
        Box::new(map.keys().progress())
    } else {
        Box::new(map.keys())
    };
    for fingerprint in iter {
        let motifs = &map[fingerprint];

        let mut unique_motifs = HashSet::new();
        for motif in motifs {
            unique_motifs.insert(*motif);
        }

        let mut isomorphism = HashSet::new();
        motifs[0].enum_isomorphism(|iso| {
            isomorphism.insert(iso);
        });

        if isomorphism != unique_motifs {
            clashing_buckets += 1;
        }
    }

    let rv = EnumerationStats {
        total_count,
        connected_count,
        distinct_fingerprints: map.len(),
        elapsed_time,
        clashing_buckets_count: clashing_buckets,
    };

    Ok(rv)
}
