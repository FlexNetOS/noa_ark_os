# CRC Quick Reference Card

## 🚀 Quick Start - Drop Code in 30 Seconds

### Step 1: Choose Your Folder

```
crc/drop-in/incoming/
├── repos/    ← Active repos (< 6 months old)
├── forks/    ← Your modified forks
├── mirrors/  ← Cloned/mirror repos
└── stale/    ← Old/unmaintained code ⭐ YOU ARE HERE
```

### Step 2: Copy Your File

**Windows:**
```powershell
Copy-Item "C:\path\to\your-code.zip" "D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\stale\"
```

**Linux/Mac:**
```bash
cp ~/path/to/your-code.zip ~/noa_ark_os/crc/drop-in/incoming/stale/
```

### Step 3: Wait ~7 Minutes

```
✓ Detected (2s)
↓
✓ Analysis (2min)
↓
✓ Adaptation (3min)
↓
✓ Validation (2min)
↓
✓ Ready for deployment!
```

---

## 📋 Chat Commands

```bash
# Drop code
/drop stale path/to/code.zip

# Check status
/drop status drop-abc-123

# List drops
/drop list

# Cancel drop
/drop cancel drop-abc-123
```

---

## 📂 Where Files Go

```
Your file:
  incoming/stale/your-code.zip

Processing:
  processing/analysis/your-code/
  processing/adaptation/your-code/
  processing/validation/your-code/

Queue:
  ready/model-c-queue/your-code/

Sandbox:
  sandboxes/model-c/active/your-code/

Integration:
  sandboxes/model-d/merged/your-code/

Deployed! 🎉
```

---

## 🎯 Source Type Decision Tree

```
Is your code...
├─ Active & maintained (< 6 months)? → repos/
├─ A fork you modified? → forks/
├─ A mirror/clone? → mirrors/
└─ Old/unmaintained (> 1 year)? → stale/ ⭐
```

---

## 📊 What Confidence Score Means

| Score | Meaning | Auto-Deploy? | Sandbox |
|-------|---------|--------------|---------|
| ≥95% | Excellent | ✅ Yes | Model A/B |
| 85-94% | Good | ❌ Needs review | Model B/C |
| <85% | Needs work | ❌ Manual review | Model C |

---

## ⚡ Performance

- **Detection:** < 2 seconds
- **Processing:** ~7 minutes
- **Deployment:** ~5 minutes
- **Total:** ~12 minutes

**Parallel Processing:**
- 4 Analysis workers
- 4 Adaptation workers
- 4 Validation workers
- **Can process 4 drops simultaneously**

---

## 🔍 Check Status

```bash
/drop status drop-abc-123
```

**Output:**
```
📊 Drop Status: drop-abc-123

  Status: Processing
  Stage: Adaptation (2/3)
  Confidence: 87%
  Started: 5 minutes ago
  ETA: 2 minutes

  Progress:
  ✓ Analysis completed
  ⏳ Adaptation in progress
  ⏸  Validation pending
```

---

## 📦 Supported Formats

✅ `.zip` (recommended)
✅ `.tar.gz`
✅ `.tar.zst`
✅ `.tar`
✅ `.git` (repository)
✅ Directory (uncompressed)

❌ Individual files (must be archived)
❌ Executables without source
❌ Files > 500 MB

---

## 🚨 Troubleshooting

### File Not Detected?

1. Wait 2 seconds (debounce)
2. Check file format
3. Avoid temp files (.tmp, .partial)
4. Restart watcher: `/drop restart`

### Low Confidence (<80%)?

1. Include README.md
2. Add documentation
3. Update dependencies manually
4. Contact support

### Stuck in Processing?

1. Check status: `/drop status drop-id`
2. View logs: `crc/temp/logs/drop-id.log`
3. Retry: `/drop retry drop-id`

---

## ✅ Best Practices

**DO:**
- ✅ Name files clearly
- ✅ Include README.md
- ✅ Use correct folder
- ✅ Remove secrets first
- ✅ Review before production

**DON'T:**
- ❌ Drop without context
- ❌ Skip validation
- ❌ Ignore errors
- ❌ Drop sensitive data
- ❌ Exceed 500 MB

---

## 🎯 Stale Code Special Notes

**What happens to stale code:**

1. ✅ AI updates dependencies
2. ✅ AI modernizes syntax
3. ✅ AI adds compatibility layers
4. ✅ Extra security scanning
5. ✅ More thorough testing
6. ⚠️  Manual review recommended

**Default Settings:**
- Priority: Normal
- Sandbox: Model C (Experimental)
- Auto-approve threshold: 80%
- Extra validation: Enabled

---

## 📞 Get Help

```bash
/drop help          # Show help
/drop examples      # Show examples
/drop system-status # Check system status
```

**Support:**
- GitHub Issues
- docs/FAQ.md
- security@noa-ark-os.com

---

## 🎉 Example: Complete Flow

```bash
# 1. Copy file (Windows)
Copy-Item "C:\old\legacy-app.zip" "D:\noa_ark_os\crc\drop-in\incoming\stale\"

# 2. System auto-detects
[CRC] ✓ Drop registered: drop-abc-123

# 3. Check status
/drop status drop-abc-123
# Status: Processing (ETA: 5 min)

# 4. Wait for completion
[CRC] ✓ Ready for deployment
[CRC] Confidence: 87% → Needs review

# 5. Review and approve
/drop approve drop-abc-123

# 6. Deploy!
[CI/CD] ✓ Deployed to production 🎉
```

---

## 📈 Quick Stats

| Metric | Value |
|--------|-------|
| Detection Time | < 2s |
| Processing Time | ~7 min |
| Concurrent Drops | 4 |
| Throughput | 80/hour |
| Max File Size | 500 MB |
| Max Queue | 100 drops |

---

**Print this card and keep it handy!** 📋
**Or bookmark: `crc/QUICK_REFERENCE.md`** 🔖
