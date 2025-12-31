# Legacy Inventory

**Status**: Active  
**Last Updated**: 2025-11-13  
**Maintenance Mode**: Enabled

## Overview

This document tracks legacy systems, components, and patterns that are being maintained for backward compatibility or gradual deprecation. Legacy code is preserved under the `legacy` feature flag to isolate it from modern codebase patterns.

## Feature Flag

**Cargo Feature**: `legacy`  
**Default**: Disabled  
**Usage**: `cargo build --features legacy`

## Legacy Components

### Agents
- **Status**: Migration pending
- **Location**: `agents/src/implementations/_backup/`
- **Count**: 902 agents pending restoration
- **Scope**: Original agent implementations that need refactoring

### Systems
- **Status**: Under review
- **Items**: [To be cataloged]
- **Migration Path**: [To be defined]

### Patterns
- **Status**: Documentation in progress
- **Items**: [To be listed]
- **Replacement Patterns**: [To be linked]

## Deprecation Timeline

### Phase 1: Cataloging (Current)
- [ ] Inventory all legacy components
- [ ] Assess restoration effort for each
- [ ] Establish priority ranking
- [ ] Document replacement patterns

### Phase 2: Refactoring
- [ ] Select high-priority agents for restoration
- [ ] Implement modern trait-based patterns
- [ ] Add comprehensive tests
- [ ] Update documentation

### Phase 3: Gradual Deprecation
- [ ] Move to `legacy` feature flag
- [ ] Issue deprecation notices
- [ ] Guide migration path
- [ ] Set sunset date

## Dependencies on Legacy

Components that currently depend on legacy systems:
- [To be filled as dependencies are discovered]

## Test Coverage

- **Unit Tests**: [Status]
- **Integration Tests**: [Status]
- **Feature Tests**: [Status]

## Performance Baseline

Metrics for legacy components before optimization:
- [To be established]

## Notes

- All legacy restoration follows the agent trait pattern defined in `agents/src/trait.rs`
- No new code should be added to legacy implementations
- Restoration follows the workflow patterns in `WORKSPACE_MEMORY.md`
- Archive references maintained in `crc/archive/forks/`

## Related Documentation

- `WORKSPACE_MEMORY.md` - Restoration workflow
- `agents/README.md` - Agent architecture
- `crc/FORK_PROCESSING_SYSTEM.md` - Archive system
- `CONTRIBUTING.md` - Code standards
