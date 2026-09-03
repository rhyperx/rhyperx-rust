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

use crate::collections::chunked_array_map::non_typed::ChunkedArrayMap;

/// The archived form of [`ChunkedArrayMap`].
///
/// Mirroring the live layout, `keys` is kept as one flat [`ArchivedVec`]
/// (`chunk_size` keys per logical entry, in row-major order) and `values`
/// as the parallel [`ArchivedVec`]. The Swiss-table lookup structure is
/// stored as an [`ArchivedHashTable`] whose payload is the archived index
/// into those vecs, so no new indirection or per-entry allocation is
/// introduced by archiving.
#[derive(Portable, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck)]
#[repr(C)]
pub struct ArchivedChunkedArrayMap<K, V> {
    pub chunk_size: ArchivedUsize,
    pub keys: ArchivedVec<K>,
    pub values: ArchivedVec<V>,
    pub table: ArchivedHashTable<ArchivedUsize>,
}

impl<K, V> ArchivedChunkedArrayMap<K, V> {
    /// Returns the chunk size of the archived map.
    #[inline]
    pub fn chunk_size(&self) -> usize {
        self.chunk_size.to_native() as usize
    }

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
    /// so it must be a chunk of exactly `chunk_size()` elements.
    pub fn get(&self, key: &[K]) -> Option<&V>
    where
        K: Hash + Eq,
    {
        let chunk_size = self.chunk_size();
        debug_assert_eq!(key.len(), chunk_size);
        let hasher = FixedState::with_seed(0);
        let hash = hasher.hash_one(key);
        self.table
            .get_with(hash, |&entry_idx| {
                let idx = entry_idx.to_native() as usize;
                let start = idx * chunk_size;
                &self.keys.as_slice()[start..start + chunk_size] == key
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

impl<K, V> Archive for ChunkedArrayMap<K, V>
where
    K: Archive + Hash + Eq,
    K::Archived: Hash + Eq,
    V: Archive,
{
    type Archived = ArchivedChunkedArrayMap<K::Archived, V::Archived>;
    type Resolver = ChunkedArrayMapResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        munge!(let ArchivedChunkedArrayMap {
                chunk_size,
                keys,
                values,
                table,
            } = out);

        usize::resolve(&self.chunk_size, (), chunk_size);
        ArchivedVec::resolve_from_len(self.keys.len(), resolver.keys, keys);
        ArchivedVec::resolve_from_len(self.values.len(), resolver.values, values);
        ArchivedHashTable::resolve_from_len(self.values.len(), (7, 8), resolver.table, table);
    }
}

impl<K, V, S> Serialize<S> for ChunkedArrayMap<K, V>
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

        let chunk_size = self.chunk_size;
        let hasher = FixedState::with_seed(0);

        // The hash table contains exactly the indices `0..len`. We re-derive
        // each index together with the hash of its key chunk straight off
        // the contiguous keys buffer: two cheap `ExactSizeIterator`s, no
        // heap allocations, and no dependence on hashbrown's internal
        // iteration order.
        let indices = 0..self.values.len();
        let hashes = indices.clone().map(|index| {
            let start = index * chunk_size;
            hasher.hash_one(&self.keys[start..start + chunk_size])
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

impl<K, V, D> Deserialize<ChunkedArrayMap<K, V>, D>
    for ArchivedChunkedArrayMap<K::Archived, V::Archived>
where
    K: Archive + Hash + Eq,
    K::Archived: Hash + Eq + Deserialize<K, D>,
    V: Archive,
    V::Archived: Deserialize<V, D>,
    D: Fallible + ?Sized,
    D::Error: Source,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<ChunkedArrayMap<K, V>, D::Error> {
        let chunk_size = self.chunk_size.to_native() as usize;
        debug_assert!(chunk_size > 0);

        let keys: Vec<K> = self.keys.deserialize(deserializer)?;
        let values: Vec<V> = self.values.deserialize(deserializer)?;

        debug_assert_eq!(keys.len(), values.len() * chunk_size);

        let hasher = FixedState::with_seed(0);
        let mut table = HashTable::with_capacity(self.len());
        for index in 0..self.len() {
            let start = index * chunk_size;
            let hash = hasher.hash_one(&keys[start..start + chunk_size]);
            table.insert_unique(hash, index, |&entry_idx| {
                let s = entry_idx * chunk_size;
                hasher.hash_one(&keys[s..s + chunk_size])
            });
        }

        Ok(ChunkedArrayMap {
            chunk_size,
            keys,
            values,
            table,
        })
    }
}
