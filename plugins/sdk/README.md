# NOA Plugin SDK (Preview)

The NOA Plugin SDK provides lightweight helpers for discovering capabilities
from the shared `registry/tools.registry.json` catalogue. Plugins can leverage
this metadata to dynamically register commands, surface documentation, or bind
to the matching REST and gRPC endpoints exposed by the `noa` CLI.

## Features

- Load and validate the canonical tool registry file
- Filter capabilities by category (`observability`, `automation`, `analysis`,
  `collaboration`, `plugin`)
- Discover CLI mappings, parameter schemas, budget allocations, and side effect
  declarations
- Reference matching REST and gRPC endpoints for remote orchestration
- Capture automation guardrails such as approval requirements and linked
  runbooks

## Usage

```rust
use noa_plugin_sdk::ToolRegistry;

fn main() -> anyhow::Result<()> {
    let registry = ToolRegistry::from_path("registry/tools.registry.json")?;
    let observability = registry.tools_for_category("observability");
    for tool in observability {
        println!("{} => {}", tool.id, tool.summary);
    }
    Ok(())
}
```

Additional end-to-end samples live in `examples/` (coming in future updates).
