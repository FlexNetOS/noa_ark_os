# CRC Drop-In Quickstart Guide

**For Manual Integration (Current State)**

---

## 🎯 Overview

The CRC (Continuous ReCode) system is currently in **manual integration mode**. The full automation pipeline is under development (Option B), but you can manually drop and integrate code right now (Option A).

**Current Flow**:
```
You drop code → You notify me → I load & integrate → Code ready to use
```

**Future Flow** (Option B):
```
You drop code → CRC auto-processes → CI/CD validates → Auto-deploy
```

---

## 📂 Where to Drop Your Code

### Directory Structure

```
crc/drop-in/incoming/
├── repos/      ← Fresh, actively maintained repositories
├── forks/      ← Forked projects with your modifications
├── mirrors/    ← Mirror/clone repositories
└── stale/      ← Old, unmaintained, or abandoned code
```

**Choose the right folder based on your code:**

| Code Type | Folder | Example |
|-----------|--------|---------|
| Active GitHub repo (< 6 months old) | `repos/` | `my-active-project/` |
| Fork you modified | `forks/` | `my-fork-of-something/` |
| Mirror from another source | `mirrors/` | `gitlab-mirror/` |
| Old code (> 1 year, unmaintained) | `stale/` | `legacy-app/` |

---

## 🚀 Quick Drop Instructions

### Option 1: Drop a Folder

```powershell
# Windows PowerShell
Copy-Item -Recurse "C:\your-code\project-name" `
  "D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\stale\project-name"
```

```bash
# Linux/WSL
cp -r ~/your-code/project-name \
  ~/noa_ark_os/crc/drop-in/incoming/stale/project-name
```

### Option 2: Drop a ZIP/Archive

```powershell
# Windows PowerShell
Copy-Item "C:\your-code\project.zip" `
  "D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\stale\"
```

```bash
# Linux/WSL
cp ~/your-code/project.zip \
  ~/noa_ark_os/crc/drop-in/incoming/stale/
```

### Option 3: Clone Directly

```bash
# Navigate to incoming folder
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\repos

# Clone the repository
git clone https://github.com/user/repo.git
```

---

## 📝 Create a Manifest (Optional but Helpful)

After dropping your code, create a `manifest.json` to help with integration:

**Location**: `crc/drop-in/incoming/{type}/{your-code}/manifest.json`

**Example**:
```json
{
  "name": "my-legacy-app",
  "source_type": "stale",
  "priority": "normal",
  "analysis": {
    "Language": "Rust",
    "Description": "Old authentication service from 2020",
    "LastCommit": "2020-06-15",
    "Dependencies": ["tokio", "serde", "reqwest"]
  },
  "integration_notes": {
    "must_have": ["auth module", "user management"],
    "nice_to_have": ["admin dashboard"],
    "skip": ["old tests", "deprecated APIs"]
  }
}
```

**Manifest Fields** (all optional):
- `name`: Friendly name for the drop
- `source_type`: repos/forks/mirrors/stale
- `priority`: normal/high/low
- `analysis.Language`: Primary programming language
- `analysis.Description`: What this code does
- `analysis.LastCommit`: When it was last updated
- `integration_notes.must_have`: What features you definitely need
- `integration_notes.nice_to_have`: Optional features
- `integration_notes.skip`: What to ignore

---

## ✅ What Happens Next

### Manual Integration Process

1. **You drop the code** (see above)
2. **You notify me**: "I dropped {name} in {folder}"
3. **I analyze the code**:
   - Review structure
   - Identify key components
   - Check dependencies
   - Assess integration effort
4. **I create integration plan**:
   - What to integrate
   - What to adapt
   - What to skip
   - Phase approach
5. **I integrate the code**:
   - Extract key files
   - Adapt to NOA conventions
   - Update imports/dependencies
   - Create tests
   - Update documentation
6. **Code ready to use!**

### Timeline

| Code Size | Analysis | Integration | Total |
|-----------|----------|-------------|-------|
| Small (< 1000 lines) | 15 min | 1-2 hours | ~2 hours |
| Medium (1K-10K lines) | 30 min | 2-8 hours | ~1 day |
| Large (> 10K lines) | 1 hour | 8-40 hours | ~1 week |

---

## 💡 Best Practices

### DO ✅

1. **Use clear folder names**
   ```
   Good: legacy-auth-service/
   Bad:  code/, stuff/, temp/
   ```

2. **Include documentation**
   - README.md explaining what it does
   - Architecture diagrams if available
   - Original documentation

3. **Add a manifest.json**
   - Helps me understand your priorities
   - Speeds up integration
   - Ensures I focus on what you need

4. **One project per folder**
   ```
   Good: incoming/stale/auth-service/
         incoming/stale/payment-api/
   Bad:  incoming/stale/all-my-code/auth/
         incoming/stale/all-my-code/payment/
   ```

5. **Notify me after dropping**
   - "I dropped auth-service in stale/"
   - Provides context
   - Lets me prioritize

### DON'T ❌

1. **Don't drop without structure**
   ```
   Bad: incoming/stale/file1.rs
        incoming/stale/file2.rs
        incoming/stale/random.txt
   ```

2. **Don't drop sensitive data**
   - Remove API keys, passwords, tokens
   - Remove `.env` files with secrets
   - Remove database credentials
   - Remove SSH keys

3. **Don't drop binaries**
   - Source code only
   - No compiled `.exe`, `.dll`, `.so` files
   - Exception: Small utilities if essential

4. **Don't drop huge files**
   - Keep individual drops < 500 MB
   - Split large projects if needed
   - Remove unnecessary assets

5. **Don't drop random stuff**
   - Only code you want integrated
   - Not experiments or tests
   - Not "just in case" code

---

## 🔍 Supported Code Types

### ✅ Fully Supported

- **Rust projects** - Native language, best integration
- **Python scripts** - Via Python runtime
- **Go modules** - Via Go runtime
- **Configuration files** - YAML, TOML, JSON
- **Documentation** - Markdown, text files

### ⚠️ Partial Support (Needs Adaptation)

- **JavaScript/TypeScript** - Need Node runtime setup
- **C/C++** - Need compilation setup
- **Shell scripts** - Platform-specific issues
- **Other languages** - Case-by-case evaluation

### ❌ Not Supported (Yet)

- **Java/JVM** - No runtime currently
- **Docker images** - Not needed (self-contained OS)
- **.NET Framework** - Use .NET Core/Rust instead
- **Binary-only code** - Source code required

---

## 📋 Example Drops

### Example 1: Small Rust Utility

```powershell
# Drop a small Rust utility
Copy-Item -Recurse "C:\code\file-watcher" `
  "D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\repos\file-watcher"

# Notify me
# "I dropped file-watcher in repos/ - it's a small Rust utility for watching file changes"
```

**What I'll do**:
- Review the code (~15 min)
- Extract the watcher logic
- Integrate into NOA's file system module
- Total time: ~2 hours

### Example 2: Legacy Python Service

```powershell
# Drop old Python service
Copy-Item "C:\old-projects\auth-api.zip" `
  "D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\stale\"

# Extract it
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\stale
Expand-Archive auth-api.zip

# Create manifest
@"
{
  "name": "auth-api",
  "source_type": "stale",
  "analysis": {
    "Language": "Python",
    "Description": "Old authentication API from 2019",
    "LastCommit": "2019-08-20"
  },
  "integration_notes": {
    "must_have": ["JWT token generation", "user validation"],
    "skip": ["old database layer", "deprecated endpoints"]
  }
}
"@ | Out-File -Encoding UTF8 auth-api\manifest.json

# Notify me
# "I dropped auth-api in stale/ - need the JWT and validation logic only"
```

**What I'll do**:
- Review Python code (~30 min)
- Port JWT logic to Rust (~2 hours)
- Port validation logic to Rust (~1 hour)
- Write tests (~1 hour)
- Total time: ~5 hours

### Example 3: Large React Application (Complex)

```powershell
# Clone large project
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\repos
git clone https://github.com/company/admin-dashboard.git

# Create manifest
cd admin-dashboard
@"
{
  "name": "admin-dashboard",
  "source_type": "repos",
  "priority": "high",
  "analysis": {
    "Language": "TypeScript/React",
    "Description": "Admin dashboard with user management"
  },
  "integration_notes": {
    "must_have": ["user CRUD UI", "auth flow", "dashboard layout"],
    "nice_to_have": ["analytics charts", "export features"],
    "skip": ["build scripts", "old components"]
  }
}
"@ | Out-File -Encoding UTF8 manifest.json

# Notify me
# "I dropped admin-dashboard in repos/ - it's a React app, I need the UI components adapted for NOA"
```

**What I'll do**:
- Analyze React components (~1 hour)
- Create integration plan (~1 hour)
- Discuss approach with you (15 min)
- Port key components to NOA UI system (~2 days)
- Test and validate (~4 hours)
- Total time: ~3-4 days (phased approach)

---

## 🎯 Integration Priorities

When you drop code, I prioritize like this:

### Priority 1: Critical Infrastructure
- Security modules (auth, encryption)
- Core utilities (logging, error handling)
- Data management (storage, caching)

### Priority 2: Business Logic
- Domain-specific logic
- Workflow engines
- API implementations

### Priority 3: UI Components
- User interfaces
- Admin dashboards
- Visualization

### Priority 4: Developer Tools
- Build scripts
- Testing utilities
- Development helpers

---

## 🚨 Troubleshooting

### "Where should I drop my code?"

**Question**: Is it fresh code (< 6 months old)?
- Yes → `incoming/repos/`
- No → Continue

**Question**: Is it a fork you modified?
- Yes → `incoming/forks/`
- No → Continue

**Question**: Is it old/abandoned code (> 1 year)?
- Yes → `incoming/stale/`
- No → `incoming/repos/` (default)

### "I dropped code but nothing happened"

**Current system is manual!**
1. You need to notify me: "I dropped X in Y/"
2. I then analyze and integrate
3. In the future (Option B), this will be automatic

### "Can I drop multiple projects at once?"

**Yes!**
- One folder per project
- Notify me about all of them
- I'll prioritize based on importance

### "What if my code has secrets/API keys?"

**Remove them first:**
```powershell
# Before dropping, clean sensitive files
cd C:\your-code\project
Remove-Item -Recurse .env,.env.*,secrets/,*.key,*.pem
```

Then drop the cleaned code.

### "What if integration takes too long?"

**Phased approach:**
1. We start with must-have features
2. Get those working first
3. Add nice-to-have features later
4. Skip the rest

This way you get value quickly, not all-or-nothing.

---

## 📚 Next Steps After Dropping

### 1. I Analyze Your Code
You'll get a report like:
```
📊 Analysis Complete: auth-service

Files: 45
Size: 2.3 MB
Language: Rust
Confidence: 92%

Key Components Identified:
✅ JWT token generation (core/jwt.rs)
✅ User validation (core/validate.rs)
✅ Password hashing (core/crypto.rs)
⚠️ Database layer (uses external Postgres)
❌ Old API endpoints (deprecated)

Integration Plan:
Phase 1: Extract JWT and validation (2 hours)
Phase 2: Adapt crypto module (1 hour)
Phase 3: Replace DB layer with NOA storage (3 hours)

Total Estimate: ~6 hours
```

### 2. We Discuss the Plan
- Review what I found
- Adjust priorities if needed
- Confirm approach

### 3. I Integrate the Code
- Extract key components
- Adapt to NOA conventions
- Test thoroughly
- Update documentation

### 4. You Test It
- Verify it works as expected
- Report any issues
- Request adjustments if needed

---

## 🎉 Success Stories

### Agent Registry (Completed)
**Dropped**: 928-agent CSV directory  
**Integration**: Agent registry module in `noa_agents`  
**Time**: 2 hours  
**Result**: ✅ `cargo run --example load_agent_registry` works!

### AgentAsKit (In Planning)
**Dropped**: 46 MB, 2,299 files Rust project  
**Integration**: Phased approach (see AGENTASKIT_INTEGRATION_PLAN.md)  
**Phase 1**: Core types (~4 hours)  
**Status**: Plan ready, awaiting your decision

---

## 💬 How to Notify Me

After dropping code, just say:

**Simple**:
```
"I dropped auth-service in stale/"
```

**Better**:
```
"I dropped auth-service in stale/ - it's an old Rust authentication 
service from 2019. I need the JWT generation and user validation logic."
```

**Best**:
```
"I dropped auth-service in stale/ with a manifest.json. 
It's Rust code from 2019. Priority is JWT + validation, 
skip the old database layer. Let me know the integration plan."
```

---

## ✅ Checklist Before Dropping

- [ ] Code is in the right folder (repos/forks/mirrors/stale)
- [ ] Folder has a clear, descriptive name
- [ ] Sensitive data removed (keys, passwords, secrets)
- [ ] README or documentation included (if available)
- [ ] Manifest.json created (optional but helpful)
- [ ] Ready to notify me about the drop

---

**Ready to drop your first code?** 🚀

**Just follow these steps:**
1. Choose the right folder
2. Copy your code there
3. (Optional) Create manifest.json
4. Notify me: "I dropped {name} in {folder}/"
5. Wait for analysis and integration

**Questions?** Ask anytime!
