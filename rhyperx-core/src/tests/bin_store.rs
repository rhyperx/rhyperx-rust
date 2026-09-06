use crate::collections::BinStore;
use crate::compact_motif;
use crate::motif::CompactMotif;

/// Helper trait so tests can enumerate set bits of any `BinStore` word type.
trait BinStoreBits {
    fn bits_set(&self) -> Vec<usize>;
}

macro_rules! impl_bin_store_bits {
    ($t:ty) => {
        impl<const N: usize> BinStoreBits for BinStore<$t, N> {
            fn bits_set(&self) -> Vec<usize> {
                (0..N * (<$t>::BITS as usize))
                    .filter(|&i| self.get_bit(i))
                    .collect()
            }
        }
    };
}

impl_bin_store_bits!(u8);
impl_bin_store_bits!(u16);
impl_bin_store_bits!(u32);
impl_bin_store_bits!(u64);
impl_bin_store_bits!(u128);

#[test]
fn set_range_single_word_middle() {
    let mut bs = BinStore::<u8, 1>::ZERO;
    bs.set_range(2, 6);
    assert_eq!(bs.bits_set(), vec![2, 3, 4, 5]);
}

#[test]
fn set_range_single_word_full() {
    let mut bs = BinStore::<u8, 1>::ZERO;
    bs.set_range(0, 8);
    assert_eq!(bs.bits_set(), (0..8).collect::<Vec<_>>());
}

#[test]
fn set_range_start_on_word_boundary() {
    let mut bs = BinStore::<u8, 2>::ZERO;
    bs.set_range(8, 16);
    assert_eq!(bs.bits_set(), (8..16).collect::<Vec<_>>());
}

#[test]
fn set_range_across_words() {
    let mut bs = BinStore::<u8, 2>::ZERO;
    bs.set_range(4, 13);
    assert_eq!(bs.bits_set(), (4..13).collect::<Vec<_>>());
}

#[test]
fn set_range_u32_full_store() {
    let mut bs = BinStore::<u32, 1>::ZERO;
    bs.set_range(0, 32);
    assert_eq!(bs.count_ones(), 32);
}

#[test]
fn set_range_end_beyond_capacity_is_clamped() {
    let mut bs = BinStore::<u8, 1>::ZERO;
    bs.set_range(3, 40);
    assert_eq!(bs.bits_set(), (3..8).collect::<Vec<_>>());
}

#[test]
fn set_range_start_beyond_capacity_is_noop() {
    let mut bs = BinStore::<u8, 1>::ZERO;
    bs.set_range(10, 12);
    assert!(bs.is_empty());
}

#[test]
fn set_range_empty_and_inverted_ranges_are_noop() {
    let mut bs = BinStore::<u8, 1>::ZERO;
    bs.set_range(3, 3);
    bs.set_range(5, 2);
    assert!(bs.is_empty());
}

#[test]
fn set_range_preserves_existing_bits_outside_range() {
    let mut bs = BinStore::<u16, 1>::ZERO;
    bs.set_bit(1);
    bs.set_bit(14);
    bs.set_range(4, 9);
    assert_eq!(bs.bits_set(), vec![1, 4, 5, 6, 7, 8, 14]);
}

// `set_range` is used in const contexts (e.g. `Fingerprint5::GROUP_ID_ADJ`);
// make sure none of the edge cases overflow in const evaluation.
const _SET_RANGE_CONST_CHECK: () = {
    let mut bs = BinStore::<u32, 1>::ZERO;
    bs.set_range(0, 32);
    bs.set_range(0, 40);
    bs.set_range(16, 16);
    bs.set_range(40, 50);
    assert!(bs.count_ones() == 32);
};

#[test]
fn retain_ids_in_range_filters_edges() {
    let mut m: CompactMotif!(3) = compact_motif!(3);
    for e in [0usize, 1, 3, 4] {
        m.add_edge(e);
    }

    let mut filtered = m;
    filtered.retain_ids_in_range(1, 4);
    assert_eq!(filtered.iter_edges_ids().collect::<Vec<_>>(), vec![1, 3]);

    let mut filtered = m;
    filtered.retain_ids_in_range(0, 7);
    assert_eq!(
        filtered.iter_edges_ids().collect::<Vec<_>>(),
        vec![0, 1, 3, 4]
    );

    let mut filtered = m;
    filtered.retain_ids_in_range(4, 100);
    assert_eq!(filtered.iter_edges_ids().collect::<Vec<_>>(), vec![4]);

    let mut filtered = m;
    filtered.retain_ids_in_range(10, 20);
    assert!(filtered.is_empty());
}

#[test]
fn id_ordinal_counts_smaller_present_ids() {
    let mut m: CompactMotif!(3) = compact_motif!(3);
    for e in [0usize, 1, 3, 4] {
        m.add_edge(e);
    }

    // Doc example: ids [0, 1, 3, 4] → id_ordinal(3) = 2.
    assert_eq!(m.id_ordinal(3), 2);
    assert_eq!(m.id_ordinal(0), 0);
    assert_eq!(m.id_ordinal(1), 1);
    assert_eq!(m.id_ordinal(2), 2);
    assert_eq!(m.id_ordinal(4), 3);
    assert_eq!(m.id_ordinal(6), 4);
    // Out-of-range ids clamp to the store capacity.
    assert_eq!(m.id_ordinal(100), 4);

    let empty: CompactMotif!(3) = compact_motif!(3);
    assert_eq!(empty.id_ordinal(4), 0);
}

#[test]
fn id_ordinal_order5() {
    let mut m: CompactMotif!(5) = compact_motif!(5);
    // Edge id layout for N = 5: 1-edges 0..=4, 2-edges 5..=14, 3-edges 15..=24.
    m.add_edge_with_nodes([0, 1]);
    m.add_edge_with_nodes([0, 2]);
    m.add_edge_with_nodes([1, 2, 3]);
    m.add_edge_with_nodes([0, 1, 2, 3]);

    assert_eq!(m.id_ordinal(0), 0);
    // Present ids: {0,1}→5, {0,2}→6, {1,2,3}→21, {0,1,2,3}→25.
    assert_eq!(m.id_ordinal(6), 1);
    assert_eq!(m.id_ordinal(15), 2);
    assert_eq!(m.id_ordinal(20), 2);
    assert_eq!(m.id_ordinal(22), 3);
    assert_eq!(m.id_ordinal(25), 3);
    assert_eq!(m.id_ordinal(26), 4);
    assert_eq!(m.id_ordinal(31), 4);
}

// `id_ordinal` is used in const evaluation when building `Fingerprint5::GROUP_ID_ADJ`.
const _ID_ORDINAL_CONST_CHECK: () = {
    let mut m: CompactMotif!(5) = compact_motif!(5);
    m.add_edge_with_nodes([0, 1]);
    m.add_edge_with_nodes([2, 3, 4]);
    assert!(m.id_ordinal(0) == 0);
    assert!(m.id_ordinal(5) == 0);
    assert!(m.id_ordinal(6) == 1);
    assert!(m.id_ordinal(24) == 1);
    assert!(m.id_ordinal(25) == 2);
    assert!(m.id_ordinal(100) == 2);
};
