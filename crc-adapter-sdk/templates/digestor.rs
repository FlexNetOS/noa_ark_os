// Digestor adapter template
use crc_adapter_sdk::{AdapterMetadata, CapabilityAdapter};
use serde_json::Value;

#[derive(Default)]
pub struct TemplateDigestor;

#[async_trait::async_trait]
impl CapabilityAdapter for TemplateDigestor {
    async fn execute(&self, input: Value) -> anyhow::Result<Value> {
        Ok(input)
    }

    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: "template.digestor".into(),
            kind: "digestor".into(),
            version: "0.1.0".into(),
            requires: vec![],
            provides: vec!["digest".into()],
        }
    }
}
