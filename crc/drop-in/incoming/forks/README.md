# Fork Repository Drop-in Directory

**Purpose**: External fork repositories for processing through CRC  
**Location**: `crc/drop-in/incoming/forks/`  

---

## 📂 How to Add a Fork

### Method 1: Manual Drop (Recommended for Testing)

1. **Create directory** with fork name:
   ```powershell
   cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks
   mkdir my-awesome-fork
   ```

2. **Copy source files** into the directory:
   ```powershell
   Copy-Item -Path "C:\path\to\external\fork\*" -Destination ".\my-awesome-fork\" -Recurse
   ```

3. **Run detector** to process:
   ```powershell
   cd D:\dev\workspaces\noa_ark_os\crc
   .\detect-forks.ps1 -Mode process -ForkName "my-awesome-fork"
   ```

### Method 2: Git Clone

1. **Clone repository** into forks directory:
   ```powershell
   cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks
   git clone https://github.com/user/repo.git my-fork-name
   ```

2. **Run detector**:
   ```powershell
   cd D:\dev\workspaces\noa_ark_os\crc
   .\detect-forks.ps1 -Mode process -ForkName "my-fork-name"
   ```

### Method 3: Automated Watch

1. **Start fork monitor**:
   ```powershell
   cd D:\dev\workspaces\noa_ark_os\crc
   .\detect-forks.ps1 -Mode watch
   ```

2. **Drop forks** into directory - they'll be detected automatically

---

## 📋 What Happens After Drop-in

### Automatic Processing Flow

1. **Detection**
   - Fork detector finds new directory
   - Initializes metadata
   - Analyzes file structure

2. **Branch Creation**
   - Creates branch: `fork/{fork-name}`
   - Isolates changes
   - Writes `branch.txt`

3. **CRC Analysis** (Future)
   - AI analyzes code
   - Generates adaptation plan
   - Scores confidence level
   - Identifies useful components

4. **Integration** (Future)
   - Auto-adapt to workspace conventions
   - Run tests in sandbox
   - Validate integration
   - Merge if confident

5. **Archival**
   - Compress original fork
   - Save to `crc/archive/forks/`
   - Create metadata
   - Clean up original directory

6. **Cleanup**
   - Delete original folder
   - Keep compressed archive
   - Maintain cross-reference

---

## 📊 Fork Status

### Check Fork Status

```powershell
# List all forks
.\detect-forks.ps1 -Mode list

# Output shows:
# - Fork name
# - Current status
# - Language detected
# - File count
# - Line count
# - Processing dates
# - Branch name
```

### Status Values

- **`pending`**: Waiting to be processed
- **`processing`**: Currently being processed
- **`needs_review`**: Requires manual review
- **`approved`**: Auto-approved for integration
- **`rejected`**: Not suitable for integration
- **`archived`**: Compressed and archived
- **`error`**: Processing failed

---

## 📁 Directory Structure After Processing

```
my-awesome-fork/
├── src/                    # Source files (original)
├── tests/                  # Test files (original)
├── README.md               # Original README
├── Cargo.toml              # Original manifest
├── metadata.json           # Generated metadata ✨
└── branch.txt              # Branch name ✨
```

---

## 🔍 Metadata File

After detection, a `metadata.json` file is automatically created:

```json
{
  "fork_id": "uuid-v4",
  "repo_name": "my-awesome-fork",
  "original_url": "https://github.com/user/repo",
  "fork_source": "manual|github|gitlab",
  "received_date": "2024-10-08T10:30:00Z",
  "processed_date": null,
  "status": "pending",
  "language": "rust|python|go|csharp|javascript",
  "metrics": {
    "lines_of_code": 5420,
    "files": 42,
    "file_types": {
      ".rs": 35,
      ".toml": 3,
      ".md": 4
    }
  },
  "integration": {
    "branch": "fork/my-awesome-fork",
    "merged": false
  }
}
```

---

## 🎯 Commands Reference

### Processing

```powershell
# Process specific fork
.\detect-forks.ps1 -Mode process -ForkName "my-fork"

# Watch for new forks (continuous)
.\detect-forks.ps1 -Mode watch

# Watch with custom interval
.\detect-forks.ps1 -Mode watch -IntervalSeconds 30

# List all forks and their status
.\detect-forks.ps1 -Mode list
```

### Manual Operations

```powershell
# View metadata
Get-Content ".\forks\my-fork\metadata.json" | ConvertFrom-Json | ConvertTo-Json -Depth 10

# Check branch
Get-Content ".\forks\my-fork\branch.txt"

# Delete fork (manual cleanup)
Remove-Item ".\forks\my-fork" -Recurse -Force
```

---

## 🔒 Security & Best Practices

### Before Dropping a Fork

1. **Verify source**: Only drop trusted repositories
2. **Scan for malware**: Use antivirus/security tools
3. **Check license**: Ensure compatible license (MIT, Apache, BSD, etc.)
4. **Remove secrets**: No API keys, passwords, or tokens
5. **Review size**: Large repos (>1GB) may need special handling

### What to Include

✅ **Include**:
- Source code files
- Documentation (README, docs/)
- Tests
- Configuration files
- License file
- Git history (optional, via `.git/`)

❌ **Exclude**:
- `target/` or `build/` directories
- `node_modules/`
- Large binary files (>100MB)
- Secrets or credentials
- Personal data
- Proprietary code

---

## 📚 Examples

### Example 1: Simple Rust Library

```powershell
# Clone external Rust library
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks
git clone https://github.com/external/rust-parser.git rust-parser

# Process it
cd ..\..
.\detect-forks.ps1 -Mode process -ForkName "rust-parser"

# Check status
.\detect-forks.ps1 -Mode list
```

### Example 2: Manual Code Drop

```powershell
# Create directory
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks
mkdir custom-validator

# Copy files
Copy-Item -Path "C:\downloads\validator-code\*" -Destination ".\custom-validator\" -Recurse

# Process
cd ..\..
.\detect-forks.ps1 -Mode process -ForkName "custom-validator"
```

### Example 3: Automated Monitoring

```powershell
# Start monitor in background
cd D:\dev\workspaces\noa_ark_os\crc
Start-Process powershell -ArgumentList "-NoExit", "-File", ".\detect-forks.ps1", "-Mode", "watch"

# Now drop forks as needed - they'll be detected automatically
```

---

## 🚨 Troubleshooting

### Fork Not Detected

**Problem**: Fork added but not being processed

**Solutions**:
1. Check directory name (no special characters)
2. Verify files exist in directory
3. Run manual process: `.\detect-forks.ps1 -Mode process -ForkName "fork-name"`
4. Check metadata.json was created
5. Verify status with: `.\detect-forks.ps1 -Mode list`

### Processing Failed

**Problem**: Fork status shows "error"

**Solutions**:
1. Check metadata.json for error details
2. Verify Git is installed and accessible
3. Ensure workspace is in clean state
4. Try creating branch manually: `git checkout -b fork/fork-name`
5. Re-run processing

### Branch Already Exists

**Problem**: Branch already exists for fork

**Solutions**:
1. Delete old branch: `git branch -D fork/fork-name`
2. Or use existing branch: `git checkout fork/fork-name`
3. Re-run processing

---

## 📖 Additional Documentation

- **Fork Processing System**: `../FORK_PROCESSING_SYSTEM.md`
- **CRC Overview**: `../README.md`
- **Workspace Memory**: `../../WORKSPACE_MEMORY.md`
- **Build Status**: `../../BUILD_SUCCESS_STATUS.md`

---

## 🎯 Quick Start

**Simplest Way to Test:**

```powershell
# 1. Navigate to forks directory
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks

# 2. Create test fork
mkdir test-fork
echo "println!(\"Hello from test fork\");" > test-fork\main.rs

# 3. Process it
cd ..\..\..\..
.\crc\detect-forks.ps1 -Mode process -ForkName "test-fork"

# 4. Check result
.\crc\detect-forks.ps1 -Mode list
```

---

**Status**: Ready for fork drop-ins!  
**Detector**: `crc/detect-forks.ps1`  
**Next**: Drop forks and run detector  
