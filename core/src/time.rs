//! Time utilities for NOA ARK OS

use std::time::{SystemTime, UNIX_EPOCH};

/// Get the current timestamp in milliseconds since the Unix epoch.
///
/// Returns 0 if the system time is before the Unix epoch (which should never happen
/// on modern systems, but we handle it gracefully).
pub fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_timestamp_millis() {
        let timestamp = current_timestamp_millis();
        // Check that we get a reasonable timestamp (after 2020-01-01)
        assert!(timestamp > 1577836800000);
    }
}
