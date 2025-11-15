# CRC User Workflow Guide

## Quick Start - Dropping Code into NOA ARK OS

This guide shows you how to drop external code into the CRC (Continuous ReCode) system for AI-assisted adaptation and deployment.

## 📂 Where to Drop Your Code

### Option 1: Direct File Placement (Recommended)

Simply copy/paste your code into the appropriate folder:

```
crc/drop-in/incoming/
├── repos/      ← Drop fresh, actively maintained repositories
├── forks/      ← Drop forked projects with modifications
├── mirrors/    ← Drop mirror/clone repositories
└── stale/      ← Drop old, unmaintained, or abandoned code
```

**Example (Windows):**
```powershell
# Copy your code archive
Copy-Item "C:\projects\old-project.zip" "D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\stale\"
```

**Example (Linux/Mac):**
```bash
# Copy your code archive
cp ~/projects/old-project.zip ~/noa_ark_os/crc/drop-in/incoming/stale/
```

### Option 2: Chat Commands (CLI)

Use the chat interface to drop code:

```bash
# Drop a fresh repository
/drop repo github.com/user/project

# Drop a forked project
/drop fork github.com/original/project my-fork-url

# Drop a mirror repository
/drop mirror github.com/mirror/project

# Drop stale/abandoned code
/drop stale path/to/old-code.zip --name "legacy-app" --last-commit "2020-06-15"
```

---

## 🔄 Complete Workflow

### 1. Choose Source Type

Select the folder based on your code type:

| Type | Folder | Use When | Priority |
|------|--------|----------|----------|
| **repos/** | Fresh Repository | Active, maintained repos (< 6 months) | High |
| **forks/** | Forked Project | Modified fork of existing project | Medium |
| **mirrors/** | Mirror | Clone from different source | Medium |
| **stale/** | Stale/Abandoned | Unmaintained code (> 1 year) | Normal |

### 2. Drop Your Code

**Supported Formats:**
- `.zip` - Recommended
- `.tar.gz` - Compressed archive
- `.tar.zst` - High compression
- `.git` - Git repository
- Directory - Uncompressed folder

**Example:**
```bash
# Windows - Drop stale code
Copy-Item "C:\old-code\legacy-app.zip" "D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\stale\"

# Linux/Mac - Drop stale code
cp ~/old-code/legacy-app.zip ~/noa_ark_os/crc/drop-in/incoming/stale/
```

### 3. Automatic Detection

The **File Watcher** automatically detects your drop within **1-2 seconds**:

```
[CRC] File detected: legacy-app.zip
[CRC] Detected source type: Stale
[CRC] ✓ Drop registered: drop-abc-123
```

### 4. Processing Stages (Parallel)

Your code goes through 3 parallel stages:

```
Stage 1: AI Analysis     (~2 min)  → Confidence score
Stage 2: Code Adaptation (~3 min)  → NOA ARK OS adaptation
Stage 3: Validation      (~2 min)  → Pre-sandbox checks
─────────────────────────────────────────────────────
Total Time: ~7 minutes
```

**You'll see progress:**
```
✓ Drop registered: drop-abc-123
  Location: crc/drop-in/incoming/stale/legacy-app.zip
  Status: Processing...
  
  → AI Analysis: ~2 minutes
  → Adaptation: ~3 minutes
  → Validation: ~2 minutes
  → ETA: ~7 minutes
```

### 5. Sandbox Assignment

Based on AI confidence and source type, your code is assigned to a sandbox:

| Confidence | Source Type | Sandbox | Purpose |
|-----------|-------------|---------|---------|
| ≥95% | repos | **Model A** | Feature Development |
| 85-95% | forks | **Model B** | Bug Fixes |
| <85% | stale | **Model C** | Experimental (extra validation) |

**Your code path:**
```
incoming/stale/legacy-app.zip
  ↓ (Analysis + Adaptation + Validation)
processing/validation/legacy-app/
  ↓ (Assigned based on confidence)
ready/model-c-queue/legacy-app/
  ↓ (Moved to sandbox)
sandboxes/model-c/active/legacy-app/
```

### 6. Sandbox Validation

Your code is tested in an isolated sandbox:

```
[Model C] Running tests...
  ✓ Syntax validation: PASSED
  ✓ Dependency resolution: PASSED
  ✓ Security scan: PASSED
  ✓ Integration tests: PASSED (87% coverage)
  
Confidence: 87% → Ready for merge
```

### 7. Merge to Integration (Model D)

If validation passes, your code merges with others:

```
Model A (Feature) ┐
Model B (Bug Fix)  ├→ Model D (Integration)
Model C (Stale)    ┘
```

### 8. CI/CD Deployment

**Auto-Deploy Decision:**
- **≥95% confidence** → Auto-deploy to staging
- **<95% confidence** → Human review required

```
[CI/CD] Deployment pipeline triggered
  ✓ Staging: Blue-Green deployment (PASSED)
  ✓ Production: Canary deployment (5% → 100%)
  ✓ Health checks: All PASSED
  
🎉 Deployed to production!
```

---

## 📊 Checking Status

### CLI Commands

```bash
# Check status of your drop
/drop status drop-abc-123

# List all your drops
/drop list

# List by status
/drop list --status processing

# List by type
/drop list --source-type stale
```

### Example Output

```
📊 Drop Status: drop-abc-123

  Status: Processing
  Stage: Adaptation (2/3)
  Confidence: 87%
  Started: 5 minutes ago
  ETA: 2 minutes

  Progress:
  ✓ Analysis completed (87% confidence)
  ⏳ Adaptation in progress...
  ⏸  Validation pending
  
  Next: Will be assigned to Model C (Experimental)
```

---

## 🎯 Real-World Examples

### Example 1: Fresh Repository

```bash
# You have an active GitHub repo
/drop repo github.com/user/awesome-project

# OR copy directly
cp ~/awesome-project.zip ~/noa_ark_os/crc/drop-in/incoming/repos/
```

**Result:**
- Auto-detected as: repos (High priority)
- AI Confidence: 96%
- Assigned to: Model A (Feature)
- Auto-deployed: Yes (≥95%)
- Time to production: ~15 minutes

### Example 2: Forked Project

```bash
# You forked a project and made changes
/drop fork github.com/original/proj github.com/yourfork/proj

# OR copy directly
cp ~/my-fork.zip ~/noa_ark_os/crc/drop-in/incoming/forks/
```

**Result:**
- Auto-detected as: fork (Medium priority)
- AI Confidence: 91%
- Assigned to: Model B (Bug Fix)
- Auto-deployed: No (< 95%, needs review)
- Time to review: ~10 minutes

### Example 3: Stale Abandoned Code (Your Use Case!)

```bash
# You have old, unmaintained code
/drop stale ~/old-code/legacy-app.zip --name "legacy-crm" --last-commit "2018-03-15"

# OR copy directly (simplest way)
cp ~/old-code/legacy-app.zip ~/noa_ark_os/crc/drop-in/incoming/stale/
```

**Result:**
- Auto-detected as: stale (Normal priority)
- AI Confidence: 87%
- Assigned to: Model C (Experimental)
- Extra validation: Yes (dependency updates, compatibility checks)
- Auto-deployed: No (< 95%, needs review)
- Time to review: ~15 minutes

**What happens to stale code:**
1. AI analyzes outdated dependencies
2. AI updates syntax to modern standards
3. AI adds compatibility layers
4. Extra security scanning
5. More thorough testing
6. Human review recommended before production

---

## ⚙️ Configuration

### Adjust Confidence Thresholds

Edit `crc/config/sources.yaml`:

```yaml
stale:
  auto_approve_threshold: 0.80  # Lower for stale code
  default_sandbox: model-c
  validation:
    dependency_update: true      # Auto-update dependencies
    compatibility_check: true    # Extra checks
```

### Worker Configuration

```yaml
workers:
  validation:
    count: 4  # Increased from 2 for faster processing
```

---

## 🚨 Troubleshooting

### Drop Not Detected

**Problem:** File copied but not detected

**Solutions:**
1. Check file format (must be .zip, .tar.gz, .tar.zst, or .git)
2. Wait 2 seconds (file watcher debounce)
3. Check file name (avoid `.tmp`, `.partial`, `.DS_Store`)
4. Restart file watcher: `/drop restart`

### Low Confidence Score

**Problem:** AI confidence < 80%

**Solutions:**
1. Check if dependencies are too outdated
2. Add README.md with context
3. Include original documentation
4. Manually specify language/framework
5. Contact support for manual review

### Processing Stuck

**Problem:** Drop stuck in processing

**Solutions:**
1. Check status: `/drop status drop-id`
2. View logs: `crc/temp/logs/drop-id.log`
3. Cancel and retry: `/drop cancel drop-id` then `/drop retry drop-id`

---

## 📈 Performance

### Parallel Processing

- **4 Analysis workers** → 4 concurrent drops
- **4 Adaptation workers** → 4 concurrent adaptations
- **4 Validation workers** → 4 concurrent validations

**Throughput:**
- Analysis: 120 drops/hour
- Adaptation: 80 drops/hour
- Validation: 120 drops/hour
- **Bottleneck:** Adaptation (80/hour)

### Multiple Users

Multiple users can drop code simultaneously:

```
Time  User A          User B          User C
────  ─────────────   ─────────────   ─────────────
10:00 Drops repo-1    
10:01                 Drops fork-1    
10:02                                 Drops stale-1
10:03 All processing in parallel (4 workers each stage)
```

---

## 🎓 Best Practices

### DO ✅

- **Name your files clearly:** `legacy-crm-v2.zip` not `code.zip`
- **Include README.md:** Helps AI understand context
- **Use correct folder:** Match your code type to folder
- **Check status regularly:** Monitor progress
- **Review before production:** Even with high confidence

### DON'T ❌

- **Drop massive files:** Keep under 500 MB
- **Drop without context:** Include documentation
- **Ignore validation errors:** Address issues before retry
- **Skip human review:** Always review < 95% confidence
- **Drop sensitive data:** Remove secrets first

---

## 📞 Support

### Get Help

```bash
# Show help
/drop help

# Show examples
/drop examples

# Check system status
/drop system-status
```

### Contact

- **Issues:** Open GitHub issue
- **Questions:** Check docs/FAQ.md
- **Security:** security@noa-ark-os.com

---

## 🎉 Success! What's Next?

After your drop is deployed:

1. **Monitor production:** Check dashboards
2. **Review metrics:** Performance and errors
3. **Optimize:** Based on real-world usage
4. **Drop more code:** Repeat the process!

**Welcome to NOA ARK OS!** 🚀
