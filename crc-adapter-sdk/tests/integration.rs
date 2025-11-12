use async_trait::async_trait;
use crc_adapter_sdk::{scaffold, AdapterMetadata, CapabilityAdapter, CapabilityRegistry};
use serde_json::Value;

struct TestAdapter;

#[async_trait]
impl CapabilityAdapter for TestAdapter {
    async fn execute(&self, input: Value) -> anyhow::Result<Value> {
        Ok(input)
    }

    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: "test.adapter".into(),
            kind: "digestor".into(),
            version: "0.1.0".into(),
            requires: vec![],
            provides: vec!["digest".into()],
        }
    }
}

#[tokio::test]
async fn registry_resolves() {
    let mut registry = CapabilityRegistry::new();
    let adapter = TestAdapter;
    registry.register(adapter.metadata());
    let adapters = registry.resolve(&["digest".into()]).unwrap();
    assert_eq!(adapters.len(), 1);
}

#[test]
fn scaffold_contains_name() {
    let output = scaffold("digestor", "Sample");
    assert!(output.contains("SampleAdapter"));
}
