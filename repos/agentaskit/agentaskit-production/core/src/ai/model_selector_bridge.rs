//! Feature-gated model selector bridge.
//! Enable with `--features model-selector-llama` to include llama.cpp bridge.

#[cfg(feature = "model-selector-llama")]
pub mod llama_bridge {
    // TODO: fill with real bridge implementation wiring to integrations/llama.cpp
    pub fn select_and_infer(prompt: &str) -> Result<String, String> {
        // placeholder logic: route to llama.cpp binary/lib via FFI or CLI
        // keep this no-op to avoid affecting default builds
        Ok(format!("[llama.cpp] not yet wired; prompt.len={}", prompt.len()))
    }
}

#[cfg(not(feature = "model-selector-llama"))]
pub mod llama_bridge {
    pub fn select_and_infer(_prompt: &str) -> Result<String, String> {
        Err("model-selector-llama feature disabled".into())
    }
}
