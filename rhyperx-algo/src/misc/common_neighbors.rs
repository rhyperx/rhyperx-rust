use crate::types::NodeId;

/// Efficiently computes the common elements shared by two sorted lists.
/// Time Complexity: O(deg(u) + deg(v))
pub fn common_neighbors_sorted_list<'a, T: Eq + Ord>(a: &'a [T], b: &'a [T]) -> Vec<&'a T> {
    let mut neighbors = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            neighbors.push(&a[i]);
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            i += 1;
        } else {
            j += 1;
        }
    }
    neighbors
}

/// Efficiently computes the common elements shared by three sorted lists.
/// Time Complexity: O(deg(u) + deg(v) + deg(w))
pub fn common_neighbors_sorted_list_3<'a, T: Eq + Ord>(
    a: &'a [T],
    b: &'a [T],
    c: &'a [T],
    hint: T,
) -> Vec<&'a T> {
    let mut neighbors = Vec::new();
    let (mut i, mut j, mut k) = (0, 0, 0);

    while i < a.len() && a[i] < hint && j < b.len() && b[j] < hint && k < c.len() && c[k] < hint {
        if a[i] == b[j] && b[j] == c[k] {
            neighbors.push(&a[i]);
            i += 1;
            j += 1;
            k += 1;
        } else if a[i] < b[j] {
            i += 1;
        } else if b[j] < c[k] {
            j += 1;
        } else {
            k += 1;
        }
    }
    neighbors
}

/// Counts the number of common neighbors.
pub fn count_common_neighbors_sorted_list<T: Eq + Ord>(a: &[T], b: &[T]) -> usize {
    let mut count = 0;
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            count += 1;
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            i += 1;
        } else {
            j += 1;
        }
    }
    count
}

/// Efficiently computes the common elements shared by three sorted lists.
/// Time Complexity: O(deg(u) + deg(v) + deg(w))
pub fn count_common_neighbors_sorted_list_3<T: Eq + Ord>(
    a: &[T],
    b: &[T],
    c: &[T],
    hint: T,
) -> usize {
    let mut count = 0;
    let (mut i, mut j, mut k) = (0, 0, 0);

    while i < a.len() && a[i] < hint && j < b.len() && b[j] < hint && k < c.len() && c[k] < hint {
        if a[i] == b[j] && b[j] == c[k] {
            count += 1;
            i += 1;
            j += 1;
            k += 1;
        } else if a[i] < b[j] {
            i += 1;
        } else if b[j] < c[k] {
            j += 1;
        } else {
            k += 1;
        }
    }
    count
}

/// Executes a closure for each common neighbor found.
pub fn neighbors_sorted_list_cloj<T: Eq + Ord, F>(a: &[T], b: &[T], mut f: F)
where
    F: FnMut(&T),
{
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            f(&a[i]);
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            i += 1;
        } else {
            j += 1;
        }
    }
}

pub fn common_neighbors_sorted_list_3_cloj<T, F>(a: &[T], b: &[T], c: &[T], hint: &T, mut f: F)
where
    F: FnMut(usize, usize, usize),
    T: Eq + Ord + std::fmt::Debug,
{
    let (mut i, mut j, mut k) = (0, 0, 0);

    println!("{:?}", a);
    println!("{:?}", b);
    println!("{:?}", c);

    while i < a.len() && a[i] < *hint && j < b.len() && b[j] < *hint && k < c.len() && c[k] < *hint
    {
        if a[i] == b[j] && b[j] == c[k] {
            f(i, j, k);
            i += 1;
            j += 1;
            k += 1;
        } else {
            let max_val = (&a[i]).max(&b[j]).max(&c[k]);

            while i < a.len() && a[i] < *max_val {
                i += 1;
            }

            while j < b.len() && b[j] < *max_val {
                j += 1;
            }

            while k < c.len() && c[k] < *max_val {
                k += 1;
            }
        }
    }
}

pub fn common_neighbors_sorted_list_3_by_key<'a, 'b, T, F, G, K>(
    a: &'a [T],
    b: &'a [T],
    c: &'a [T],
    hint: &'b K,
    key: G,
    mut f: F,
) where
    'b: 'a,
    G: Fn(&'a T) -> &'b K,
    F: FnMut(usize, usize, usize),
    K: Eq + Ord,
{
    let (mut i, mut j, mut k) = (0, 0, 0);

    while i < a.len() && j < b.len() && k < c.len() {
        let ka = key(&a[i]);
        let kb = key(&b[j]);
        let kc = key(&c[k]);

        if ka >= hint || kb >= hint || kc >= hint {
            break;
        }

        if ka == kb && kb == kc {
            f(i, j, k);
            i += 1;
            j += 1;
            k += 1;
        } else {
            let max = ka.max(kb).max(kc);

            while i < a.len() && key(&a[i]) < max {
                i += 1;
            }
            while j < b.len() && key(&b[j]) < max {
                j += 1;
            }
            while k < c.len() && key(&c[k]) < max {
                k += 1;
            }
        }
    }
}

pub fn common_neighbors_sorted_list_by_key<T, F, G, K>(a: &[T], b: &[T], key: G, mut f: F)
where
    G: Fn(&T) -> K,
    F: FnMut(usize, usize),
    K: Eq + Ord,
{
    let (mut i, mut j) = (0, 0);

    while i < a.len() && j < b.len() {
        let ka = key(&a[i]);
        let kb = key(&b[j]);

        if ka == kb {
            f(i, j);
            i += 1;
            j += 1;
        } else if ka < kb {
            i += 1;
        } else {
            j += 1;
        }
    }
}
