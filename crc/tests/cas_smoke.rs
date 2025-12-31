use anyhow::Result;
use noa_crc::cas::Cas;

#[test]
fn cas_round_trip_preserves_hash() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let cas = Cas::new(temp_dir.path())?;

    let payload = b"smoke";
    let expected_hash = blake3::hash(payload).to_hex().to_string();

    let hash = cas.put_bytes(payload)?;
    assert_eq!(hash, expected_hash, "CAS hash should be stable");
    assert!(cas.exists(&hash));

    let fetched = cas.get(&hash)?;
    assert_eq!(fetched, payload);

    let entry = cas.stat(&hash)?;
    assert_eq!(entry.size, payload.len() as u64);

    let expected_path = temp_dir
        .path()
        .join(&hash[..2])
        .join(&hash[2..])
        .canonicalize()?;
    assert_eq!(entry.path, expected_path);

    Ok(())
}
