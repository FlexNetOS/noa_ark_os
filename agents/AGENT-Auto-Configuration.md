# AI Development Team 🚀

**Supercharge AI Development with a team of specialized AI agents** that work
together to build complete features, debug complex issues, and handle any
technology stack with expert-level knowledge.

## ⚠️ Important Notice - **NEED TO FIX**

**This project is experimental and token-intensive.** I'm actively testing these
agents with Claude subscription - expect high token consumption during complex
workflows. Multi-agent orchestration can consume 10-50k tokens per complex
feature. Use with caution and monitor your usage.

## 🚀 Quick Start (3 Minutes)

### Prerequisites (Provider Example)

- **Claude Code CLI** installed and authenticated
- **Claude subscription** - required for intensive agent workflows
- Active project directory with your codebase
- **Optional**: [Context7 MCP](docs/dependencies.md) for enhanced documentation
  access

### 1. Install the Agents **Skip this step if already installed**

```bash
git clone https://github.com/FlexNetOS/noa-server.git
```

#### Option A: Symlink (Recommended - auto-updates)

**macOS/Linux:**

```bash
# Create agents directory if it doesn't exist (preserves existing agents)
mkdir -p ~/.claude/agents

# Symlink the agents collection
ln -sf "$(pwd)/noa-server/agents/" ~/.claude/agents/noa-server
```

**Windows (PowerShell):**

```powershell
# Create agents directory
New-Item -Path "$env:USERPROFILE\.claude\agents" -ItemType Directory -Force

# Create symlink
cmd /c mklink /D "$env:USERPROFILE\.claude\agents\noa-server" "$(Get-Location)\noa-server\agents"
```

#### Option B: Copy (Static - no auto-updates)

```bash
# Create agents directory if it doesn't exist
mkdir -p ~/.claude/agents

# Copy all agents
cp -r noa-server/agents ~/.claude/agents/noa-server
```

### 2. Verify Installation

```bash
claude /agents
# Should show all 24 agents.
```

### 3. Initialize Your Project

**Navigate** to your **project directory** and run the following command to
configure your AI team:

```bash
claude "use @agent-team-configurator and optimize my project to best use the available subagents."
```

### 4. Start Building

```bash
claude "use @agent-tech-lead-orchestrator and build a user authentication system"
```

Your AI team will automatically detect your stack and use the right specialists!

## 🎯 How Auto-Configuration Works

The @agent-team-configurator automatically sets up your perfect AI development
team. When invoked, it:

1. **Locates CLAUDE.md** - Finds existing project configuration and preserves
   all your custom content outside the "AI Team Configuration" section
2. **Detects Technology Stack** - Inspects package.json, composer.json,
   requirements.txt, go.mod, Gemfile, and build configs to understand your
   project
3. **Discovers Available Agents** - Scans ~/.claude/agents/ and .claude/
   folders, building a capability table of all available specialists
4. **Selects Specialists** - Prefers framework-specific agents over universal
   ones, always includes @agent-code-reviewer and @agent-performance-optimizer
   for quality assurance
5. **Updates CLAUDE.md** - Creates a timestamped "AI Team Configuration" section
   with your detected stack and a Task|Agent|Notes mapping table
6. **Provides Usage Guidance** - Shows you the detected stack, selected agents,
   and gives sample commands to start building
