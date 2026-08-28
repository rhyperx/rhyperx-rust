#![allow(dead_code)]
// Recursive bitset tree. Each level is a bitset of the next level.
use num_traits::PrimInt;
use std::{
    fmt::Binary,
    ops::{BitAnd, BitAndAssign, BitOrAssign},
};

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
    T: PrimInt + BitOrAssign + BitAnd<Output = T> + BitAndAssign + Binary,
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

    fn count_common_rec(
        cbst_a: &CBST<T>,
        cbst_b: &CBST<T>,
        level: usize,
        a: T,
        b: T,
        leading_a: usize,
        leading_b: usize,
    ) -> (usize, usize, usize) {
        // println!("{} a {:016b}, i_a {}", level, a, leading_a);
        // println!("{} b {:016b}, i_b {}", level, b, leading_b);
        if level == 0 {
            return (
                (a & b).count_ones() as usize,
                a.count_ones() as usize,
                b.count_ones() as usize,
            );
        }

        let mut common = a & b;
        let mut curr_i = 0;
        let mut count = 0;

        let mut new_leading_a = 0;
        let mut new_leading_b = 0;

        // Iterate all common 1 bits
        while common != T::zero() {
            let trail_zeros = common.trailing_zeros() as usize;
            curr_i += trail_zeros;
            common = common >> trail_zeros;
            common &= common - T::one();

            let child_i_a =
                leading_a + (a & ((T::one() << curr_i) - T::one())).count_ones() as usize;
            let child_i_b =
                leading_b + (b & ((T::one() << curr_i) - T::one())).count_ones() as usize;

            let rv = Self::count_common_rec(
                cbst_a,
                cbst_b,
                level - 1,
                cbst_a.rep[level - 1][child_i_a],
                cbst_b.rep[level - 1][child_i_b],
                new_leading_a,
                new_leading_b,
            );
            count += rv.0;
            new_leading_a += rv.1;
            new_leading_b += rv.2;
        }

        (count, a.count_ones() as usize, b.count_ones() as usize)
    }

    fn descend_to_level(
        cbst: &CBST<T>,
        mut a: T,
        level: usize,
        target_level: usize,
        leading: usize,
        offset: usize,
        target_offset: usize,
    ) -> (usize, Option<(usize, T)>) {
        if level == target_level {
            if offset == target_offset {
                return (a.count_ones() as usize, Some((leading, a)));
            }
            return (a.count_ones() as usize, None);
        }

        let bits_size = size_of::<T>() * 8;
        let mut curr_i = 0;

        let mut curr_leading = 0;

        // Iterate all common 1 bits
        while a != T::zero() {
            let trail_zeros = a.trailing_zeros() as usize;
            curr_i += trail_zeros;
            a = a >> trail_zeros;
            a &= a - T::one();

            let (new_leading, rv) = Self::descend_to_level(
                cbst,
                cbst.rep[level - 1][curr_i],
                level - 1,
                target_level,
                curr_leading,
                bits_size * offset + curr_i,
                target_offset,
            );
            curr_leading += new_leading;

            if let Some((leading, a)) = rv {
                return (a.count_ones() as usize, Some((leading, a)));
            }
        }

        (a.count_ones() as usize, None)
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
