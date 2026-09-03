use crate::collections::chunked_array_map::non_typed::{
    ChunkedArrayMap as NonTypedChunkedArrayMap, ChunkedArraySet,
};
use crate::collections::chunked_array_map::typed::{
    ChunkedArrayMap as TypedChunkedArrayMap, ChunkedArraySet as TypedChunkedArraySet,
};

// =============================================================================
// Tests for non_typed::ChunkedArraySet
// =============================================================================

#[test]
fn test_nontyped_set_new_and_basic_insert() {
    let mut set = ChunkedArraySet::new(3);
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);

    let item = [1, 2, 3];
    assert!(set.insert(&item, ()).is_none()); // New insertion returns None
    assert!(!set.is_empty());
    assert_eq!(set.len(), 1);
    assert!(set.get(&item).is_some()); // Check presence via get
}

#[test]
fn test_nontyped_set_duplicates() {
    let mut set = ChunkedArraySet::new(2);
    let item = [10, 20];

    assert!(set.insert(&item, ()).is_none());
    assert_eq!(set.len(), 1);

    // Inserting the exact same item should return Some(()) and not inflate size
    assert!(set.insert(&item, ()).is_some());
    assert_eq!(set.len(), 1);
    assert!(set.get(&item).is_some());
}

#[test]
fn test_nontyped_set_empty_operations() {
    let set = ChunkedArraySet::<i32>::new(4);
    let item = [1, 2, 3, 4];

    assert!(set.get(&item).is_none());

    let mut set_mut = ChunkedArraySet::<i32>::new(4);
    assert!(set_mut.remove(&item).is_none());
}

#[test]
fn test_nontyped_set_remove_only_element() {
    let mut set = ChunkedArraySet::new(2);
    let item = [99, 88];

    set.insert(&item, ());
    assert_eq!(set.len(), 1);

    assert!(set.remove(&item).is_some());
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
    assert!(set.get(&item).is_none());
}

#[test]
fn test_nontyped_set_remove_nonexistent() {
    let mut set = ChunkedArraySet::new(2);
    set.insert(&[1, 2], ());

    // Removing something not in the set
    assert!(set.remove(&[3, 4]).is_none());
    assert_eq!(set.len(), 1);
}

#[test]
fn test_nontyped_set_remove_middle_element_swap_logic() {
    // Inserts 3 elements. Removing the middle one triggers the swap-and-pop logic
    // where the last element gets moved into the middle index slot.
    let mut set = ChunkedArraySet::new(1);
    let a = [1];
    let b = [2];
    let c = [3];

    set.insert(&a, ());
    set.insert(&b, ());
    set.insert(&c, ());
    assert_eq!(set.len(), 3);

    // Remove the middle element 'b'
    assert!(set.remove(&b).is_some());
    assert_eq!(set.len(), 2);

    // Ensure 'b' is gone, but 'a' and 'c' remain discoverable and correct
    assert!(set.get(&b).is_none());
    assert!(set.get(&a).is_some());
    assert!(set.get(&c).is_some());
}

#[test]
fn test_nontyped_set_remove_last_element() {
    let mut set = ChunkedArraySet::new(1);
    let a = [10];
    let b = [20];

    set.insert(&a, ());
    set.insert(&b, ());

    // Removing the last inserted element
    assert!(set.remove(&b).is_some());
    assert_eq!(set.len(), 1);
    assert!(set.get(&a).is_some());
    assert!(set.get(&b).is_none());
}

#[test]
fn test_nontyped_set_multiple_removals_and_re_insertions() {
    let mut set = ChunkedArraySet::new(2);
    let items: Vec<[usize; 2]> = (0..20).map(|i| [i, i + 1]).collect();

    // Insert all
    for item in &items {
        assert!(set.insert(item, ()).is_none());
    }
    assert_eq!(set.len(), 20);

    // Remove even-indexed items
    for (idx, item) in items.iter().enumerate() {
        if idx % 2 == 0 {
            assert!(set.remove(item).is_some());
        }
    }
    assert_eq!(set.len(), 10);

    // Verify state
    for (idx, item) in items.iter().enumerate() {
        if idx % 2 == 0 {
            assert!(set.get(item).is_none());
        } else {
            assert!(set.get(item).is_some());
        }
    }

    // Re-insert removed items
    for (idx, item) in items.iter().enumerate() {
        if idx % 2 == 0 {
            assert!(set.insert(item, ()).is_none());
        }
    }
    assert_eq!(set.len(), 20);

    // Verify all are present again
    for item in &items {
        assert!(set.get(item).is_some());
    }
}

#[test]
fn test_nontyped_set_with_capacity() {
    let set: ChunkedArraySet<i32> = ChunkedArraySet::with_capacity(3, 100);
    assert!(set.is_empty());
    assert_eq!(set.chunk_size(), 3);
}

#[test]
#[should_panic(expected = "chunk_size must be greater than 0")]
fn test_nontyped_set_zero_chunk_size_new_panics() {
    let _: ChunkedArraySet<i32> = ChunkedArraySet::new(0);
}

#[test]
#[should_panic(expected = "chunk_size must be greater than 0")]
fn test_nontyped_set_zero_chunk_size_capacity_panics() {
    let _: ChunkedArraySet<i32> = ChunkedArraySet::with_capacity(0, 50);
}

#[test]
fn test_nontyped_set_sequential_drains() {
    let mut set = ChunkedArraySet::new(1);
    let items: Vec<[usize; 1]> = (0..50).map(|i| [i]).collect();

    for item in &items {
        set.insert(item, ());
    }

    // Remove them one by one in reverse order
    for item in items.iter().rev() {
        assert!(set.remove(item).is_some());
    }

    assert!(set.is_empty());
}

#[test]
fn test_nontyped_set_clear() {
    let mut set = ChunkedArraySet::new(2);
    set.insert(&[0, 1], ());
    set.insert(&[2, 3], ());
    set.clear();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
}

#[test]
fn test_nontyped_set_iter() {
    let mut set = ChunkedArraySet::new(2);
    set.insert(&[0, 1], ());
    set.insert(&[2, 3], ());
    set.insert(&[4, 5], ());
    let collected: Vec<_> = set.iter().collect();
    assert_eq!(collected.len(), 3);
}

#[test]
fn test_nontyped_set_drain() {
    let mut set = ChunkedArraySet::new(2);
    set.insert(&[0, 1], ());
    set.insert(&[2, 3], ());
    let drained: Vec<_> = set.drain().collect();
    assert_eq!(drained.len(), 2);
    assert!(set.is_empty());
}

#[test]
fn test_nontyped_set_retain() {
    let mut set = ChunkedArraySet::new(2);
    set.insert(&[0, 1], ());
    set.insert(&[2, 3], ());
    set.insert(&[1, 2], ());
    // Retain only edges containing 1
    set.retain(|edge, _| edge.iter().any(|&v| v == 1));
    assert_eq!(set.len(), 2);
    assert!(set.get(&[0, 1]).is_some());
    assert!(set.get(&[1, 2]).is_some());
    assert!(set.get(&[2, 3]).is_none());
}

#[test]
fn test_nontyped_set_contains_key() {
    let mut set = ChunkedArraySet::new(3);
    set.insert(&[0, 1, 2], ());
    assert!(set.contains_key(&[0, 1, 2]));
    assert!(!set.contains_key(&[3, 4, 5]));
}

// =============================================================================
// Tests for non_typed::ChunkedArrayMap
// =============================================================================

#[test]
fn test_nontyped_map_basic_operations() {
    let mut map = NonTypedChunkedArrayMap::new(2);
    assert!(map.is_empty());

    // Insert and retrieve
    assert_eq!(map.insert(&[1, 2], "a"), None);
    assert_eq!(map.get(&[1, 2]), Some(&"a"));

    // Update existing key
    assert_eq!(map.insert(&[1, 2], "b"), Some("a"));
    assert_eq!(map.get(&[1, 2]), Some(&"b"));

    // Non-existent key
    assert_eq!(map.get(&[3, 4]), None);
}

#[test]
fn test_nontyped_map_get_mut() {
    let mut map = NonTypedChunkedArrayMap::new(2);
    map.insert(&[1, 2], 10);

    if let Some(v) = map.get_mut(&[1, 2]) {
        *v = 20;
    }

    assert_eq!(map.get(&[1, 2]), Some(&20));
}

#[test]
fn test_nontyped_map_remove() {
    let mut map = NonTypedChunkedArrayMap::new(2);
    map.insert(&[1, 2], "a");
    map.insert(&[3, 4], "b");

    assert_eq!(map.remove(&[1, 2]), Some("a"));
    assert_eq!(map.get(&[1, 2]), None);
    assert_eq!(map.len(), 1);
}

#[test]
fn test_nontyped_map_iter() {
    let mut map = NonTypedChunkedArrayMap::new(2);
    map.insert(&[1, 2], "a");
    map.insert(&[3, 4], "b");

    let mut values: Vec<&str> = map.iter().map(|(_, &v)| v).collect();
    values.sort();
    assert_eq!(values, vec!["a", "b"]);
}

#[test]
fn test_nontyped_map_iter_mut() {
    let mut map = NonTypedChunkedArrayMap::new(2);
    map.insert(&[1, 2], 10);
    map.insert(&[3, 4], 20);

    for (_, v) in map.iter_mut() {
        *v *= 2;
    }

    assert_eq!(map.get(&[1, 2]), Some(&20));
    assert_eq!(map.get(&[3, 4]), Some(&40));
}

#[test]
fn test_nontyped_map_drain() {
    let mut map = NonTypedChunkedArrayMap::new(2);
    map.insert(&[1, 2], "a");
    map.insert(&[3, 4], "b");

    let drained: Vec<_> = map.drain().collect();
    assert_eq!(drained.len(), 2);
    assert!(map.is_empty());
}

#[test]
fn test_nontyped_map_retain() {
    let mut map = NonTypedChunkedArrayMap::new(2);
    map.insert(&[1, 2], 10);
    map.insert(&[3, 4], 20);
    map.insert(&[5, 6], 30);

    map.retain(|_, v| *v > 15);

    assert_eq!(map.len(), 2);
    assert!(map.get(&[3, 4]).is_some());
    assert!(map.get(&[5, 6]).is_some());
    assert!(map.get(&[1, 2]).is_none());
}

#[test]
fn test_nontyped_map_with_capacity() {
    let map: NonTypedChunkedArrayMap<i32, String> = NonTypedChunkedArrayMap::with_capacity(3, 100);
    assert_eq!(map.chunk_size(), 3);
    assert_eq!(map.capacity(), 100);
}

// =============================================================================
// Tests for typed::ChunkedArraySet
// =============================================================================

#[test]
fn test_typed_set_new_and_basic_insert() {
    let mut set: TypedChunkedArraySet<3, i32> = TypedChunkedArraySet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);

    let item = [1, 2, 3];
    assert!(set.insert(&item, ()).is_none());
    assert!(!set.is_empty());
    assert_eq!(set.len(), 1);
    assert!(set.get(&item).is_some());
}

#[test]
fn test_typed_set_duplicates() {
    let mut set: TypedChunkedArraySet<2, i32> = TypedChunkedArraySet::new();
    let item = [10, 20];

    assert!(set.insert(&item, ()).is_none());
    assert_eq!(set.len(), 1);

    // Inserting the exact same item should return Some(())
    assert!(set.insert(&item, ()).is_some());
    assert_eq!(set.len(), 1);
}

#[test]
fn test_typed_set_remove() {
    let mut set: TypedChunkedArraySet<2, i32> = TypedChunkedArraySet::new();
    set.insert(&[1, 2], ());
    set.insert(&[3, 4], ());

    assert!(set.remove(&[1, 2]).is_some());
    assert_eq!(set.len(), 1);
    assert!(set.get(&[1, 2]).is_none());
}

#[test]
fn test_typed_set_clear() {
    let mut set: TypedChunkedArraySet<2, i32> = TypedChunkedArraySet::new();
    set.insert(&[1, 2], ());
    set.insert(&[3, 4], ());
    set.clear();
    assert!(set.is_empty());
}

#[test]
fn test_typed_set_iter() {
    let mut set: TypedChunkedArraySet<2, i32> = TypedChunkedArraySet::new();
    set.insert(&[1, 2], ());
    set.insert(&[3, 4], ());

    let collected: Vec<_> = set.iter().collect();
    assert_eq!(collected.len(), 2);
}

#[test]
fn test_typed_set_retain() {
    let mut set: TypedChunkedArraySet<2, i32> = TypedChunkedArraySet::new();
    set.insert(&[1, 2], ());
    set.insert(&[3, 4], ());
    set.insert(&[5, 6], ());

    set.retain(|edge, _| edge[0] > 2);

    assert_eq!(set.len(), 2);
    assert!(set.get(&[3, 4]).is_some());
    assert!(set.get(&[5, 6]).is_some());
    assert!(set.get(&[1, 2]).is_none());
}

#[test]
fn test_typed_set_with_capacity() {
    let set: TypedChunkedArraySet<3, i32> = TypedChunkedArraySet::with_capacity(100);
    assert!(set.is_empty());
    // Note: capacity() returns the values vector capacity, which for () is usize::MAX
    // We just verify it doesn't panic and is at least the requested capacity
    assert!(set.capacity() >= 100);
}

// =============================================================================
// Tests for typed::ChunkedArrayMap
// =============================================================================

#[test]
fn test_typed_map_basic_operations() {
    let mut map: TypedChunkedArrayMap<2, i32, &str> = TypedChunkedArrayMap::new();
    assert!(map.is_empty());

    // Insert and retrieve
    assert_eq!(map.insert(&[1, 2], "a"), None);
    assert_eq!(map.get(&[1, 2]), Some(&"a"));

    // Update existing key
    assert_eq!(map.insert(&[1, 2], "b"), Some("a"));
    assert_eq!(map.get(&[1, 2]), Some(&"b"));
}

#[test]
fn test_typed_map_get_mut() {
    let mut map: TypedChunkedArrayMap<2, i32, i32> = TypedChunkedArrayMap::new();
    map.insert(&[1, 2], 10);

    if let Some(v) = map.get_mut(&[1, 2]) {
        *v = 20;
    }

    assert_eq!(map.get(&[1, 2]), Some(&20));
}

#[test]
fn test_typed_map_remove() {
    let mut map: TypedChunkedArrayMap<2, i32, &str> = TypedChunkedArrayMap::new();
    map.insert(&[1, 2], "a");
    map.insert(&[3, 4], "b");

    assert_eq!(map.remove(&[1, 2]), Some("a"));
    assert_eq!(map.get(&[1, 2]), None);
    assert_eq!(map.len(), 1);
}

#[test]
fn test_typed_map_iter() {
    let mut map: TypedChunkedArrayMap<2, i32, &str> = TypedChunkedArrayMap::new();
    map.insert(&[1, 2], "a");
    map.insert(&[3, 4], "b");

    let mut values: Vec<&str> = map.iter().map(|(_, &v)| v).collect();
    values.sort();
    assert_eq!(values, vec!["a", "b"]);
}

#[test]
fn test_typed_map_iter_mut() {
    let mut map: TypedChunkedArrayMap<2, i32, i32> = TypedChunkedArrayMap::new();
    map.insert(&[1, 2], 10);
    map.insert(&[3, 4], 20);

    for (_, v) in map.iter_mut() {
        *v *= 2;
    }

    assert_eq!(map.get(&[1, 2]), Some(&20));
    assert_eq!(map.get(&[3, 4]), Some(&40));
}

#[test]
fn test_typed_map_drain() {
    let mut map: TypedChunkedArrayMap<2, i32, &str> = TypedChunkedArrayMap::new();
    map.insert(&[1, 2], "a");
    map.insert(&[3, 4], "b");

    let drained: Vec<_> = map.drain().collect();
    assert_eq!(drained.len(), 2);
    assert!(map.is_empty());
}

#[test]
fn test_typed_map_retain() {
    let mut map: TypedChunkedArrayMap<2, i32, i32> = TypedChunkedArrayMap::new();
    map.insert(&[1, 2], 10);
    map.insert(&[3, 4], 20);
    map.insert(&[5, 6], 30);

    map.retain(|_, v| *v > 15);

    assert_eq!(map.len(), 2);
    assert!(map.get(&[3, 4]).is_some());
    assert!(map.get(&[5, 6]).is_some());
    assert!(map.get(&[1, 2]).is_none());
}

#[test]
fn test_typed_map_with_capacity() {
    let map: TypedChunkedArrayMap<3, i32, String> = TypedChunkedArrayMap::with_capacity(100);
    assert_eq!(map.capacity(), 100);
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn test_large_chunk_size() {
    let mut set = ChunkedArraySet::new(100);
    let large_key: Vec<i32> = (0..100).collect();
    set.insert(&large_key, ());
    assert_eq!(set.len(), 1);
    assert!(set.get(&large_key).is_some());
}

#[test]
fn test_typed_large_chunk_size() {
    let mut set: TypedChunkedArraySet<10, i32> = TypedChunkedArraySet::new();
    let key = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    set.insert(&key, ());
    assert_eq!(set.len(), 1);
    assert!(set.get(&key).is_some());
}
