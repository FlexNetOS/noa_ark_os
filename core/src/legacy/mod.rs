//! Legacy module scaffold
//! All deprecated or archived logic lives here behind an optional feature gate.
//! Enable with `--features legacy` when the feature is added to `core/Cargo.toml`.

#![allow(dead_code)]
#![allow(unused_imports)]

// Macro for wrapping individual legacy items with allowance attributes.
macro_rules! legacy_item {
    ($item:item) => {
        #[allow(dead_code)]
        #[allow(unused_imports)]
        $item
    };
}

pub(crate) use legacy_item;

// LEGACY BLOCK TEMPLATE
// Origin: (fill in source project / commit)
// Reason: (brief reason for archival)
// Dependencies: (list modules/crates relied upon)
// Revival Plan: (target version / milestone)
// Status: Archived

legacy_item! {
    /// Example legacy function placeholder. Replace with actual archived logic.
    pub fn legacy_placeholder() {
        // Intentionally left blank. Insert preserved code here.
    }
}
