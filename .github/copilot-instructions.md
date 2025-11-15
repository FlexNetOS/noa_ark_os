# NOA ARK OS - GitHub Copilot Workspace Instructions

**Workspace**: NOA ARK OS - Self-Contained Operating System Platform  
**Location**: `D:\dev\workspaces\noa_ark_os\`  
**Last Updated**: 2024-10-08

---

## 🎯 Environment & Setup

### Primary Environment
- **Operating System**: Windows 11 Pro
- **Primary Shell**: PowerShell 7+
- **IDE**: Visual Studio 2025 (18.0)
- **Rust Toolchain**: Portable Cargo v1.90.0
- **Rust Location**: `server/tools/cargo-portable/`
- **Workspace Root**: `D:\dev\workspaces\noa_ark_os\`

### Critical First Step
**ALWAYS activate portable Cargo before any Rust commands:**
```powershell
.\server\tools\activate-cargo.ps1
```

### Platform Preference
- **Primary**: Windows PowerShell (portable Cargo)
- **Alternative**: WSL/Ubuntu (for Linux testing)
- **Never**: Mix platforms in same session
- **Default Terminal**: PowerShell in VS Code

---

## 🏗️ Project Architecture

### Core Components

#### 1. **Agents** (`agents/`)
- **Purpose**: Agent Factory with hive mind & swarm capabilities
- **Registry**: 928 agents cataloged
- **Integrated**: 26 agents (placeholders)
- **Pending**: 902 agents to restore
- **Key Files**:
  - `agents/src/registry.rs` - 928 agent catalog
  - `agents/src/factory.rs` - Agent spawning
  - `agents/src/implementations/` - Agent implementations
  - `agents/src/implementations/_backup/` - Original 26 agents

#### 2. **CRC** (`crc/`)
- **Purpose**: Continuous ReCode system (AI-supervised adaptation)
- **Fork System**: Process external repositories
- **Drop-in**: `crc/drop-in/incoming/forks/`
- **Archive**: `crc/archive/forks/`
- **Key Files**:
  - `crc/detect-forks.ps1` - Fork detection automation
  - `crc/FORK_PROCESSING_SYSTEM.md` - Complete architecture

#### 3. **Core** (`core/`)
- **Purpose**: OS kernel and core services
- **Language**: Rust
- **Status**: Framework ready

#### 4. **CI/CD** (`cicd/`)
- **Purpose**: Continuous Integration/Deployment pipeline
- **Philosophy**: Maximum CD focus (<20 min commit to production)
- **Integration**: Connected to CRC system

#### 5. **Workflow** (`workflow/`)
- **Purpose**: Unified workflow orchestration
- **Stages**: Sequential, Parallel, Conditional, Loop
- **Multi-language**: Coordinates Rust, Python, Go

#### 6. **Sandbox** (`sandbox/`)
- **Purpose**: Isolated development environments
- **Pattern**: A, B, C → D (integration)
- **Definition of Ready**: Tests, coverage, security, performance

#### 7. **UI** (`ui/`)
- **Purpose**: Dynamic multi-platform UI/UX
- **Platforms**: Server, Mobile, Desktop, Web, AR, XR

#### 8. **Server** (`server/`)
- **Purpose**: Unified application server
- **Components**: MCP server, API gateway, orchestration
- **Tools**: `server/tools/` - Portable development tools

---

## ⚙️ Common Commands

### Build & Test
```powershell
# Build entire workspace
cargo build --workspace

# Build with release optimizations
cargo build --workspace --release

# Run all tests
cargo test --workspace

# Check without building
cargo check --workspace

# Clean build artifacts
cargo clean
```

### Code Quality
```powershell
# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace -- -D warnings

# Fix warnings automatically
cargo fix --workspace --allow-dirty
```

### Fork Processing
```powershell
# Process specific fork
.\crc\detect-forks.ps1 -Mode process -ForkName "fork-name"

# List all forks
.\crc\detect-forks.ps1 -Mode list

# Watch for new forks (continuous)
.\crc\detect-forks.ps1 -Mode watch -IntervalSeconds 60
```

### Examples
```powershell
# Agent Registry Demo
cargo run --example agent_registry_demo

# Full System Demo
cargo run --example full_system_demo

# CRC/CI/CD Demo
cargo run --example crc_cicd_demo
```

---

## 📝 Coding Patterns & Standards

### Rust Conventions

#### Error Handling
```rust
// For applications (binaries)
use anyhow::{Result, Context};

fn do_something() -> Result<()> {
    some_operation()
        .context("Failed to do something")?;
    Ok(())
}

// For libraries (crates)
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

#### Async Operations
```rust
// Always use tokio for async runtime
use tokio::fs;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Prefer async for I/O operations
    let contents = fs::read_to_string("file.txt").await?;
    Ok(())
}
```

#### Agent Pattern
```rust
// Standard agent structure (to be defined)
pub trait Agent: Send + Sync {
    fn metadata(&self) -> &AgentMetadata;
    async fn initialize(&mut self) -> Result<()>;
    async fn execute_task(&mut self, task: Task) -> Result<TaskResult>;
    async fn shutdown(&mut self) -> Result<()>;
}
```

### File Organization
- **Module structure**: Use `mod.rs` or single file with `pub mod`
- **Tests**: In `tests/` directory or `#[cfg(test)] mod tests`
- **Examples**: In `examples/` directory at workspace root
- **Documentation**: Inline `///` comments for public APIs

### Documentation Standards
```rust
/// Brief one-line description
///
/// Longer description with details about the function,
/// its purpose, and any important notes.
///
/// # Arguments
/// * `param1` - Description of parameter
///
/// # Returns
/// Description of return value
///
/// # Errors
/// When this function returns an error
///
/// # Examples
/// ```
/// let result = function(param);
/// assert_eq!(result, expected);
/// ```
pub fn function(param: Type) -> Result<ReturnType> {
    // Implementation
}
```

---

## 🔄 Workflow Patterns

### Feature Development
```powershell
# 1. Create feature branch
git checkout -b feature/my-feature

# 2. Make changes
# ... edit files ...

# 3. Build and test
cargo build --workspace
cargo test --workspace
cargo clippy --workspace

# 4. Commit
git add .
git commit -m "feat: description of feature"

# 5. Push
git push origin feature/my-feature
```

### Fork Processing Workflow
```powershell
# 1. Drop fork into directory
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks
mkdir awesome-library
# Copy files into awesome-library/

# 2. Process fork
cd ..\..\..\..
.\crc\detect-forks.ps1 -Mode process -ForkName "awesome-library"

# 3. Review branch
git checkout fork/awesome-library
git log

# 4. Test integration
cargo build
cargo test

# 5. Merge if successful
git checkout main
git merge fork/awesome-library
```

### Agent Restoration Workflow
```powershell
# 1. Review backup
cat agents\src\implementations\_backup\agent_name.rs

# 2. Restore agent
# ... implement full functionality ...

# 3. Update factory
# ... register in factory ...

# 4. Test
cargo test -p noa_agents

# 5. Document
# ... update README ...
```

---

## 🚨 Important Rules & Constraints

### Always Do
- ✅ Activate portable Cargo at session start
- ✅ Use PowerShell for all operations
- ✅ Build entire workspace before committing
- ✅ Run tests after changes
- ✅ Document public APIs
- ✅ Follow Rust 2021 edition conventions
- ✅ Use async/await for I/O operations
- ✅ Add error context with `.context()`
- ✅ Check `cargo clippy` warnings

### Never Do
- ❌ Mix WSL and Windows paths
- ❌ Commit without building
- ❌ Leave commented-out code
- ❌ Use `.unwrap()` in production code
- ❌ Commit `target/` directory
- ❌ Keep stale code after fork processing
- ❌ Use blocking I/O in async functions
- ❌ Ignore compiler warnings

### Fork Processing Rules
- ❌ **No live code after processing** - All originals compressed
- ✅ **Branch isolation** - Each fork gets `fork/{name}` branch
- ✅ **Compress & archive** - After integration to `crc/archive/forks/`
- ✅ **Cross-reference** - Maintain file mappings in metadata
- ✅ **Clean workspace** - Delete originals after archival

---

## 📦 Dependencies & Packages

### Core Dependencies
```toml
# Async runtime
tokio = { version = "1", features = ["full"] }

# Error handling
anyhow = "1.0"      # For applications
thiserror = "1.0"   # For libraries

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

### When Adding New Dependencies
1. Check if already in workspace `Cargo.toml`
2. Use workspace-level version if available
3. Document why the dependency is needed
4. Verify license compatibility (MIT, Apache-2.0, BSD)
5. Run `cargo build` to verify

---

## 🔍 Debugging & Troubleshooting

### Common Issues

#### "cargo: command not found"
**Cause**: Forgot to activate portable Cargo  
**Fix**: Run `.\server\tools\activate-cargo.ps1`

#### "rust-analyzer: not found"
**Cause**: VS Code not configured for portable Cargo  
**Fix**: Check `.vscode/settings.json`, reload window

#### "Compilation failed"
**Cause**: Workspace changes not built  
**Fix**: Run `cargo build --workspace`

#### "Tests failing"
**Cause**: Changes broke existing functionality  
**Fix**: Run `cargo test --workspace --verbose`, review output

#### "Fork not detected"
**Cause**: Metadata not initialized  
**Fix**: Run `.\crc\detect-forks.ps1 -Mode process -ForkName "name"`

### Debug Commands
```powershell
# Verbose build
cargo build --workspace --verbose

# Show test output
cargo test --workspace -- --nocapture

# Check specific package
cargo check -p noa_agents --verbose

# Show dependency tree
cargo tree

# Check for updates
cargo outdated
```

---

## 📊 Project Metrics & Standards

### Quality Targets
- **Test Coverage**: > 80%
- **Build Time**: < 5 minutes (full workspace)
- **Test Time**: < 10 minutes (all tests)
- **Response Time**: < 100ms (p95)
- **Change Failure Rate**: < 5%
- **Mean Time to Recovery**: < 5 minutes

### Performance Targets
- **Build**: < 5 minutes
- **Tests**: < 10 minutes
- **Deploy**: < 5 minutes
- **CI/CD Total**: < 20 minutes (commit to production)

---

## 🔗 Quick References

### File Locations
```
D:\dev\workspaces\noa_ark_os\
├── agents/                     # Agent factory (928 agents)
├── crc/                        # Continuous ReCode
│   ├── drop-in/incoming/forks/ # Fork drop-in
│   ├── archive/forks/          # Compressed archives
│   └── detect-forks.ps1        # Automation
├── cicd/                       # CI/CD pipeline
├── core/                       # OS kernel
├── workflow/                   # Workflow engine
├── sandbox/                    # Development sandboxes
├── ui/                         # Multi-platform UI
├── server/                     # Application server
│   └── tools/                  # Portable tools
├── examples/                   # Example applications
└── Cargo.toml                  # Workspace manifest
```

### Documentation
- **Workspace Memory**: `WORKSPACE_MEMORY.md`
- **Session Complete**: `SESSION_COMPLETE.md`
- **Fork System**: `crc/FORK_PROCESSING_SYSTEM.md`
- **Fork Test Plan**: `crc/FORK_TEST_PLAN.md`
- **Build Status**: `BUILD_SUCCESS_STATUS.md`

### External Resources
- **Rust Book**: https://doc.rust-lang.org/book/
- **Tokio Docs**: https://tokio.rs/
- **Cargo Book**: https://doc.rust-lang.org/cargo/

---

## 🎯 Current Focus & Priorities

### Phase 1: ✅ COMPLETE
- Agent registry system (928 agents)
- Fork processing infrastructure
- Build system verification
- Documentation foundation

### Phase 2: 🔄 IN PROGRESS
- Fork system testing
- CRC AI integration design
- Agent trait definition
- First agent restoration (DigestAgent)

### Phase 3: 📋 PLANNED
- Runtime environment integration
- AI engine implementation
- Server infrastructure
- Full agent restoration (902 agents)

---

## 💡 AI Assistant Guidelines

### When I Ask You To...

**"Build the workspace"**
```powershell
cd D:\dev\workspaces\noa_ark_os
.\server\tools\activate-cargo.ps1
cargo build --workspace
```

**"Process a fork"**
```powershell
cd D:\dev\workspaces\noa_ark_os
.\crc\detect-forks.ps1 -Mode process -ForkName "fork-name"
```

**"Run tests"**
```powershell
cd D:\dev\workspaces\noa_ark_os
.\server\tools\activate-cargo.ps1
cargo test --workspace
```

**"Add a new agent"**
1. Create file in `agents/src/implementations/`
2. Update `agents/src/implementations/mod.rs`
3. Register in `agents/src/registry.rs`
4. Add to factory in `agents/src/factory.rs`
5. Write tests
6. Build and verify

**"Fix build errors"**
1. Read error output carefully
2. Identify root cause
3. Fix in correct file
4. Run `cargo check` to verify
5. Run `cargo build` for full build
6. Run `cargo test` to ensure no regressions

---

## 🔒 Security Considerations

### Code Review Checklist
- [ ] No hardcoded secrets or credentials
- [ ] Input validation on all external data
- [ ] Proper error handling (no panics in production)
- [ ] Safe unwrapping (use `?` or handle explicitly)
- [ ] No SQL injection vectors
- [ ] No command injection vectors
- [ ] Dependencies are from trusted sources
- [ ] Licenses are compatible

### Fork Security
- Scan for malware before processing
- Check for exposed secrets
- Verify license compatibility
- Audit dependencies
- Isolate in sandbox for testing
- No automatic execution of external code

---

## 📞 Support & Help

### When Stuck
1. Check this file first
2. Review `WORKSPACE_MEMORY.md`
3. Check component README in directory
4. Search documentation map
5. Ask for clarification with context

### Reporting Issues
Include:
- Exact command run
- Full error output
- Current directory
- Cargo version (`cargo --version`)
- What you expected vs what happened

---

**Last Updated**: 2024-10-08  
**Build Status**: ✅ Passing (0.13s)  
**Workspace Status**: ✅ Operational  
**Fork System**: ✅ Ready for use  

---

**Remember**: This is a self-contained, zero-dependency workspace. Everything runs locally with portable tools. Always activate Cargo first, and use PowerShell for consistency!
