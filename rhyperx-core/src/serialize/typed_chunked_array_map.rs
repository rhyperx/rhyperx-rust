use foldhash::quality::FixedState;
use hashbrown::HashTable;
use rkyv::bytecheck::CheckBytes;
use rkyv::{
    Archive, Deserialize, Place, Portable, Serialize,
    collections::swiss_table::{ArchivedHashTable, HashTableResolver},
    munge::munge,
    primitive::ArchivedUsize,
    rancor::{Fallible, Source},
    ser::{Allocator, Writer},
    vec::{ArchivedVec, VecResolver},
};
use std::hash::{BuildHasher, Hash};

use crate::collections::chunked_array_map::typed::ChunkedArrayMap;

/// The archived form of [`ChunkedArrayMap`].
///
/// The chunk size is a compile-time constant (`N`), so unlike the
/// non-typed variant nothing needs to be stored for it. `keys` stays a
/// single flat [`ArchivedVec`] (one row of `N` archived keys per logical
/// entry), `values` the parallel [`ArchivedVec`], and the lookup structure
/// an [`ArchivedHashTable`] of archived indices.
#[derive(Portable, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedChunkedArrayMap<K, V> {
    pub keys: ArchivedVec<K>,
    pub values: ArchivedVec<V>,
    pub table: ArchivedHashTable<ArchivedUsize>,
}

impl<K: Hash + Eq, V> ArchivedChunkedArrayMap<K, V> {
    /// Returns the number of entries in the archived map.
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether the archived map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Retrieves a reference to the value corresponding to the key chunk.
    ///
    /// The archived key slice is hashed with the same seed as the live map,
    /// so it must be a slice of exactly `N` elements.
    pub fn get<const N: usize>(&self, key: &[K; N]) -> Option<&V> {
        let hasher = FixedState::with_seed(0);
        let hash = hasher.hash_one(key.as_slice());
        self.table
            .get_with(hash, |&entry_idx| {
                let idx = entry_idx.to_native() as usize;
                let start = idx * N;
                &self.keys.as_slice()[start..start + N] == key.as_slice()
            })
            .map(|&entry_idx| {
                let idx = entry_idx.to_native() as usize;
                &self.values.as_slice()[idx]
            })
    }
}

/// The resolver for [`ArchivedChunkedArrayMap`].
pub struct ChunkedArrayMapResolver {
    keys: VecResolver,
    values: VecResolver,
    table: HashTableResolver,
}

impl<const N: usize, K, V> Archive for ChunkedArrayMap<N, K, V>
where
    K: Archive + Hash + Eq,
    K::Archived: Hash + Eq,
    V: Archive,
{
    type Archived = ArchivedChunkedArrayMap<K::Archived, V::Archived>;
    type Resolver = ChunkedArrayMapResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        munge!(let ArchivedChunkedArrayMap {
                keys,
                values,
                table,
            } = out);

        ArchivedVec::resolve_from_len(self.keys.len(), resolver.keys, keys);
        ArchivedVec::resolve_from_len(self.values.len(), resolver.values, values);
        ArchivedHashTable::resolve_from_len(self.values.len(), (7, 8), resolver.table, table);
    }
}

impl<const N: usize, K, V, S> Serialize<S> for ChunkedArrayMap<N, K, V>
where
    K: Archive + Hash + Eq + Serialize<S>,
    K::Archived: Hash + Eq,
    V: Archive + Serialize<S>,
    S: Fallible + Writer + Allocator + ?Sized,
    S::Error: Source,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let keys = ArchivedVec::<K::Archived>::serialize_from_slice(&self.keys, serializer)?;
        let values = ArchivedVec::<V::Archived>::serialize_from_slice(&self.values, serializer)?;

        let hasher = FixedState::with_seed(0);

        // The hash table contains exactly the indices `0..len`; re-derive
        // them together with the hash of each key chunk straight off the
        // contiguous keys buffer (no heap allocations, no dependence on
        // hashbrown's internal iteration order).
        let indices = 0..self.values.len();
        let hashes = indices.clone().map(|index| {
            let start = index * N;
            hasher.hash_one(&self.keys[start..start + N])
        });

        let table = ArchivedHashTable::<ArchivedUsize>::serialize_from_iter(
            indices,
            hashes,
            (7, 8),
            serializer,
        )?;

        Ok(ChunkedArrayMapResolver {
            keys,
            values,
            table,
        })
    }
}

impl<const N: usize, K, V, D> Deserialize<ChunkedArrayMap<N, K, V>, D>
    for ArchivedChunkedArrayMap<K::Archived, V::Archived>
where
    K: Archive + Hash + Eq,
    K::Archived: Hash + Eq + Deserialize<K, D>,
    V: Archive,
    V::Archived: Deserialize<V, D>,
    D: Fallible + ?Sized,
    D::Error: Source,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<ChunkedArrayMap<N, K, V>, D::Error> {
        let keys: Vec<K> = self.keys.deserialize(deserializer)?;
        let values: Vec<V> = self.values.deserialize(deserializer)?;

        debug_assert_eq!(keys.len(), values.len() * N);

        let hasher = FixedState::with_seed(0);
        let mut table = HashTable::with_capacity(self.len());
        for index in 0..self.len() {
            let start = index * N;
            let hash = hasher.hash_one(&keys[start..start + N]);
            table.insert_unique(hash, index, |&entry_idx| {
                let s = entry_idx * N;
                hasher.hash_one(&keys[s..s + N])
            });
        }

        Ok(ChunkedArrayMap {
            keys,
            values,
            table,
        })
    }
}
