# NOA ARK OS - Examples

Complete examples demonstrating the full NOA ARK OS system integration.

## Examples

### 1. Complete System Demo (`complete_system_demo.rs`)

**The ultimate comprehensive example** showing ALL 11 components working together:

#### Systems Demonstrated

1. **Workspace Organization** 
   - Single Source of Truth (SOT)
   - File hash registry
   - Version tracking
   - Automated backups and compression
   
2. **Self-Hosted Apps**
   - Owned vs external apps
   - Dynamic app switching (redis → noa_cache)
   - Auto-fallback mechanisms

3. **Graph Generation**
   - Architecture diagrams
   - Dependency graphs
   - Workflow visualizations
   - Real-time metrics dashboards

4. **Core OS**
   - Kernel initialization
   - Process management
   - Memory management
   - IPC system
   - File system
   - Security subsystem

5. **CRC System with Sandbox Models**
   - Code drop from external sources
   - AI-supervised adaptation
   - **Sandbox Model A**: Feature development
   - **Sandbox Model B**: Bug fixes
   - **Sandbox Model C**: Experimental code
   - **Sandbox Model D**: Integration (A+B+C merged)
   - Auto-approve based on confidence (≥95%)

6. **Agent Factory**
   - Master orchestrator agent
   - Multi-language worker swarms (Rust, Python, Go)
   - Hive mind knowledge sharing
   - Disposable sub-agents

7. **Unified Workflow**
   - 7-stage deployment pipeline
   - Parallel execution across swarms
   - Cross-component orchestration
   - Stage dependencies

8. **Sandbox System (Traditional)**
   - Feature/BugFix/Experimental sandboxes
   - Validation and testing
   - Merge to integration

9. **CI/CD Pipeline**
   - CRC-integrated pipeline
   - Auto-approval from AI confidence
   - Blue-Green deployment (staging)
   - Canary deployment (production)
   - Auto-rollback on failure

10. **Server Infrastructure** (conceptual)
    - API Gateway
    - Orchestration
    - Inference engine
    - Retrieval/RAG

11. **Observability**
    - Structured logging
    - Prometheus metrics
    - OpenTelemetry traces
    - Real-time dashboards

#### Running the Example

```bash
# Run the complete system demo
cargo run --example complete_system_demo

# With release optimizations
cargo run --release --example complete_system_demo
```

#### Example Output

```
╔════════════════════════════════════════════════════════════════════╗
║          NOA ARK OS - COMPLETE SYSTEM INTEGRATION DEMO            ║
║     Full Automation: Code Drop → Adaptation → Production          ║
╚════════════════════════════════════════════════════════════════════╝

PHASE 1: WORKSPACE & INFRASTRUCTURE SETUP
✓ Core OS initialized
✓ Workspace management initialized
✓ Self-hosted apps system ready
✓ Graphs generated

PHASE 2: CRC - CONTINUOUS RECODE WITH SANDBOX MODELS
✓ Drop A processed in Model A (Feature)
✓ Drop B processed in Model B (Bug Fix)
✓ Drop C processed in Model C (Experimental)
✓ Merged A+B+C → Model D (Integration)

PHASE 3: AGENT FACTORY - MULTI-LAYERED AI AGENTS
✓ Master agent created
✓ 3 swarms created (12 agents total)
✓ Knowledge shared across hive mind

PHASE 4: UNIFIED WORKFLOW - CROSS-COMPONENT ORCHESTRATION
✓ 7-stage workflow executed
✓ Parallel execution across swarms

PHASE 5: SANDBOX SYSTEM - TRADITIONAL VALIDATION
✓ Integration test sandbox validated

PHASE 6: CI/CD - AUTOMATED DEPLOYMENT
✓ Pipeline auto-approved (96% AI confidence)
✓ Deployed to staging (Blue-Green)
✓ Deployed to production (Canary)

PHASE 7: OBSERVABILITY & MONITORING
✓ Metrics: Active
✓ Traces: 47 spans recorded
✓ Logs: Structured JSON

PHASE 8: CLEANUP & ARCHIVAL
✓ Archives compressed (94.2% reduction)
✓ Disposable agents cleaned
✓ Registry updated

🎉 COMPLETE SYSTEM INTEGRATION SUCCESSFUL!
⚡ From code drop to production in < 15 minutes
```

#### Scenario

1. **External Code Drop**: 3 different code projects dropped
2. **CRC Adaptation**: AI analyzes and adapts code to NOA ARK OS
3. **Sandbox Isolation**: Each drop processed in separate sandbox (A, B, C)
4. **Validation**: Independent validation of each sandbox
5. **Merge**: All sandboxes merged to integration (D)
6. **Swarm Testing**: Multi-language agents run parallel tests
7. **CI Pipeline**: Automated build and validation
8. **Staging Deploy**: Blue-Green strategy for zero downtime
9. **Production Deploy**: Canary release with monitoring
10. **Auto-Rollback**: Automatic rollback if health checks fail
11. **Cleanup**: Archives, compression, agent cleanup

#### Key Metrics

- **Time**: ~15 minutes (code drop → production)
- **Automation**: 100% (zero human intervention)
- **AI Confidence**: 96% (auto-approved)
- **Compression**: 94.2% reduction in archives
- **Agents**: 13 total (1 master + 12 workers)
- **Test Coverage**: Parallel execution across 3 swarms
- **Downtime**: Zero (Blue-Green + Canary)
- **Success Rate**: 100% (with auto-rollback safety)

---

### 2. CRC/CI/CD Integration Demo (`crc_cicd_demo.rs`)

Focused example demonstrating the CRC (Continuous ReCode) system integrated with CI/CD pipelines.

#### Features

- Code drop and AI adaptation
- Auto-approve based on confidence
- Sandbox validation (A, B, C → D)
- CI/CD deployment strategies
- Health monitoring
- Auto-rollback

#### Running the Example

```bash
cargo run --example crc_cicd_demo
```

#### Scenario

1. Drop external code
2. CRC analyzes and adapts
3. High confidence (96%) → auto-approve
4. Validate in sandboxes
5. Merge to integration
6. Deploy to staging (Blue-Green)
7. Deploy to production (Canary)
8. Monitor and auto-promote or rollback

---

## Architecture Visualization

```
┌─────────────────────────────────────────────────────────────────┐
│                        EXTERNAL CODE                             │
│              (Stale repos, forks, mirrors)                       │
└────────────┬─────────────┬──────────────┬──────────────────────┘
             │             │              │
             ▼             ▼              ▼
      ┌──────────┐  ┌──────────┐  ┌──────────┐
      │ Model A  │  │ Model B  │  │ Model C  │
      │ Feature  │  │ Bug Fix  │  │Experiment│
      └────┬─────┘  └────┬─────┘  └────┬─────┘
           │             │              │
           │      CRC AI Adaptation     │
           │             │              │
           └─────────────┼──────────────┘
                         │
                         ▼
                  ┌──────────────┐
                  │   Model D    │
                  │ Integration  │
                  └──────┬───────┘
                         │
                         ▼
              ┌──────────────────────┐
              │   Agent Factory      │
              │   (Hive + Swarms)    │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Unified Workflow    │
              │  (7-stage pipeline)  │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │    CI/CD Pipeline    │
              │  (Blue-Green/Canary) │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │     PRODUCTION       │
              └──────────────────────┘
```

## Running All Examples

```bash
# Run all examples sequentially
cargo run --example complete_system_demo && \
cargo run --example crc_cicd_demo
```

## Development

### Adding New Examples

1. Create new example in `examples/`
2. Add to `Cargo.toml` `[[example]]` section
3. Document in this README
4. Use existing examples as templates

### Testing Examples

```bash
# Test that examples compile
cargo build --examples

# Run specific example
cargo run --example <example_name>

# With verbose output
RUST_LOG=debug cargo run --example <example_name>
```

## Key Concepts Demonstrated

### 1. Single Source of Truth (SOT)
No duplicate files, all versions tracked and compressed

### 2. CRC Sandbox Models
- **A**: Feature development (new functionality)
- **B**: Bug fixes (critical patches)
- **C**: Experimental (R&D, POCs)
- **D**: Integration (merged from A+B+C)

### 3. Multi-Layered Agents
- **Master**: High-level orchestration
- **Worker**: Specialized tasks
- **Sub-Agent**: Temporary, disposable
- **Swarm**: Coordinated parallel execution

### 4. Hive Mind
Shared knowledge across all agents, distributed intelligence

### 5. Zero-Downtime Deployment
- **Blue-Green**: Instant switchover
- **Canary**: Gradual rollout with monitoring
- **Auto-Rollback**: < 30 seconds on failure

### 6. AI-Supervised Automation
- 95%+ confidence → auto-approve
- < 95% confidence → human review
- Continuous learning and adaptation

### 7. Full Observability
- Structured logs (JSON)
- Metrics (Prometheus)
- Traces (OpenTelemetry)
- Real-time dashboards

## Performance Expectations

- **Code Drop to Production**: < 15 minutes
- **AI Adaptation**: < 2 minutes per drop
- **Parallel Testing**: 5x faster with swarms
- **Deployment**: < 1 minute (Blue-Green/Canary)
- **Rollback**: < 30 seconds
- **Compression**: 90-95% size reduction

## Requirements

- Rust 1.70+
- 4 GB RAM minimum (8 GB recommended)
- Multi-core CPU for parallel execution
- Disk space for archives and backups

## Troubleshooting

### Example Won't Compile

```bash
# Clean and rebuild
cargo clean
cargo build --examples

# Check dependencies
cargo tree
```

### Runtime Errors

```bash
# Run with debugging
RUST_LOG=debug cargo run --example complete_system_demo

# Check component initialization order
# Core OS must initialize first
```

### Performance Issues

```bash
# Use release mode for better performance
cargo run --release --example complete_system_demo

# Profile if needed
cargo flamegraph --example complete_system_demo
```

## Next Steps

After running these examples:

1. Explore individual component READMEs
2. Review architecture documentation
3. Check the roadmap for upcoming features
4. Try modifying examples to test different scenarios
5. Build your own workflows

## Related Documentation

- [Architecture](../docs/ARCHITECTURE.md)
- [Getting Started](../docs/GETTING_STARTED.md)
- [Integration Guide](../docs/INTEGRATION.md)
- [Roadmap](../docs/ROADMAP.md)
- [CRC/CI/CD Guide](../cicd/CRC_CI_CD.md)
- [Workspace Organization](../.workspace/README.md)
- [CRC Sandbox Models](../crc/SANDBOX_MODELS.md)

## License

MIT License - See [LICENSE](../LICENSE) for details.
