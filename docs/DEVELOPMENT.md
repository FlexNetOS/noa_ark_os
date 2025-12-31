# Development Guide

## Getting Started with Development

Welcome to the noa_ark_os unified development environment! This guide will help you set up your development environment and understand our development workflows.

## Prerequisites

### Required Tools

1. **Git** (2.30+)
   ```bash
   git --version
   ```

2. **Docker** (20.10+) and **Docker Compose** (1.29+)
   ```bash
   docker --version
   docker-compose --version
   ```

3. **Python** (3.8+)
   ```bash
   python --version
   pip --version
   ```

4. **Rust** (1.70+) and **Cargo**
   ```bash
   rustc --version
   cargo --version
   ```

5. **Node.js** (16+) - Optional, for some components
   ```bash
   node --version
   npm --version
   ```

### Optional Tools

- **VS Code** with recommended extensions
- **Git LFS** for large file handling
- **pre-commit** for automated checks

## Initial Setup

### 1. Clone the Repository

```bash
git clone https://github.com/FlexNetOS/noa_ark_os.git
cd noa_ark_os
```

### 2. Verify Subtree Structure

```bash
# List all remotes
git remote -v

# Check subtree structure
ls -la repos/
```

### 3. Install Dependencies

#### Python Components
```bash
# MicroAgentStack
cd repos/MicroAgentStack
pip install -r requirements.txt

# deflexnet-app
cd ../deflexnet-app
pip install -r requirements.txt

# ark-os-noa
cd ../ark-os-noa
pip install -r requirements.txt

cd ../..
```

#### Rust Components
```bash
# agentaskit
cd repos/agentaskit
cargo build

# deflex-ai-os
cd ../deflex-ai-os
cargo build

cd ../..
```

## Development Workflow

### Working with Subtrees

Each component in `/repos` is a git subtree. Here's how to work with them:

#### Update a Component from Upstream
```bash
# Pull latest changes from a component
git subtree pull --prefix=repos/MicroAgentStack MicroAgentStack main --squash
```

#### Push Changes to Component
```bash
# Push changes back to component repository
git subtree push --prefix=repos/MicroAgentStack MicroAgentStack main
```

#### Make Changes to a Component
```bash
# Navigate to the component
cd repos/MicroAgentStack

# Make your changes
# ... edit files ...

# Commit in the main repo
cd ../..
git add repos/MicroAgentStack
git commit -m "Update MicroAgentStack: description of changes"
```

### Branch Strategy

- `main` - Stable, production-ready code
- `develop` - Integration branch for features
- `feature/*` - Feature development branches
- `bugfix/*` - Bug fix branches
- `hotfix/*` - Emergency production fixes

### Making Changes

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Your Changes**
   - Edit files in the appropriate component directory
   - Follow component-specific coding standards
   - Add tests for new functionality

3. **Test Your Changes**
   ```bash
   # Run component-specific tests
   cd repos/MicroAgentStack
   python -m pytest tests/
   
   cd ../agentaskit
   cargo test
   ```

4. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: descriptive commit message"
   ```

5. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   # Create PR via GitHub UI
   ```

## Coding Standards

### Python (PEP 8)
```python
# Use type hints
def process_data(input_data: dict) -> list:
    """Process input data and return results.
    
    Args:
        input_data: Dictionary containing input parameters
        
    Returns:
        List of processed results
    """
    pass

# Use docstrings
# Follow naming conventions: snake_case for functions/variables
# Use 4 spaces for indentation
```

### Rust (Rustfmt)
```rust
// Use rustfmt for formatting
// cargo fmt

// Follow Rust naming conventions
// snake_case for functions, variables
// CamelCase for types, traits

pub fn process_task(task: Task) -> Result<Output, Error> {
    // Implementation
}
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Example**:
```
feat(MicroAgentStack): add agent health monitoring

Implement health check endpoints for all agents.
Includes metrics collection and alerting.

Closes #123
```

## Testing

### Python Tests
```bash
# MicroAgentStack
cd repos/MicroAgentStack
python -m pytest tests/ -v

# With coverage
python -m pytest --cov=. tests/
```

### Rust Tests
```bash
# agentaskit
cd repos/agentaskit
cargo test

# With output
cargo test -- --nocapture
```

### Integration Tests
```bash
# Run integration tests (to be implemented)
./scripts/run-integration-tests.sh
```

## Running Services Locally

### Individual Components

#### MicroAgentStack
```bash
cd repos/MicroAgentStack
docker-compose up
# Or
python main.py
```

#### ark-os-noa
```bash
cd repos/ark-os-noa
docker-compose up
```

#### deflex-ai-os
```bash
cd repos/deflex-ai-os
cargo run
# Or with Docker
docker-compose up
```

### All Services Together
```bash
# From root directory
./scripts/start-all-services.sh
```

## Debugging

### Python Debugging
```python
# Use pdb for debugging
import pdb; pdb.set_trace()

# Or use VS Code debugger with launch.json configuration
```

### Rust Debugging
```bash
# Use rust-gdb or rust-lldb
rust-gdb target/debug/your-binary

# Or use VS Code with CodeLLDB extension
```

### Docker Debugging
```bash
# View logs
docker-compose logs -f service-name

# Execute commands in running container
docker-compose exec service-name bash

# Inspect container
docker inspect container-id
```

## Performance Profiling

### Python Profiling
```bash
python -m cProfile -o output.prof your_script.py
python -m pstats output.prof
```

### Rust Profiling
```bash
cargo build --release
# Use perf, flamegraph, or other profiling tools
```

## Documentation

### Generating Documentation

#### Python (Sphinx)
```bash
cd repos/MicroAgentStack/docs
make html
```

#### Rust (rustdoc)
```bash
cd repos/agentaskit
cargo doc --open
```

### Writing Documentation

- Add docstrings to all public functions and classes
- Update README files when adding features
- Keep architecture diagrams up to date
- Document API changes in CHANGELOG

## Troubleshooting

### Common Issues

1. **Git Subtree Issues**
   ```bash
   # If subtree pull fails, try:
   git subtree split --rejoin --prefix=repos/ComponentName
   ```

2. **Docker Issues**
   ```bash
   # Clean up Docker
   docker system prune -a
   
   # Rebuild containers
   docker-compose down -v
   docker-compose up --build
   ```

3. **Dependency Issues**
   ```bash
   # Python: Clear pip cache
   pip cache purge
   pip install -r requirements.txt --force-reinstall
   
   # Rust: Clean build
   cargo clean
   cargo build
   ```

## CI/CD Integration

### GitHub Actions

Our CI/CD pipeline runs:
- Linting checks
- Unit tests
- Integration tests
- Security scans
- Build verification

Check `.github/workflows/` for pipeline definitions.

### Pre-commit Hooks

Install pre-commit hooks:
```bash
pip install pre-commit
pre-commit install
```

Hooks run automatically on `git commit`:
- Code formatting
- Linting
- Type checking
- Test execution

## Best Practices

1. **Write Tests First**: Follow TDD when possible
2. **Keep PRs Small**: Easier to review and merge
3. **Document As You Go**: Don't leave documentation for later
4. **Use Descriptive Names**: Make code self-documenting
5. **Avoid Premature Optimization**: Make it work, then make it fast
6. **Review Your Own Code**: Review changes before submitting PR
7. **Communicate Early**: Discuss significant changes before implementing

## Resources

- [Git Subtree Documentation](https://www.atlassian.com/git/tutorials/git-subtree)
- [Docker Documentation](https://docs.docker.com/)
- [Python Best Practices](https://docs.python-guide.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [FastAPI Documentation](https://fastapi.tiangolo.com/)

## Getting Help

- Check component-specific README files
- Review architecture documentation
- Ask questions in GitHub Discussions
- Join our developer community (link TBD)

## Next Steps

1. Explore the codebase
2. Run the examples
3. Pick an issue from GitHub
4. Make your first contribution!

Happy coding! 🚀
