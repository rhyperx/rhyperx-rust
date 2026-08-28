use rust_core::types::{Hx, StaticEdgeSet};

pub fn main() {
    let mut x = StaticEdgeSet::new();
    let _y = Hx::new_unweighted_unchecked([1, 2]);

    x.insert(Hx::new_unchecked([1, 2, 3, 4], 3));
    x.insert(Hx::new_unchecked([1, 2, 3, 4], 2));
    x.insert(Hx::new_unchecked([1, 2, 3, 4, 5], 2));
}
