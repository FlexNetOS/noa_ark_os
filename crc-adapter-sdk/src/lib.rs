use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[async_trait]
pub trait CapabilityAdapter: Send + Sync {
    async fn execute(&self, input: Value) -> Result<Value>;
    fn metadata(&self) -> AdapterMetadata;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterMetadata {
    pub id: String,
    pub kind: String,
    pub version: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub provides: Vec<String>,
}

pub struct CapabilityRegistry {
    adapters: HashMap<String, AdapterMetadata>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn register(&mut self, metadata: AdapterMetadata) {
        self.adapters.insert(metadata.id.clone(), metadata);
    }

    pub fn resolve(&self, requirements: &[String]) -> Result<Vec<AdapterMetadata>> {
        let mut resolved = Vec::new();
        for metadata in self.adapters.values() {
            if requirements
                .iter()
                .all(|req| metadata.provides.contains(req))
            {
                resolved.push(metadata.clone());
            }
        }
        if resolved.is_empty() {
            return Err(anyhow!("no adapters match requirements"));
        }
        Ok(resolved)
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn scaffold(kind: &str, name: &str) -> String {
    format!(
        "// Generated adapter\nuse crc_adapter_sdk::{{CapabilityAdapter, AdapterMetadata}};\nuse serde_json::Value;\n\n#[derive(Default)]\npub struct {name}Adapter;\n\n#[async_trait::async_trait]\nimpl CapabilityAdapter for {name}Adapter {{\n    async fn execute(&self, input: Value) -> anyhow::Result<Value> {{\n        Ok(input)\n    }}\n\n    fn metadata(&self) -> AdapterMetadata {{\n        AdapterMetadata {{\n            id: \"{kind}.{name}\".into(),\n            kind: \"{kind}\".into(),\n            version: \"0.1.0\".into(),\n            requires: vec![],\n            provides: vec![\"{kind}\".into()],\n        }}\n    }}\n}}\n",
        name = name,
        kind = kind
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoAdapter;

    #[async_trait]
    impl CapabilityAdapter for EchoAdapter {
        async fn execute(&self, input: Value) -> Result<Value> {
            Ok(input)
        }

        fn metadata(&self) -> AdapterMetadata {
            AdapterMetadata {
                id: "echo".into(),
                kind: "digestor".into(),
                version: "1.0.0".into(),
                requires: vec!["test".into()],
                provides: vec!["test".into()],
            }
        }
    }

    #[tokio::test]
    async fn registry_resolves_capabilities() {
        let mut registry = CapabilityRegistry::new();
        let adapter = EchoAdapter;
        registry.register(adapter.metadata());
        let resolved = registry.resolve(&["test".into()]).unwrap();
        assert_eq!(resolved.len(), 1);
    }

    #[test]
    fn scaffold_generates_source() {
        let source = scaffold("digestor", "Example");
        assert!(source.contains("ExampleAdapter"));
    }
}
