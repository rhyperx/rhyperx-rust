#![allow(dead_code)]
use std::{
    fmt::Binary,
    ops::{BitAnd, BitAndAssign, BitOrAssign, ShrAssign},
};

use num_traits::PrimInt;

pub struct CBST<T> {
    pub rep: Vec<Vec<T>>,
    pub offset: usize,
}

pub struct CBSTGraph<T> {
    pub rep: Vec<CBST<T>>,
    pub max_lvl: usize,
}

impl<T> Default for CBST<T>
where
    T: PrimInt + BitOrAssign,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CBST<T>
where
    T: PrimInt + BitOrAssign,
{
    pub fn new() -> Self {
        CBST {
            rep: Vec::new(),
            offset: 0,
        }
    }

    fn build_cbs_list(neigboors: &mut [usize], sort: bool) -> (Vec<T>, Vec<usize>) {
        if sort {
            neigboors.sort_unstable();
        }

        let bit_size = size_of::<T>() * 8;

        let mut next = T::zero();
        let mut offset = 0;
        let mut base = 0;
        let mut cbs = Vec::new();
        let mut offsets = Vec::new();

        // println!("Neigboors: {:?}", neigboors);
        for n in neigboors.iter().cloned() {
            let mut r_n = n - base;
            // println!("n: {}, r_n: {} offset: {}, base: {}", n, r_n, offset, base);
            if r_n >= bit_size {
                if next != T::zero() {
                    cbs.push(next);
                    offsets.push(offset);
                }
                offset += r_n / bit_size;
                base = offset * bit_size;
                next = T::zero();
                r_n = n - base;
            }
            next |= T::one() << r_n;
        }

        if next != T::zero() {
            cbs.push(next);
            offsets.push(offset);
        }

        (cbs, offsets)
    }

    pub fn from(neigboors: &mut [usize], sort: bool) -> Self {
        let mut rep = Vec::new();

        if neigboors.is_empty() {
            return Self { rep, offset: 0 };
        }

        let mut cbs;
        let mut offsets = Vec::from(neigboors);

        loop {
            (cbs, offsets) = Self::build_cbs_list(&mut offsets, sort);
            rep.push(cbs);
            if offsets.len() <= 1 {
                break;
            }
        }

        Self {
            rep,
            offset: offsets[0],
        }
    }

    pub fn size(&self) -> usize {
        self.rep.iter().map(|level| level.len()).sum()
    }

    pub fn levels(&self) -> usize {
        self.rep.len()
    }

    pub fn bytes(&self) -> usize {
        size_of::<T>() * self.size()
    }
}

impl<T> std::fmt::Debug for CBST<T>
where
    T: PrimInt + BitOrAssign + Binary,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let padding = size_of::<T>() * 8;
        for (i, level) in self.rep.iter().enumerate() {
            writeln!(f, "Level: {}", i)?; // Writes to the formatter, not stdout
            for c in level {
                writeln!(f, "{:0width$b}", c, width = padding)?;
            }
        }
        Ok(())
    }
}

impl<T> CBSTGraph<T>
where
    T: PrimInt + BitOrAssign + BitAnd<Output = T> + BitAndAssign + Binary + ShrAssign<usize>,
{
    fn new() -> Self {
        CBSTGraph {
            rep: Vec::new(),
            max_lvl: 0,
        }
    }

    pub fn from(graph: &mut [Vec<usize>], sort: bool) -> Self {
        let mut rep = Vec::new();
        for neigboors in graph.iter_mut() {
            rep.push(CBST::from(neigboors, sort));
        }
        let max_lvl = rep.iter().map(|cbst| cbst.levels()).max().unwrap_or(0);

        Self { rep, max_lvl }
    }

    #[inline(always)]
    fn count_common_rec(
        cbst_a: &CBST<T>,
        cbst_b: &CBST<T>,
        level: usize,
        a: T,
        b: T,
        leading_a: usize,
        leading_b: usize,
    ) -> (usize, usize, usize) {
        let mut stack = Vec::with_capacity(32);

        stack.push((level, a, b, leading_a, leading_b));

        let mut total_common = 0;

        while let Some((lvl, a, b, lead_a, lead_b)) = stack.pop() {
            if lvl == 0 {
                total_common += (a & b).count_ones() as usize;
                continue;
            }

            let mut common = a & b;
            let mut curr_i = 0;

            let mut prefix_a = 0;
            let mut prefix_b = 0;

            while common != T::zero() {
                let tz = common.trailing_zeros() as usize;
                curr_i += tz;

                common >>= tz;
                common &= common - T::one();

                let mask = (T::one() << curr_i) - T::one();

                let child_i_a = lead_a + (a & mask).count_ones() as usize;
                let child_i_b = lead_b + (b & mask).count_ones() as usize;

                let next_a = cbst_a.rep[lvl - 1][child_i_a];
                let next_b = cbst_b.rep[lvl - 1][child_i_b];

                stack.push((lvl - 1, next_a, next_b, prefix_a, prefix_b));

                prefix_a += next_a.count_ones() as usize;
                prefix_b += next_b.count_ones() as usize;
            }
        }

        (
            total_common,
            a.count_ones() as usize,
            b.count_ones() as usize,
        )
    }

    #[inline(always)]
    fn descend_to_level(
        cbst: &CBST<T>,
        a: T,
        level: usize,
        target_level: usize,
        leading: usize,
        offset: usize,
        target_offset: usize,
    ) -> (usize, Option<(usize, T)>) {
        let mut stack = Vec::with_capacity(32);
        stack.push((level, a, leading, offset));

        let bits_size = size_of::<T>() * 8;

        while let Some((lvl, mut bits, lead, off)) = stack.pop() {
            if lvl == target_level {
                if off == target_offset {
                    return (bits.count_ones() as usize, Some((lead, bits)));
                }
                continue;
            }

            let mut curr_i = 0;
            let mut curr_leading = 0;

            while bits != T::zero() {
                let tz = bits.trailing_zeros() as usize;
                curr_i += tz;

                bits >>= tz;
                bits &= bits - T::one();

                let child = cbst.rep[lvl - 1][curr_i];

                stack.push((lvl - 1, child, curr_leading, bits_size * off + curr_i));

                curr_leading += child.count_ones() as usize;
            }
        }

        (0, None)
    }

    pub fn count_common_neighbors(&self, u: usize, v: usize) -> usize {
        // Assumes both representations have been panned
        let cbst_a = &self.rep[u];
        let cbst_b = &self.rep[v];
        if cbst_a.levels() == 0 || cbst_b.levels() == 0 {
            return 0;
        }
        let (cbst_a, cbst_b) = if cbst_a.levels() < cbst_b.levels() {
            (cbst_a, cbst_b)
        } else {
            (cbst_b, cbst_a)
        };

        // println!("cbst_a {:?}", cbst_a);
        // println!("cbst_b {:?}", cbst_b);
        // println!("------------------------------");

        let a_root = cbst_a.rep[cbst_a.levels() - 1][0];
        let leading_a = 0;

        let b_root = cbst_b.rep[cbst_b.levels() - 1][0];
        let (_, descent_rv) = Self::descend_to_level(
            cbst_b,
            b_root,
            cbst_b.levels() - 1,
            cbst_a.levels() - 1,
            0,
            cbst_b.offset,
            cbst_a.offset,
        );

        if let Some((leading_b, b_root)) = descent_rv {
            // println!("a_root {:016b}, leading_a {}", a_root, leading_a);
            // println!("b_root {:016b}, leading_b {}", b_root, leading_b);
            // println!("------------------------------");
            Self::count_common_rec(
                cbst_a,
                cbst_b,
                cbst_a.levels() - 1,
                a_root,
                b_root,
                leading_a,
                leading_b,
            )
            .0
        } else {
            0
        }
    }
}
