//! Utility functions shared across the NOA ARK OS core.

use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in milliseconds since UNIX epoch.
pub fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

/// Compute a simple FNV-1a hash for the given value.
///
/// WARNING: FNV-1a is a NON-CRYPTOGRAPHIC hash function designed for speed, not security.
/// It is vulnerable to collision attacks and does NOT provide the security guarantees
/// typically expected from digital signatures. This is used for deterministic, stable
/// signatures in the ledger system, but should NOT be relied upon for tamper detection
/// or security-critical operations. For production use, consider using a cryptographic
/// hash function like SHA-256 from the `sha2` crate.
pub fn simple_hash(value: &str) -> String {
    const OFFSET_BASIS: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_hash_deterministic() {
        let hash1 = simple_hash("test");
        let hash2 = simple_hash("test");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_simple_hash_different_inputs() {
        let hash1 = simple_hash("test1");
        let hash2 = simple_hash("test2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_timestamp_not_zero() {
        let ts = current_timestamp_millis();
        assert!(ts > 0);
    }
}
