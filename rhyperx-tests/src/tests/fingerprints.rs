use crate::shared::fingeprints::compute_all_fingerprints;

#[test]
fn test_order3() -> Result<(), Box<dyn std::error::Error>> {
    let rv = compute_all_fingerprints::<3>(false)?;
    assert_eq!(rv.clashing_buckets_count, 0);
    Ok(())
}

#[test]
fn test_order4() -> Result<(), Box<dyn std::error::Error>> {
    let rv = compute_all_fingerprints::<4>(false)?;
    assert_eq!(rv.clashing_buckets_count, 0);
    Ok(())
}

#[test]
fn test_order5() -> Result<(), Box<dyn std::error::Error>> {
    let rv = compute_all_fingerprints::<5>(false)?;
    assert_eq!(rv.clashing_buckets_count, 0);
    Ok(())
}
