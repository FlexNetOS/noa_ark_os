# Defer to AGENT.md

Use `AGENT.md` at the repository root as the sole policy and instruction source. Do not duplicate logic here.

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
