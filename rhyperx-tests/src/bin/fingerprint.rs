use rhyperx::CompactMotif;

use rhyperx_tests::shared::fingeprints::compute_all_fingerprints;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", compute_all_fingerprints::<CompactMotif!(3)>(true)?);
    println!("{}", compute_all_fingerprints::<CompactMotif!(4)>(true)?);
    println!("{}", compute_all_fingerprints::<CompactMotif!(5)>(true)?);
    Ok(())
}
