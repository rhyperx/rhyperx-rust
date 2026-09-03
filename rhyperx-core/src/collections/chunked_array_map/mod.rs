///! A cache-friendly hash map designed for storing collections of fixed-size arrays (chunks).
///!
///! Both implementations provide O(1) hash-based lookups with SIMD probing, while avoiding
///! per-chunk allocations. They do this by flattening the keys into a single contiguous
///! `Vec<K>` and keeping values in a parallel `Vec<V>`. This layout maximizes CPU cache
///! utilization and minimizes memory fragmentation, ideal for hypergraph edges, coordinate
///! vectors, or fixed-size feature sets.
///!
pub mod non_typed;
pub mod typed;
