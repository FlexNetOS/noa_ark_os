# Next Moves Execution Plan

**Date**: Current  
**Status**: Ready to Execute  
**Priority**: High  

---

## 🎯 **Overview**

Two major initiatives to move the project forward:

1. **Convert FlexNetOS Fork Repos** → Integrate external forks into workspace
2. **Set Up Llama.cpp** → Enable local LLM inference for agents

---

## 📋 **Move 1: Convert Fork Repos to Branches**

### **Objective**
Integrate all fork repositories from the FlexNetOS GitHub organization into the main workspace using the fork processing system.

### **Prerequisites**
- ✅ Git installed and configured
- ✅ GitHub access to FlexNetOS organization
- ✅ Fork processing system operational
- ✅ Sufficient disk space (~500MB per fork)

### **Execution Steps**

#### **Step 1: List Available Forks** (5 minutes)

```powershell
cd D:\dev\workspaces\noa_ark_os

# List all forks in FlexNetOS organization
.\scripts\integration\process-github-forks.ps1 -ListOnly
```

**Expected Output**: List of all fork repositories

#### **Step 2: Process All Forks** (30-60 minutes)

```powershell
# Process all forks automatically
.\scripts\integration\process-github-forks.ps1

# Or process specific forks
.\scripts\integration\process-github-forks.ps1 -ForkNames "noa_ark_os-agents", "noa_ark_os-ui"
```

**What Happens**:
1. Clones each fork to `crc/drop-in/incoming/forks/`
2. Removes `.git` directory
3. Processes through fork detection system
4. Creates isolated `fork/{name}` branch for each
5. Generates metadata

#### **Step 3: Review Fork Branches** (15 minutes)

```powershell
# List all fork branches
git branch -a | Select-String "fork/"

# Check metadata for each fork
.\crc\detect-forks.ps1 -Mode list

# Review specific fork
git checkout fork/noa_ark_os-agents
git log --oneline
git diff main
```

#### **Step 4: Selective Integration** (Per Fork: 10-20 minutes)

For each valuable fork:

```powershell
# Switch to fork branch
git checkout fork/fork-name

# Test if it builds
cargo build
cargo test

# If good, merge to main
git checkout main
git merge fork/fork-name --no-ff -m "integrate: Fork fork-name"

# Or cherry-pick specific commits
git checkout main
git cherry-pick <commit-hash>

# Push to GitHub
git push origin main
```

### **Success Criteria**
- ✅ All forks cloned and processed
- ✅ Fork branches created
- ✅ Metadata generated
- ✅ Useful code integrated
- ✅ Build still passes

---

## 🤖 **Move 2: Set Up Llama.cpp**

### **Objective**
Install and configure llama.cpp for local LLM inference to power the agent system.

### **Prerequisites**
- ✅ ~4GB disk space for model
- ✅ 8GB+ RAM (16GB recommended)
- ✅ Optional: NVIDIA GPU with CUDA for acceleration

### **Execution Steps**

#### **Step 1: Install Llama.cpp** (15 minutes)

```powershell
cd D:\dev\workspaces\noa_ark_os

# Install with default model (Llama 3.2 3B)
.\scripts\dev\setup-llama-cpp.ps1

# Or specify model size
.\scripts\dev\setup-llama-cpp.ps1 -ModelSize 7b

# Or build from source (advanced)
.\scripts\dev\setup-llama-cpp.ps1 -BuildFromSource
```

**What Happens**:
1. Creates directory structure
2. Downloads llama.cpp binaries
3. Downloads model (~2-5GB depending on size)
4. Creates configuration file
5. Creates start script

#### **Step 2: Start Llama.cpp Server** (2 minutes)

```powershell
# Start the server
.\scripts\dev\start-llama-server.ps1

# Server will run on http://127.0.0.1:8080
```

**Keep this terminal open** or run in background.

#### **Step 3: Test Server** (5 minutes)

In a new terminal:

```powershell
# Test health check
Invoke-RestMethod -Uri "http://127.0.0.1:8080/health"

# Test completion
$body = @{
    prompt = "Write a hello world in Rust"
    temperature = 0.7
    max_tokens = 200
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://127.0.0.1:8080/completion" -Method Post -Body $body -ContentType "application/json"
```

**Expected**: Server responds with completion

#### **Step 4: Create Rust Inference Client** (30 minutes)

```powershell
# Create inference crate
cd server/ai
cargo new inference --lib

# Add dependencies (already in guide)
# Implement client code (see LLAMA_CPP_SETUP.md)

# Test client
cd inference
cargo test
```

#### **Step 5: Integrate with Agents** (60 minutes)

```rust
// Update agents/src/lib.rs
pub mod inference;

// Add inference trait (see guide)
// Update agent implementations to use inference
```

```powershell
# Build agents with inference
cargo build -p noa_agents

# Test inference integration
cargo test -p noa_agents --test inference_tests
```

#### **Step 6: Update Agent Flow** (30 minutes)

Review and update agent execution flow:

1. **DigestAgent** - Use inference for code analysis
2. **TaskAgent** - Use inference for task planning
3. **CodeAgent** - Use inference for code generation

```rust
// Example: DigestAgent with inference
impl DigestAgent {
    pub async fn analyze(&self, code: &str) -> Result<Digest> {
        let prompt = format!("Analyze this code:\n\n{}", code);
        let response = self.inference.generate(&prompt, config).await?;
        Ok(parse_digest(&response)?)
    }
}
```

### **Success Criteria**
- ✅ Llama.cpp server running
- ✅ Server responding to requests
- ✅ Rust client functional
- ✅ Agents using inference
- ✅ Tests passing

---

## 📊 **Timeline & Effort**

### **Move 1: Fork Integration**
- **List forks**: 5 minutes
- **Process forks**: 30-60 minutes (automated)
- **Review branches**: 15 minutes
- **Integration**: 10-20 minutes per fork
- **Total**: 1-3 hours (depending on fork count)

### **Move 2: Llama.cpp Setup**
- **Installation**: 15 minutes
- **Server start**: 2 minutes
- **Testing**: 5 minutes
- **Client implementation**: 30 minutes
- **Agent integration**: 60 minutes
- **Flow update**: 30 minutes
- **Total**: 2-3 hours

### **Combined Timeline**
- **Parallel execution**: 2-3 hours (run both simultaneously)
- **Sequential execution**: 3-6 hours
- **Recommendation**: Do llama.cpp first, then forks

---

## 🚀 **Quick Start Commands**

### **For Fork Integration**

```powershell
cd D:\dev\workspaces\noa_ark_os

# 1. List available forks
.\scripts\integration\process-github-forks.ps1 -ListOnly

# 2. Process all forks
.\scripts\integration\process-github-forks.ps1

# 3. Review and integrate
git branch -a | Select-String "fork/"
.\crc\detect-forks.ps1 -Mode list
```

### **For Llama.cpp Setup**

```powershell
cd D:\dev\workspaces\noa_ark_os

# 1. Install llama.cpp
.\scripts\dev\setup-llama-cpp.ps1

# 2. Start server (keep terminal open)
.\scripts\dev\start-llama-server.ps1

# 3. Test (in new terminal)
Invoke-RestMethod -Uri "http://127.0.0.1:8080/health"

# 4. Build inference client
cd server/ai/inference
cargo build
cargo test
```

---

## 📝 **Documentation References**

### **Created Guides**
1. ✅ `scripts/integration/process-github-forks.ps1` - Fork processing automation
2. ✅ `docs/guides/LLAMA_CPP_SETUP.md` - Complete llama.cpp guide
3. ✅ `scripts/dev/setup-llama-cpp.ps1` - Installation script
4. ✅ This file - Execution plan

### **Existing Guides**
- `crc/FORK_PROCESSING_SYSTEM.md` - Fork system architecture
- `docs/reports/analysis/MERGE_STRATEGY_GUIDE.md` - Fork merge strategies
- `WORKSPACE_MEMORY.md` - Workspace context

---

## ⚠️ **Important Notes**

### **Fork Processing**
- ⚠️ Each fork takes ~100MB disk space
- ⚠️ Processing time depends on fork size
- ⚠️ Review changes before merging
- ⚠️ Test build after integration

### **Llama.cpp**
- ⚠️ Model download requires stable internet
- ⚠️ GPU acceleration requires CUDA
- ⚠️ 8GB+ RAM recommended
- ⚠️ Keep server running for inference

---

## ✅ **Success Validation**

### **After Fork Integration**

```powershell
# All forks processed
.\crc\detect-forks.ps1 -Mode list | Measure-Object

# Fork branches created
git branch -a | Select-String "fork/" | Measure-Object

# Build still works
cargo build --workspace
cargo test --workspace
```

### **After Llama.cpp Setup**

```powershell
# Server running
Invoke-RestMethod -Uri "http://127.0.0.1:8080/health"

# Inference client works
cargo test -p noa_inference

# Agents using inference
cargo test -p noa_agents --test inference_tests
```

---

## 🎯 **Next Steps After Completion**

1. ✅ Update agent implementations with LLM inference
2. ✅ Create agent prompt templates
3. ✅ Implement agent orchestration
4. ✅ Set up agent evaluation metrics
5. ✅ Create agent test suite

---

**Status**: Ready to execute  
**Recommendation**: Start with llama.cpp setup, then process forks  
**Estimated Time**: 3-6 hours total  
**Prerequisites**: All met ✅  

---

## 🚀 **Let's Begin!**

Choose your starting point:

**Option A: Llama.cpp First** (Recommended)
```powershell
.\scripts\dev\setup-llama-cpp.ps1
```

**Option B: Fork Integration First**
```powershell
.\scripts\integration\process-github-forks.ps1 -ListOnly
```

**Option C: Both in Parallel**
```powershell
# Terminal 1: Start llama.cpp installation
.\scripts\dev\setup-llama-cpp.ps1

# Terminal 2: Process forks while installation runs
.\scripts\integration\process-github-forks.ps1
```

Ready to proceed? 🚀
