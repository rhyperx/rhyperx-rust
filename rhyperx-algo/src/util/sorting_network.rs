use paste::paste;

#[inline(always)]
fn cswap<T: Ord>(arr: &mut [T], i: usize, j: usize) {
    if arr[i] > arr[j] {
        arr.swap(i, j);
    }
}

#[inline(always)]
#[allow(unused)]
fn min<T: Ord + Copy>(arr: &mut [T], i: usize, j: usize) {
    arr[i] = std::cmp::min(arr[i], arr[j]);
}

#[inline(always)]
#[allow(unused)]
fn max<T: Ord + Copy>(arr: &mut [T], i: usize, j: usize) {
    arr[j] = std::cmp::max(arr[i], arr[j]);
}

pub trait NetSort<T: Ord, const N: usize> {
    fn network_sort(&mut self);
}

pub trait TryNetSort<T: Ord> {
    fn try_network_sort(&mut self);
    const SORT_FUNCTIONS: [fn(&mut [T]); 11];
}

macro_rules! impl_network {
    ($n:expr, { $(($i: literal, $j: literal));* $(;)? }) => {
        impl<T: Ord + Copy> NetSort<T, $n> for [T; $n] {
            #[inline(always)]
            fn network_sort(&mut self) {
                debug_assert!(self.len() == $n);
                $(
                    cswap(self, $i, $j);
                )*
            }
        }

        paste!{
            pub fn [<network_sort_$n>]<T: Ord>(_v: &mut [T]) {
                $(
                    cswap(_v, $i, $j);
                )*
            }
        }

    };
}

impl_network!(0, {});

impl_network!(1, {});

impl_network!(2, {
    (0, 1);
});

impl_network!(3, {
    (0, 1);
    (1, 2);
    (0, 1);
});

impl_network!(4, {
    (0, 1);
    (2, 3);
    (0, 2);
    (1, 3);
    (1, 2);
});

impl_network!(5, {
    (0, 1);
    (3, 4);
    (2, 4);
    (2, 3);
    (0, 3);
    (0, 2);
    (1, 4);
    (1, 3);
    (1, 2);
});

impl_network!(6, {
    (1, 2);
    (4, 5);
    (0, 2);
    (3, 5);
    (0, 1);
    (3, 4);
    (2, 5);
    (0, 3);
    (1, 4);
    (2, 4);
    (1, 3);
    (2, 3);
});

impl_network!(7, {
    (0, 1);
    (2, 3);
    (4, 5);
    (0, 2);
    (1, 3);
    (1, 2);
    (4, 6);
    (5, 6);
    (0, 4);
    (1, 5);
    (2, 6);
    (1, 4);
    (3, 6);
    (2, 4);
    (3, 5);
    (3, 4);
});

impl_network!(8, {
    (0, 1);
    (2, 3);
    (4, 5);
    (6, 7);
    (0, 2);
    (1, 3);
    (4, 6);
    (5, 7);
    (1, 2);
    (5, 6);
    (0, 4);
    (3, 7);
    (1, 5);
    (2, 6);
    (2, 4);
    (3, 5);
    (1, 2);
    (5, 6);
    (3, 4);
});

impl_network!(9, {
    (0, 1);
    (2, 3);
    (4, 5);
    (6, 7);
    (0, 6);
    (1, 7);
    (2, 4);
    (3, 8);
    (0, 2);
    (1, 6);
    (3, 4);
    (5, 8);
    (1, 5);
    (2, 3);
    (4, 6);
    (7, 8);
    (1, 2);
    (3, 4);
    (5, 7);
    (0, 1);
    (2, 3);
    (4, 5);
    (6, 7);
    (3, 4);
    (5, 6);
});

impl_network!(10, {
    (0, 5);
    (1, 6);
    (2, 7);
    (3, 8);
    (4, 9);
    (0, 3);
    (5, 8);
    (1, 4);
    (6, 9);
    (0, 2);
    (3, 6);
    (7, 9);
    (0, 1);
    (2, 4);
    (5, 7);
    (8, 9);
    (1, 2);
    (3, 5);
    (4, 6);
    (7, 8);
    (1, 3);
    (4, 7);
    (2, 5);
    (6, 8);
    (2, 3);
    (4, 5);
    (6, 7);
    (3, 4);
    (5, 6);
});

// const NETWORK_FUNCTIONS: = [
// ]

impl<T: Ord + Copy> TryNetSort<T> for [T] {
    const SORT_FUNCTIONS: [fn(&mut [T]); 11] = [
        network_sort_0::<T>,
        network_sort_1::<T>,
        network_sort_2::<T>,
        network_sort_3::<T>,
        network_sort_4::<T>,
        network_sort_5::<T>,
        network_sort_6::<T>,
        network_sort_7::<T>,
        network_sort_8::<T>,
        network_sort_9::<T>,
        network_sort_10::<T>,
    ];

    fn try_network_sort(&mut self) {
        Self::SORT_FUNCTIONS[self.len()](self);
    }
}
