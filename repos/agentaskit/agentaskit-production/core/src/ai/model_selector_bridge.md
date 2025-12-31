# Model Selector Bridge (design)

- Feature flag: `model-selector-llama`
- Reads `integrations/llama.cpp/config/models.yaml`
- Strategy:
  - Primary: 7B model for main tasks
  - Secondary: 3B model for parallel/light tasks
  - Fan-out based on stack count; join results
- Interface in `model_selector_bridge.rs` to route calls.
