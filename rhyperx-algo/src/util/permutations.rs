use crate::util::const_operations::factorial;

pub struct BinPerm {
    pub container: usize,
}

pub struct BinPermIterator {
    container: usize,
    n: usize,
}

impl Iterator for BinPermIterator {
    type Item = BinPerm;

    fn next(&mut self) -> Option<Self::Item> {
        if self.container < factorial(self.n) {
            let perm = BinPerm {
                container: self.container,
            };
            self.container += 1;
            Some(perm)
        } else {
            None
        }
    }
}

impl From<usize> for BinPerm {
    fn from(value: usize) -> Self {
        Self { container: value }
    }
}

impl BinPerm {
    pub const fn new() -> Self {
        Self { container: 0 }
    }

    pub const fn from_usize(container: usize) -> Self {
        Self { container }
    }

    pub const fn encode<const N: usize>(mut arr: [u8; N]) -> Self {
        let mut inv = [0; N];
        let mut i = 0;
        while i < N {
            inv[arr[i] as usize] = i as u8;
            i += 1;
        }

        let mut res = 0;
        let mut base = 1;
        let mut n = N;

        while n > 1 {
            let s = arr[n - 1] as usize;
            res += s * base;
            base *= n;

            let j = inv[n - 1] as usize;

            let temp_arr = arr[n - 1];
            arr[n - 1] = arr[j];
            arr[j] = temp_arr;

            let temp_inv = inv[s];
            inv[s] = inv[n - 1];
            inv[n - 1] = temp_inv;

            n -= 1;
        }

        Self { container: res }
    }

    pub const fn decode<const N: usize>(&self) -> [u8; N] {
        let mut num = self.container;

        let mut res = [0; N];
        let mut i = 0;
        while i < N {
            res[i] = i as u8;
            i += 1;
        }

        let mut n = N;
        while n > 1 {
            let swap_idx = num % n;
            num /= n;

            res.swap(n - 1, swap_idx);
            n -= 1;
        }

        res
    }

    pub const fn iter_all<const N: usize>() -> BinPermIterator {
        BinPermIterator { container: 0, n: N }
    }
}

pub struct Permutator<const N: usize> {}
