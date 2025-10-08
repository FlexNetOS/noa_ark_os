# NOA ARK OS - Architecture

## System Layers

### Layer 1: Hardware Abstraction (HAL)
- Memory management
- CPU scheduling
- Device drivers
- Interrupt handling

### Layer 2: Core OS
- Process/thread management
- File system
- Network stack
- Security subsystem

### Layer 3: Runtime Layer
- Rust runtime (native)
- Python interpreter (embedded CPython)
- Go runtime
- .NET CoreCLR

### Layer 4: Service Layer
- Unified server
- MCP server
- AI inference service
- Application services

### Layer 5: Application Layer
- System applications
- User applications
- Development tools

## Component Communication

```
┌─────────────────────────────────────────┐
│         Application Layer               │
├─────────────────────────────────────────┤
│    Service Layer (Unified Server)       │
├─────────────────────────────────────────┤
│    Runtime Layer (Multi-language)       │
├─────────────────────────────────────────┤
│         Core OS (Rust-based)            │
├─────────────────────────────────────────┤
│    Hardware Abstraction Layer (HAL)     │
└─────────────────────────────────────────┘
```

## Self-Contained Strategy

### No External Dependencies Approach:
1. **Embedded Runtimes**: Bundle Python, Go, .NET runtimes
2. **Vendored Libraries**: Include all source code
3. **Custom Build System**: Self-hosting build tools
4. **Bundled AI Models**: Include model weights
5. **Integrated llama.cpp**: Compile and embed

## Technology Stack

- **Primary Language**: Rust (for OS core, performance-critical)
- **Systems Programming**: C (HAL, low-level)
- **Scripting**: Python (embedded)
- **Services**: Go (concurrent services)
- **Applications**: C# (.NET) for rapid development

## Data Flow

1. Application requests → Service Layer
2. Service Layer → Runtime (language-specific)
3. Runtime → Core OS
4. Core OS → HAL → Hardware

## AI Integration

- Llama.cpp compiled as static library
- Model weights stored in `/storage/models`
- Inference requests via MCP protocol
- Unified API for all AI operations
