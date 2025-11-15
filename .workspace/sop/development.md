# Standard Operating Procedure: Development

## Purpose
Establish consistent development practices across the NOA ARK OS project.

## Scope
All development activities including coding, testing, and documentation.

## Procedure

### 1. Starting New Work

#### 1.1 Check Task Queue
```bash
# View current sprint tasks
workspace todo list --sprint current

# Select a task
workspace todo start <task-id>
```

#### 1.2 Verify Environment
```bash
# Check workspace status
workspace status

# Verify no conflicts
workspace check

# Update dependencies
cargo update
```

#### 1.3 Create Branch (if applicable)
```bash
# For features
git checkout -b feature/task-name

# For bugfixes
git checkout -b bugfix/issue-name
```

### 2. Making Changes

#### 2.1 Edit Files (SOT Principle)
- **DO**: Edit the single canonical version
- **DON'T**: Create `.backup`, `.old`, or duplicate files
- All changes tracked by version control

#### 2.2 Follow Code Standards
```yaml
standards:
  rust:
    - Run: cargo fmt
    - Lint: cargo clippy
    - Test: cargo test
  
  python:
    - Format: black .
    - Lint: pylint, mypy
    - Test: pytest
  
  documentation:
    - Update README if public API changes
    - Add inline comments for complex logic
    - Update CHANGELOG.md
```

#### 2.3 Run Tests Locally
```bash
# Quick tests
cargo test --lib

# Full test suite
cargo test --workspace

# Integration tests
cargo test --test integration_test

# Specific component
cargo test -p noa_core
```

#### 2.4 Update Documentation
- Update README if interface changes
- Add/update doc comments
- Update architecture docs if structure changes

### 3. Committing Changes

#### 3.1 Stage Changes
```bash
# Review changes
git status
git diff

# Stage files
git add <files>

# Or stage all
git add .
```

#### 3.2 Commit Message Format
```
type(scope): subject

body (optional)

footer (optional)
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, no code change
- `refactor`: Code restructuring
- `test`: Adding/updating tests
- `chore`: Maintenance tasks

**Examples**:
```bash
git commit -m "feat(crc): add sandbox model isolation"
git commit -m "fix(cicd): correct auto-rollback timeout"
git commit -m "docs(workflow): update stage dependencies"
```

#### 3.3 Pre-Commit Checks
Automatically runs:
- Formatting check
- Linting
- Quick tests
- File hash update

### 4. Testing

#### 4.1 Unit Tests
```bash
# Run unit tests
cargo test --lib

# With coverage
cargo tarpaulin --out Html
```

#### 4.2 Integration Tests
```bash
# Run integration tests
cargo test --test integration_test

# Run examples
cargo run --example full_system_demo
cargo run --example crc_cicd_demo
```

#### 4.3 Manual Testing
- Test affected features
- Check edge cases
- Verify error handling
- Test performance if applicable

### 5. Code Review

#### 5.1 Self-Review Checklist
- [ ] Code follows standards
- [ ] Tests pass
- [ ] Documentation updated
- [ ] No duplicate files
- [ ] Registry updated (if applicable)
- [ ] No hardcoded secrets
- [ ] Error handling adequate
- [ ] Performance acceptable

#### 5.2 Request Review
```bash
# Push branch
git push origin feature/task-name

# Create PR (if using Git platform)
# Or mark task for review
workspace todo review <task-id>
```

### 6. Completing Work

#### 6.1 Final Checks
```bash
# Full test suite
cargo test --workspace

# Build release
cargo build --release

# Check workspace
workspace check
```

#### 6.2 Update Registry
```bash
# Update file hashes
workspace registry update

# Verify integrity
workspace registry verify
```

#### 6.3 Merge and Cleanup
```bash
# Merge to main
git checkout main
git merge feature/task-name

# Delete branch
git branch -d feature/task-name

# Mark task complete
workspace todo done <task-id>
```

#### 6.4 Trigger CI/CD
- Automatic on push to main
- Or manual: `cicd trigger`

### 7. Emergency Procedures

#### 7.1 Breaking Change
1. Stop all work
2. Assess impact
3. Create hotfix branch
4. Fix and test
5. Deploy immediately
6. Document incident

#### 7.2 Rollback
```bash
# Automatic rollback (CI/CD)
# CI/CD monitors and rolls back automatically

# Manual rollback
workspace restore --date <date>
cicd rollback <deployment-id>
```

## Best Practices

### Code Quality
✅ Write tests first (TDD)
✅ Keep functions small (< 50 lines)
✅ Use meaningful names
✅ Comment complex logic
✅ Handle errors properly
✅ Log important events

### Performance
✅ Profile before optimizing
✅ Use appropriate data structures
✅ Avoid premature optimization
✅ Cache expensive operations
✅ Use async for I/O

### Security
✅ Validate all inputs
✅ Use secrets management
✅ Scan for vulnerabilities
✅ Follow least privilege
✅ Audit sensitive operations

### Collaboration
✅ Communicate changes
✅ Review others' code
✅ Document decisions
✅ Help teammates
✅ Share knowledge

## Common Issues

### Issue: Duplicate Files Created
**Solution**:
```bash
# Run cleanup
workspace clean

# Prevent future
# Don't create .backup files manually
```

### Issue: Tests Failing
**Solution**:
```bash
# Run specific test
cargo test test_name -- --nocapture

# Check recent changes
git diff HEAD~1

# Restore if needed
git checkout HEAD~1 <file>
```

### Issue: Merge Conflicts
**Solution**:
```bash
# Update from main
git pull origin main

# Resolve conflicts manually
# Edit files, remove markers

# Test after resolution
cargo test

# Commit resolution
git commit -m "fix: resolve merge conflicts"
```

## Tools

### Required
- Rust toolchain (latest stable)
- Git
- Workspace CLI tool

### Recommended
- rust-analyzer (IDE support)
- cargo-watch (auto-rebuild)
- cargo-tarpaulin (coverage)

### Optional
- cargo-expand (macro expansion)
- cargo-flamegraph (profiling)
- cargo-audit (security)

## Metrics

Track development metrics:
- Commit frequency
- Test coverage
- Build success rate
- Review turnaround time
- Bug escape rate
- Deployment frequency

## Related SOPs

- [Deployment SOP](.workspace/sop/deployment.md)
- [Backup SOP](.workspace/sop/backup.md)
- [Recovery SOP](.workspace/sop/recovery.md)

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-15 | Initial version |
