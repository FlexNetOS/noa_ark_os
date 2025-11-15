use crate::utils::simple_hash;

/// Compute a stable symbol identifier that remains consistent across
/// repository reorganisations.
///
/// The identifier is derived from semantic attributes (language, kind,
/// canonical name, and signature) so that refactors which move files but
/// keep the declaration intact do not change the ID.
pub fn stable_symbol_id(language: &str, name: &str, kind: &str, signature: &str) -> String {
    let canonical = format!(
        "{}::{}::{}::{}",
        language.to_lowercase(),
        kind.to_lowercase(),
        name.trim(),
        signature.trim()
    );
    simple_hash(&canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_stable_for_semantic_equivalence() {
        let id_a = stable_symbol_id("rust", "compute", "function", "i32,i32");
        let id_b = stable_symbol_id("RUST", "compute", "FUNCTION", "i32,i32");
        assert_eq!(id_a, id_b);
    }
}
