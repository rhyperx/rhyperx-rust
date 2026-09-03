///! The `typed` module provides `ChunkedArrayMap` with a compile-time fixed chunk size.
///!
///! By leveraging const generics (`const N: usize`), this version completely eliminates
///! runtime length checks and ensures absolute type safety. Inputs must be sized as
///! `&[K; N]` or `[K; N]`, catching length mismatches at compile time rather than
///! producing panics or silent misses at runtime.
use foldhash::quality::FixedState;
use hashbrown::HashTable;
use std::hash::{BuildHasher, Hash};

/// The `typed` module provides `ChunkedArrayMap` with a compile-time fixed chunk size.
///
/// By leveraging const generics (`const N: usize`), this version completely eliminates
/// runtime length checks and ensures absolute type safety. Inputs must be sized as
/// `&[K; N]` or `[K; N]`, catching length mismatches at compile time rather than
/// producing panics or silent misses at runtime.

/// A cache-friendly hash map mapping compile-time fixed-size key arrays to values.
///
/// # Example
/// ```
/// use rhyperx_core::collections::chunked_array_map::typed::ChunkedArrayMap;
///
/// let mut map: ChunkedArrayMap<3, i32, &str> = ChunkedArrayMap::new();
/// map.insert(&[0, 1, 2], "first");
/// assert_eq!(map.get(&[0, 1, 2]), Some(&"first"));
/// ```
#[derive(Clone)]
pub struct ChunkedArrayMap<const N: usize, K, V> {
    pub(crate) keys: Vec<K>,
    pub(crate) values: Vec<V>,
    pub(crate) table: HashTable<usize>,
}

/// A zero-cost abstraction over `ChunkedArrayMap` for sets.
pub type ChunkedArraySet<const N: usize, K> = ChunkedArrayMap<N, K, ()>;

impl<const N: usize, K, V> Default for ChunkedArrayMap<N, K, V>
where
    K: Hash + Eq + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, K, V> ChunkedArrayMap<N, K, V>
where
    K: Hash + Eq + Clone,
{
    const HASHER_BUILDER: FixedState = FixedState::with_seed(0);

    /// Creates a new empty map.
    pub fn new() -> Self {
        assert!(N > 0, "chunk size N must be greater than 0");
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            table: HashTable::new(),
        }
    }

    /// Creates a new map with a specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(N > 0, "chunk size N must be greater than 0");
        Self {
            keys: Vec::with_capacity(capacity * N),
            values: Vec::with_capacity(capacity),
            table: HashTable::with_capacity(capacity),
        }
    }

    #[inline(always)]
    fn hash_slice(slice: &[K]) -> u64 {
        Self::HASHER_BUILDER.hash_one(slice)
    }

    #[inline(always)]
    fn get_chunk(keys: &[K], entry_idx: usize) -> &[K; N] {
        let start = entry_idx * N;
        (&keys[start..start + N]).try_into().unwrap()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Checks if a key chunk is present in the map.
    pub fn contains_key(&self, key: &[K; N]) -> bool {
        let hash = Self::hash_slice(key);
        self.table
            .find(hash, |&entry_idx| {
                Self::get_chunk(&self.keys, entry_idx) == key
            })
            .is_some()
    }

    /// Retrieves a reference to the value corresponding to the key.
    pub fn get(&self, key: &[K; N]) -> Option<&V> {
        let hash = Self::hash_slice(key);
        self.table
            .find(hash, |&entry_idx| {
                Self::get_chunk(&self.keys, entry_idx) == key
            })
            .map(|&entry_idx| &self.values[entry_idx])
    }

    /// Retrieves a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: &[K; N]) -> Option<&mut V> {
        let hash = Self::hash_slice(key);
        self.table
            .find(hash, |&entry_idx| {
                Self::get_chunk(&self.keys, entry_idx) == key
            })
            .map(|&entry_idx| &mut self.values[entry_idx])
    }

    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, key: &[K; N], value: V) -> Option<V> {
        let hash = Self::hash_slice(key);

        if let Some(&entry_idx) = self
            .table
            .find(hash, |&idx| Self::get_chunk(&self.keys, idx) == key)
        {
            return Some(std::mem::replace(&mut self.values[entry_idx], value));
        }

        let new_entry_idx = self.values.len();
        self.keys.extend_from_slice(key);
        self.values.push(value);

        self.table.insert_unique(hash, new_entry_idx, |&idx| {
            let start = idx * N;
            Self::hash_slice(&self.keys[start..start + N])
        });

        None
    }

    /// Removes a key from the map via swap-remove, returning the value if found.
    pub fn remove(&mut self, key: &[K; N]) -> Option<V> {
        let target_hash = Self::hash_slice(key);

        let target_entry = match self.table.find_entry(target_hash, |&entry_idx| {
            Self::get_chunk(&self.keys, entry_idx) == key
        }) {
            Ok(entry) => entry,
            Err(_) => return None,
        };

        let target_idx = *target_entry.get();
        target_entry.remove();

        let last_chunk_idx = self.values.len() - 1;

        if target_idx != last_chunk_idx {
            let src_start = last_chunk_idx * N;
            let dst_start = target_idx * N;

            let moved_hash = Self::hash_slice(&self.keys[src_start..src_start + N]);

            let mut entry = self
                .table
                .find_entry(moved_hash, |&idx| idx == last_chunk_idx)
                .expect("Moved chunk must exist in the hash table");
            *entry.get_mut() = target_idx;

            for i in 0..N {
                self.keys.swap(dst_start + i, src_start + i);
            }
        }

        self.keys.truncate(last_chunk_idx * N);
        Some(self.values.swap_remove(target_idx))
    }

    /// Clears the map while retaining capacity.
    pub fn clear(&mut self) {
        self.table.clear();
        self.keys.clear();
        self.values.clear();
    }

    /// Yields references to all key chunks and their values.
    pub fn iter(&self) -> Iter<'_, N, K, V> {
        Iter {
            keys: &self.keys,
            values: self.values.iter(),
            current_idx: 0,
        }
    }

    /// Yields references to all key chunks and mutable references to their values.
    pub fn iter_mut(&mut self) -> IterMut<'_, N, K, V> {
        IterMut {
            keys: &self.keys,
            values: self.values.iter_mut(),
            current_idx: 0,
        }
    }

    /// Drains all elements, yielding the key chunk and the associated value.
    pub fn drain(&mut self) -> DrainIterator<'_, N, K, V> {
        self.table.clear();
        DrainIterator {
            keys: &mut self.keys,
            values: &mut self.values,
        }
    }

    /// Retains only the elements matching a predicate.
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&[K; N], &mut V) -> bool,
    {
        let mut write_idx = 0;

        for read_idx in 0..self.values.len() {
            let start = read_idx * N;
            let chunk: &[K; N] = (&self.keys[start..start + N]).try_into().unwrap();
            let value = &mut self.values[read_idx];

            if predicate(chunk, value) {
                if write_idx != read_idx {
                    let write_start = write_idx * N;
                    for i in 0..N {
                        self.keys.swap(write_start + i, start + i);
                    }
                    self.values.swap(write_idx, read_idx);
                }
                write_idx += 1;
            }
        }

        self.keys.truncate(write_idx * N);
        self.values.truncate(write_idx);

        self.table.clear();
        for i in 0..write_idx {
            let start = i * N;
            let hash = Self::hash_slice(&self.keys[start..start + N]);
            self.table.insert_unique(hash, i, |&idx| {
                let s = idx * N;
                Self::hash_slice(&self.keys[s..s + N])
            });
        }
    }
}

pub struct Iter<'a, const N: usize, K, V> {
    keys: &'a [K],
    values: std::slice::Iter<'a, V>,
    current_idx: usize,
}

impl<'a, const N: usize, K, V> Iterator for Iter<'a, N, K, V> {
    type Item = (&'a [K; N], &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.values.next()?;
        let start = self.current_idx * N;
        let chunk = (&self.keys[start..start + N]).try_into().unwrap();
        self.current_idx += 1;
        Some((chunk, value))
    }
}

pub struct IterMut<'a, const N: usize, K, V> {
    keys: &'a [K],
    values: std::slice::IterMut<'a, V>,
    current_idx: usize,
}

impl<'a, const N: usize, K, V> Iterator for IterMut<'a, N, K, V> {
    type Item = (&'a [K; N], &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.values.next()?;
        let start = self.current_idx * N;
        let chunk = (&self.keys[start..start + N]).try_into().unwrap();
        self.current_idx += 1;
        Some((chunk, value))
    }
}

pub struct DrainIterator<'a, const N: usize, K, V> {
    keys: &'a mut Vec<K>,
    values: &'a mut Vec<V>,
}

impl<'a, const N: usize, K, V> Iterator for DrainIterator<'a, N, K, V> {
    type Item = ([K; N], V);

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.values.pop()?;
        let start = self.keys.len() - N;
        let chunk_vec = self.keys.split_off(start);
        let chunk: [K; N] = chunk_vec.try_into().unwrap_or_else(|_| unreachable!());
        Some((chunk, value))
    }
}
