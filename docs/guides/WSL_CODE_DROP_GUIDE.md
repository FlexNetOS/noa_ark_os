# WSL Code Drop Guide - Option 3 Execution Plan

## 🔍 WSL Directory Discovery Results

Based on exploration of `/home/deflex/workspace/`, I've identified the following code repositories:

### **Priority 1: Core Tools & Utilities** 🔥
**Location**: `/home/deflex/workspace/tools/`
**Size**: ~140KB (manageable first drop)
**Contents**:
- `agent` - Agent execution script
- `github_coding_agent.sh` - GitHub integration
- `ollama_manager.sh` - AI model management
- `build_code_deps.py`, `build_code_index.py` - Build utilities
- Subdirectories: `bin/`, `cli/`, `merge-polish/`, `scripts/`, `utilities/`

**Why First?**: Small, focused utilities that integrate easily. Contains agent management tools that align with your existing agent system.

**Copy Command**:
```bash
# From Windows PowerShell/WSL
wsl cp -r /home/deflex/workspace/tools/ /mnt/d/dev/workspaces/noa_ark_os/crc/drop-in/incoming/stale/
```

---

### **Priority 2: Agent Registry System** 🤖
**Location**: `/home/deflex/workspace/agents/agent-registry/`
**Size**: ~50KB
**Contents**:
- `main.go` (22.6KB) - Go-based agent registry
- `Cargo.toml`, `src/` - Rust implementation
- `Dockerfile` - Container deployment

**Why Second?**: Complements the existing agent CSV directory already integrated. Provides runtime registry capabilities.

**Copy Command**:
```bash
wsl cp -r /home/deflex/workspace/agents/agent-registry/ /mnt/d/dev/workspaces/noa_ark_os/crc/drop-in/incoming/repos/agent-registry
```

---

### **Priority 3: Server Components** 🌐
**Location**: `/home/deflex/workspace/server/`
**Size**: Small (3 subdirectories)
**Contents**:
- `caddy/` - Web server configuration
- `mcp/` - Model Context Protocol
- `vault/` - Secret management

**Why Third?**: Server infrastructure components. Your Windows workspace already has `server/` directory - this adds WSL-specific configs.

**Copy Command**:
```bash
wsl cp -r /home/deflex/workspace/server/ /mnt/d/dev/workspaces/noa_ark_os/crc/drop-in/incoming/stale/server-wsl
```

---

### **Priority 4: Task Execution Kit** 📋
**Location**: `/home/deflex/workspace/task_exec_kit/`
**Size**: Tiny (~200 bytes README, 2 subdirs)
**Contents**:
- `flows/` - Task execution flows
- `templates/` - Task templates
- `README.md`

**Why Fourth?**: Lightweight task execution framework. Complements your existing workflow system.

**Copy Command**:
```bash
wsl cp -r /home/deflex/workspace/task_exec_kit/ /mnt/d/dev/workspaces/noa_ark_os/crc/drop-in/incoming/repos/
```

---

### **Priority 5: Full WSL NOA Workspace** 🏗️ (LARGE - SELECTIVE EXTRACTION)
**Location**: `/home/deflex/workspace/noa_ark_os/`
**Size**: 800KB+ (massive, includes many completion docs)
**Contents**: Complete parallel NOA workspace with extensive completion documentation

**⚠️ TOO LARGE - Extract Selectively**:

#### 5a. Extract Unique Source Code Only (Skip Docs)
```bash
# Count source files first
wsl bash -c "find /home/deflex/workspace/noa_ark_os -type f \( -name '*.rs' -o -name '*.go' -o -name '*.py' -o -name '*.sh' \) | wc -l"

# List source files (determine size)
wsl bash -c "find /home/deflex/workspace/noa_ark_os -type f \( -name '*.rs' -o -name '*.go' -o -name '*.py' \) | head -50"
```

#### 5b. Extract Key Subdirectories Only
Identify unique directories not present in Windows workspace:
```bash
# Compare directories
wsl ls -1 /home/deflex/workspace/noa_ark_os/ > D:\temp\wsl_noa_dirs.txt
ls D:\dev\workspaces\noa_ark_os\ > D:\temp\windows_noa_dirs.txt
# Manually compare to find WSL-only directories
```

---

## 📊 Additional Exploration Commands

### Find Code Files by Type
```bash
# Rust files
wsl bash -c "find /home/deflex/workspace -name '*.rs' -type f | head -30"

# Go files
wsl bash -c "find /home/deflex/workspace -name '*.go' -type f | head -30"

# Python files  
wsl bash -c "find /home/deflex/workspace -name '*.py' -type f | head -30"

# Shell scripts
wsl bash -c "find /home/deflex/workspace -name '*.sh' -type f | head -30"
```

### Check Directory Sizes
```bash
wsl du -sh /home/deflex/workspace/tools/
wsl du -sh /home/deflex/workspace/agents/
wsl du -sh /home/deflex/workspace/server/
wsl du -sh /home/deflex/workspace/noa_ark_os/
```

### Explore Other Discovered Directories
- `/home/deflex/workspace/computer_operator/`
- `/home/deflex/workspace/backend/`
- `/home/deflex/workspace/api/`
- `/home/deflex/workspace/kernel/`

---

## 🔄 Iterative Drop Workflow

### Cycle 1: Tools Drop (START HERE)
1. **Copy**: `wsl cp -r /home/deflex/workspace/tools/ /mnt/d/dev/workspaces/noa_ark_os/crc/drop-in/incoming/stale/`
2. **Verify**: Check files arrived in Windows
3. **Manifest**: Create `MANIFEST.md` describing drop:
   ```markdown
   # Tools Drop from WSL
   
   **Source**: /home/deflex/workspace/tools/
   **Date**: [Current Date]
   **Size**: ~140KB
   **Purpose**: Core build, agent, and utility scripts
   
   ## Key Files
   - agent execution scripts
   - github_coding_agent.sh
   - build utilities (Python)
   - CLI tools
   ```
4. **Process**: Wait for CRC analysis (or manually run if automated)
5. **Integrate**: Move valuable code into appropriate workspace locations
6. **Archive**: Compress and archive original drop

### Cycle 2-5: Repeat for Other Priorities
Follow same workflow for agent-registry, server-wsl, task_exec_kit, and selective noa_ark_os extracts.

---

## 🛠️ Cargo Watcher Fix (Parallel Task)

### Problem
```
cargo-clippy.exe is not installed for the custom toolchain 
'D:\dev\workspaces\noa_ark_os\server\tools\rustup-portable\toolchains\stable-x86_64-pc-windows-msvc'
```

### Solution Options

#### Option A: Install Clippy Manually (RECOMMENDED)
```powershell
# Download clippy binaries from official source
# Extract to: D:\dev\workspaces\noa_ark_os\server\tools\rustup-portable\toolchains\stable-x86_64-pc-windows-msvc\bin\

# Files needed:
# - cargo-clippy.exe
# - clippy-driver.exe
```

#### Option B: Disable Clippy in VS Code (QUICK FIX)
Edit `.vscode/settings.json`:
```json
{
  "rust-analyzer.checkOnSave.command": "check",
  "rust-analyzer.checkOnSave.extraArgs": []
}
```

#### Option C: Build Clippy from Source (TIME-CONSUMING)
```powershell
# In WSL where full Rust toolchain exists
rustup component add clippy
# Copy binaries to portable Windows toolchain
```

**Recommendation**: Start with Option B (quick), then pursue Option A when time permits.

---

## 📈 Success Metrics

### Drop Success Indicators
- ✅ Files copied successfully to Windows filesystem
- ✅ MANIFEST.md created describing drop contents
- ✅ CRC analysis completed without errors
- ✅ Code compiles after integration
- ✅ Tests pass (if applicable)
- ✅ Original drop archived

### Integration Success Indicators  
- ✅ Code moved from drop-in to permanent workspace location
- ✅ Cargo.toml updated (if Rust code)
- ✅ Dependencies resolved
- ✅ Examples/tests created
- ✅ Documentation updated

---

## 🚀 Next Steps

1. **START NOW**: Execute Cycle 1 (Tools Drop)
   ```bash
   wsl cp -r /home/deflex/workspace/tools/ /mnt/d/dev/workspaces/noa_ark_os/crc/drop-in/incoming/stale/
   ```

2. **PARALLEL**: Apply Cargo Watcher Quick Fix (Option B)

3. **VERIFY**: Check tools/ arrived in Windows CRC drop-in

4. **CREATE**: Write tools/MANIFEST.md describing drop

5. **ANALYZE**: I'll help analyze the tools code once dropped

6. **INTEGRATE**: Move valuable utilities into workspace

7. **REPEAT**: Move to Cycle 2 (agent-registry)

---

## 🔍 Additional Discovery Needed

Before proceeding with large drops, run these commands to gather more info:

```bash
# Check for unique WSL-only code in noa_ark_os/
wsl bash -c "find /home/deflex/workspace/noa_ark_os -name 'Cargo.toml' | head -20"

# Explore computer_operator/
wsl ls -la /home/deflex/workspace/computer_operator/

# Explore backend/
wsl ls -la /home/deflex/workspace/backend/

# Explore api/
wsl ls -la /home/deflex/workspace/api/
```

I'll update this guide as we discover more valuable code locations.

---

**Status**: Ready to begin Cycle 1 - awaiting your `cp` command execution.
