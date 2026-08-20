use rust_core_tests::shared::fingeprints::{compute_all_fingerprints, print_static_const};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_static_const::<3>();
    print_static_const::<4>();
    print_static_const::<5>();

    println!("{}", compute_all_fingerprints::<3>(true)?);
    println!("{}", compute_all_fingerprints::<4>(true)?);
    println!("{}", compute_all_fingerprints::<5>(true)?);
    Ok(())
}
