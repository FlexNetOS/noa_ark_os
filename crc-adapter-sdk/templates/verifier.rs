// Verifier adapter template
use crc_adapter_sdk::{AdapterMetadata, CapabilityAdapter};
use serde_json::Value;

#[derive(Default)]
pub struct TemplateVerifier;

#[async_trait::async_trait]
impl CapabilityAdapter for TemplateVerifier {
    async fn execute(&self, input: Value) -> anyhow::Result<Value> {
        Ok(Value::Bool(true))
    }

    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: "template.verifier".into(),
            kind: "verifier".into(),
            version: "0.1.0".into(),
            requires: vec![],
            provides: vec!["verify".into()],
        }
    }
}
