//! Hierarchical memory stores for NOA ARK OS workflows.

mod coordinator;
pub mod models;
mod store;

pub use coordinator::MemoryCoordinator;
pub use models::{MemoryCursor, MemoryRecord, MemoryRetrieval, MemoryRole};
pub use store::{LongTermMemory, MemoryError, SessionMemory};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn long_term_and_session_incremental_retrieval_is_fast() {
        let dir = tempdir().unwrap();
        let coordinator = MemoryCoordinator::new(dir.path()).unwrap();
        let mut metadata = HashMap::new();
        metadata.insert("topic".into(), "planning".into());

        for i in 0..50 {
            let mut tags = Vec::new();
            if i % 2 == 0 {
                tags.push("even".into());
            }
            coordinator
                .record_interaction(
                    Some("session-a"),
                    "planner",
                    MemoryRole::Observation,
                    &format!("observation-{i}"),
                    metadata.clone(),
                    tags,
                )
                .unwrap();
        }

        let mut cursor = MemoryCursor::default();
        let mut durations = Vec::new();
        for _ in 0..10 {
            let start = std::time::Instant::now();
            let retrieval = coordinator
                .incremental_context(Some("session-a"), cursor.clone(), Some(8))
                .unwrap();
            durations.push(start.elapsed().as_millis());
            cursor = retrieval.next_cursor;
            if retrieval.records.is_empty() {
                break;
            }
        }

        durations.sort();
        if !durations.is_empty() {
            let index = ((durations.len() as f64) * 0.95).ceil() as usize - 1;
            let index = index.min(durations.len() - 1);
            assert!(durations[index] < 100, "p95 exceeded: {:?}", durations);
        }
    }
}
