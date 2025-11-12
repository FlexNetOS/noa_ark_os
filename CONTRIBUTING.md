# Contributing to noa_ark_os

Thank you for your interest in contributing to noa_ark_os! This document provides guidelines and instructions for contributing.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [How to Contribute](#how-to-contribute)
4. [Development Workflow](#development-workflow)
5. [Coding Standards](#coding-standards)
6. [Testing Guidelines](#testing-guidelines)
7. [Documentation](#documentation)
8. [Pull Request Process](#pull-request-process)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors.

### Expected Behavior

- Be respectful and considerate
- Be collaborative and constructive
- Be patient with new contributors
- Focus on what is best for the community

### Unacceptable Behavior

- Harassment, discrimination, or intimidation
- Trolling or inflammatory comments
- Public or private harassment
- Publishing others' private information

## Getting Started

### Prerequisites

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/noa_ark_os.git
   cd noa_ark_os
   ```

3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/FlexNetOS/noa_ark_os.git
   ```

4. Install the GitHub CLI if it is not already available:

   ```bash
   ./scripts/install_gh_cli.sh
   ```

   The script requires `sudo` privileges and supports Debian/Ubuntu environments (used by CI and the development container).
   For other platforms follow the [official installation guide](https://cli.github.com/manual/installation).

5. Authenticate the GitHub CLI (required for `gh` commands and automation scripts):
   1. Create a fine-grained personal access token (PAT) with **`repo`**, **`workflow`**, and **`project`** read/write scopes.
   2. Run an interactive login:

      ```bash
      gh auth login --hostname github.com --git-protocol https
      ```

      Select GitHub.com → HTTPS → “Paste an authentication token” and paste the PAT when prompted.
   3. For non-interactive environments, export the token before running scripts:

      ```bash
      export GH_TOKEN="<your-token>"
      export GITHUB_TOKEN="$GH_TOKEN"
      gh auth status --hostname github.com
      ```

      The status command should report “Logged in to github.com”.

6. Install development dependencies:
   ```bash
   # Python components
   pip install -r requirements-dev.txt

   # Rust components
   cd repos/agentaskit && cargo build
   ```

5. Create a fine-grained personal access token (PAT) following [GitHub's official documentation](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens). Grant the token the scopes required to read and write to this repository and set an expiration reminder so you can renew it before it lapses.

6. Authenticate the GitHub CLI using the token:
   ```bash
   gh auth login --hostname github.com --with-token
   ```
   When prompted, paste the PAT you created in the previous step.

7. For non-interactive environments (CI jobs, scripts, etc.), export the token so the GitHub CLI can reuse it without prompts, and verify your session:
   ```bash
   export GH_TOKEN="<your-token>"
   export GITHUB_TOKEN="<your-token>"
   gh auth status --hostname github.com
   ```

> **Troubleshooting:** If you see `GitHub authentication is required` while running `gh` commands, your session has expired or was never initialized. Re-run steps 5–7 to refresh your credentials.
#### Troubleshooting GitHub CLI authentication

- Error: `GitHub authentication is required` → rerun the login command in step 4 or refresh your PAT and re-export `GH_TOKEN`/`GITHUB_TOKEN`.
- Error persists in CI or automation → confirm the token has not expired and still includes the `repo` and `workflow` scopes.

### Automating AGENTOS roadmap issue creation

Run `scripts/create_agentos_tasks.py` to convert the six AGENTOS roadmap entries in `docs/plans/gap_remediation_tasks.md` into live GitHub issues. The helper defaults to a dry-run so you can preview the generated payloads:

```bash
scripts/create_agentos_tasks.py --dry-run
```

When you are ready to create (or reuse) issues in this repository and append the URLs next to the in-document anchors, execute:

```bash
scripts/create_agentos_tasks.py --repo FlexNetOS/noa_ark_os --execute --update-doc
```

The script expects the GitHub CLI to be authenticated (see the prerequisites above) and will reuse existing issues whose titles already match `AGENTOS-# — …`.

## How to Contribute

### Types of Contributions

1. **Bug Reports**: Report issues you encounter
2. **Feature Requests**: Suggest new features
3. **Code Contributions**: Submit pull requests
4. **Documentation**: Improve docs and examples
5. **Testing**: Add or improve tests
6. **Reviews**: Review pull requests

### Reporting Bugs

**Before submitting:**
- Check existing issues
- Verify it's reproducible
- Gather relevant information

**Bug Report Template:**
```markdown
**Description:**
Clear description of the bug

**Steps to Reproduce:**
1. Step 1
2. Step 2
3. ...

**Expected Behavior:**
What should happen

**Actual Behavior:**
What actually happens

**Environment:**
- OS: Ubuntu 20.04
- Python: 3.9
- Component: MicroAgentStack
- Version: 1.0.0

**Additional Context:**
Any other relevant information
```

### Suggesting Features

**Feature Request Template:**
```markdown
**Feature Description:**
Clear description of the feature

**Use Case:**
Why is this feature needed?

**Proposed Solution:**
How should it work?

**Alternatives Considered:**
Other approaches you've thought about

**Additional Context:**
Any other relevant information
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b bugfix/issue-number-description
```

Branch naming conventions:
- `feature/` - New features
- `bugfix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/changes

### 2. Make Your Changes

- Write clean, readable code
- Follow coding standards
- Add tests for new functionality
- Update documentation as needed

### 3. Commit Your Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git add .
git commit -m "feat(component): add new feature"
```

**Commit Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding missing tests
- `chore`: Changes to build process or auxiliary tools

**Examples:**
```bash
git commit -m "feat(orchestrator): add agent health monitoring"
git commit -m "fix(pipeline): resolve memory leak in processor"
git commit -m "docs(api): update authentication examples"
```

### 4. Keep Your Branch Updated

```bash
git fetch upstream
git rebase upstream/main
```

### 5. Push Your Changes

```bash
git push origin feature/your-feature-name
```

## Coding Standards

### Python (PEP 8)

```python
"""Module docstring."""

from typing import List, Dict, Optional


class AgentManager:
    """Manages agent lifecycle and operations.
    
    Attributes:
        agents: Dictionary of active agents
        max_agents: Maximum number of concurrent agents
    """
    
    def __init__(self, max_agents: int = 10):
        """Initialize AgentManager.
        
        Args:
            max_agents: Maximum number of concurrent agents
        """
        self.agents: Dict[str, Agent] = {}
        self.max_agents = max_agents
    
    def create_agent(self, name: str, agent_type: str) -> Agent:
        """Create a new agent.
        
        Args:
            name: Agent name
            agent_type: Type of agent to create
            
        Returns:
            Created Agent instance
            
        Raises:
            ValueError: If max_agents limit reached
        """
        if len(self.agents) >= self.max_agents:
            raise ValueError("Max agents limit reached")
        
        agent = Agent(name, agent_type)
        self.agents[agent.id] = agent
        return agent
```

**Style Guidelines:**
- Use type hints
- Write docstrings for all public functions/classes
- Use meaningful variable names
- Keep functions focused and small
- Maximum line length: 88 characters (Black formatter)

### Rust

```rust
//! Module documentation.

use std::collections::HashMap;

/// Manages agent lifecycle and operations.
pub struct AgentManager {
    agents: HashMap<String, Agent>,
    max_agents: usize,
}

impl AgentManager {
    /// Creates a new AgentManager.
    ///
    /// # Arguments
    ///
    /// * `max_agents` - Maximum number of concurrent agents
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = AgentManager::new(10);
    /// ```
    pub fn new(max_agents: usize) -> Self {
        Self {
            agents: HashMap::new(),
            max_agents,
        }
    }
    
    /// Creates a new agent.
    ///
    /// # Arguments
    ///
    /// * `name` - Agent name
    /// * `agent_type` - Type of agent
    ///
    /// # Returns
    ///
    /// Result containing the created Agent or an error
    pub fn create_agent(&mut self, name: String, agent_type: String) 
        -> Result<Agent, String> {
        if self.agents.len() >= self.max_agents {
            return Err("Max agents limit reached".to_string());
        }
        
        let agent = Agent::new(name, agent_type);
        self.agents.insert(agent.id.clone(), agent.clone());
        Ok(agent)
    }
}
```

**Style Guidelines:**
- Use rustfmt for formatting: `cargo fmt`
- Use clippy for linting: `cargo clippy`
- Write documentation comments (///)
- Use Result for error handling
- Follow Rust naming conventions

## Testing Guidelines

### Python Testing

```python
import pytest
from agent_manager import AgentManager


class TestAgentManager:
    """Test suite for AgentManager."""
    
    def test_create_agent_success(self):
        """Test successful agent creation."""
        manager = AgentManager(max_agents=5)
        agent = manager.create_agent("test", "executor")
        
        assert agent.name == "test"
        assert agent.agent_type == "executor"
        assert len(manager.agents) == 1
    
    def test_create_agent_limit_reached(self):
        """Test agent creation when limit is reached."""
        manager = AgentManager(max_agents=1)
        manager.create_agent("test1", "executor")
        
        with pytest.raises(ValueError, match="Max agents limit reached"):
            manager.create_agent("test2", "executor")
    
    @pytest.fixture
    def sample_agent(self):
        """Fixture providing a sample agent."""
        return Agent("sample", "executor")
```

### Rust Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_agent_success() {
        let mut manager = AgentManager::new(5);
        let result = manager.create_agent(
            "test".to_string(),
            "executor".to_string()
        );
        
        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.name, "test");
        assert_eq!(agent.agent_type, "executor");
    }

    #[test]
    fn test_create_agent_limit_reached() {
        let mut manager = AgentManager::new(1);
        manager.create_agent("test1".to_string(), "executor".to_string()).unwrap();
        
        let result = manager.create_agent("test2".to_string(), "executor".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Max agents limit reached");
    }
}
```

### Running Tests

```bash
# Python tests
pytest tests/ -v --cov=.

# Rust tests
cargo test

# Integration tests
./scripts/run-integration-tests.sh
```

## Documentation

### Code Documentation

- Add docstrings/doc comments to all public APIs
- Include examples in documentation
- Document parameters, return values, and exceptions
- Keep documentation up to date with code changes

### README Updates

Update component README when:
- Adding new features
- Changing installation steps
- Updating usage examples
- Modifying configuration options

### Architecture Documentation

Update architecture docs when:
- Adding new components
- Changing system design
- Modifying data flow
- Updating deployment strategy

## Pull Request Process

### Before Submitting

1. **Update from upstream:**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run tests:**
   ```bash
   pytest tests/
   cargo test
   ```

3. **Run linters:**
   ```bash
   black .
   flake8 .
   cargo fmt
   cargo clippy
   ```

4. **Update documentation:**
   - Update relevant README files
   - Add/update docstrings
   - Update CHANGELOG.md

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Changes Made
- Change 1
- Change 2
- Change 3

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] CHANGELOG.md updated
- [ ] No breaking changes (or properly documented)

## Related Issues
Closes #123
```

### Review Process

1. **Automated Checks:**
   - CI/CD pipeline must pass
   - All tests must pass
   - No linting errors

2. **Code Review:**
   - At least one approval required
   - Address all comments
   - Keep discussion professional

3. **Merge:**
   - Squash commits if requested
   - Update commit message if needed
   - Merge when approved and checks pass

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project documentation

Thank you for contributing to noa_ark_os! 🎉
