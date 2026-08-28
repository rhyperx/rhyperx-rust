use crate::util::permutations::Permutator;

#[macro_export]
macro_rules! iter_hyperedges {
    ($node_count: expr, $range: expr , |$edge:ident, $edge_size:ident, $edge_idx:ident| $body:block ) => {{
        let min: usize = *$range.start();
        let max: usize = *$range.end();
        assert!(min < max);
        assert!(max <= $node_count);

        let mut target_size = min;
        let mut edge_idx = 0;
        while target_size <= max {
            let mut curr_edge: [usize; $node_count] = [0; $node_count];
            let mut positions: [usize; $node_count] = [0; $node_count];
            let mut stack_size = 1;

            while stack_size > 0 {
                let stack_index = stack_size - 1;

                if stack_index == target_size {
                    let $edge = curr_edge;
                    let $edge_size = target_size;
                    let $edge_idx = edge_idx;

                    $body;
                    edge_idx += 1;
                    stack_size -= 1;
                } else {
                    if positions[stack_index] < $node_count - target_size + stack_size {
                        curr_edge[stack_index] = positions[stack_index];
                        positions[stack_index] += 1;

                        if stack_index + 1 < $node_count {
                            positions[stack_index + 1] = positions[stack_index];
                        }

                        stack_size += 1;
                    } else {
                        stack_size -= 1;
                    }
                }
            }
            target_size += 1;
        }
    }};
}

/// A constant function to calculate factorial for the M parameter
pub const fn factorial(n: usize) -> usize {
    let mut res = 1;
    let mut i = 1;
    while i <= n {
        res *= i;
        i += 1;
    }
    res
}

/// Computes the binomial coefficient (n choose m) at compile time.
pub const fn binomial_coefficient(n: usize, mut m: usize) -> usize {
    if m > n {
        return 0;
    }
    if m == 0 || m == n {
        return 1;
    }

    // Optimize using the symmetry property: (n choose m) == (n choose n - m)
    if m > n - m {
        m = n - m;
    }

    let mut res = 1;
    let mut i = 0;

    while i < m {
        // Safe from precision loss because the product of `i + 1`
        // consecutive integers is always divisible by `(i + 1)!`
        res = (res * (n - i)) / (i + 1);
        i += 1;
    }

    res
}

pub const fn max_hyperedge_count(
    node_count: usize,
    min_edge_size: usize,
    max_edge_size: usize,
) -> usize {
    let mut total = 0;
    let mut edge_size = min_edge_size;
    while edge_size <= max_edge_size {
        total += binomial_coefficient(node_count, edge_size);
        edge_size += 1;
    }
    total
}

const fn iota<const N: usize>() -> [usize; N] {
    let mut arr = [0; N];
    let mut i = 0;
    while i < N {
        arr[i] = i;
        i += 1;
    }
    arr
}

const fn generate_permutations<const N: usize, const M: usize>(
    arr: &mut [u8; N],
    k: usize,
    out: &mut [[u8; N]; M],
    count: &mut usize,
) {
    if k == 1 {
        out[*count] = *arr;
        *count += 1;
        return;
    }

    let mut i = 0;
    while i < k {
        generate_permutations(arr, k - 1, out, count);

        if k % 2 == 0 {
            // Swap i and k-1
            let tmp = arr[i];
            arr[i] = arr[k - 1];
            arr[k - 1] = tmp;
        } else {
            // Swap 0 and k-1
            let tmp = arr[0];
            arr[0] = arr[k - 1];
            arr[k - 1] = tmp;
        }
        i += 1;
    }
}

macro_rules! define_permutator {
    ($n:literal, $m:literal) => {
        impl Permutator<$n> {
            pub const fn get_permutations(v: [u8; $n]) -> [[u8; $n]; $m] {
                let mut permutations = [[0; $n]; $m];
                let mut current = v;
                let mut count = 0;
                generate_permutations(&mut current, $n, &mut permutations, &mut count);
                permutations
            }
        }
    };
}

define_permutator!(2, 2);
define_permutator!(3, 6);
define_permutator!(4, 24);
define_permutator!(5, 120);
