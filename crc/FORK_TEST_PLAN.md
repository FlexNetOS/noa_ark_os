# Fork System Test Plan

**Purpose**: Validate fork detection and processing system  
**Status**: Ready to execute  

---

## 🎯 Test Objectives

1. Verify fork detection works
2. Confirm metadata generation
3. Test branch creation
4. Validate status tracking
5. Ensure documentation accuracy

---

## 🧪 Test Cases

### Test 1: Simple Rust File

**Objective**: Process single Rust file

**Steps**:
```powershell
# 1. Create fork
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks
mkdir test-simple-rust
echo 'fn main() { println!("Hello from test fork!"); }' > test-simple-rust\main.rs

# 2. Process
cd ..\..\..
.\detect-forks.ps1 -Mode process -ForkName "test-simple-rust"

# 3. Verify metadata created
cat .\drop-in\incoming\forks\test-simple-rust\metadata.json

# 4. Check branch created
cat .\drop-in\incoming\forks\test-simple-rust\branch.txt
git branch | Select-String "fork/test-simple-rust"

# 5. Check status
.\detect-forks.ps1 -Mode list
```

**Expected Results**:
- ✅ `metadata.json` created
- ✅ `branch.txt` created
- ✅ Branch `fork/test-simple-rust` exists
- ✅ Status shows "needs_review"
- ✅ Language detected as "rust"
- ✅ File count: 1
- ✅ Line count: 1

---

### Test 2: Simple Cargo Project

**Objective**: Process small Cargo project

**Steps**:
```powershell
# 1. Create fork with Cargo.toml
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks
mkdir test-cargo-project
cd test-cargo-project

# Create Cargo.toml
@"
[package]
name = "test-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
"@ | Out-File -FilePath Cargo.toml -Encoding UTF8

# Create src directory
mkdir src

# Create lib.rs
@"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"@ | Out-File -FilePath src\lib.rs -Encoding UTF8

# 2. Process
cd ..\..\..\..
.\detect-forks.ps1 -Mode process -ForkName "test-cargo-project"

# 3. Verify
.\detect-forks.ps1 -Mode list
```

**Expected Results**:
- ✅ Metadata detects multiple files
- ✅ Language: "rust"
- ✅ Files: 2 (Cargo.toml, lib.rs)
- ✅ Lines counted correctly
- ✅ Branch created
- ✅ Status: "needs_review"

---

### Test 3: Multiple Files

**Objective**: Process multi-file project

**Steps**:
```powershell
# 1. Create complex fork
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks
mkdir test-multi-file
cd test-multi-file

mkdir src
mkdir tests

echo 'pub fn helper() -> String { String::from("helper") }' > src\helper.rs
echo 'pub mod helper;' > src\lib.rs
echo '#[test] fn test() { assert!(true); }' > tests\integration.rs
echo '# Test Project' > README.md

# 2. Process
cd ..\..\..\..
.\detect-forks.ps1 -Mode process -ForkName "test-multi-file"

# 3. Verify
.\detect-forks.ps1 -Mode list
Get-Content .\drop-in\incoming\forks\test-multi-file\metadata.json | ConvertFrom-Json | ConvertTo-Json -Depth 10
```

**Expected Results**:
- ✅ All files counted
- ✅ Multiple file types detected (.rs, .md)
- ✅ Directory structure preserved
- ✅ Metadata accurate

---

### Test 4: List Forks

**Objective**: Verify listing functionality

**Steps**:
```powershell
cd D:\dev\workspaces\noa_ark_os\crc
.\detect-forks.ps1 -Mode list
```

**Expected Results**:
- ✅ All test forks listed
- ✅ Status shown for each
- ✅ Metrics displayed
- ✅ Dates shown

---

### Test 5: Watch Mode (Manual)

**Objective**: Test continuous monitoring

**Steps**:
```powershell
# 1. Start watch mode in separate terminal
cd D:\dev\workspaces\noa_ark_os\crc
.\detect-forks.ps1 -Mode watch -IntervalSeconds 10

# 2. In another terminal, create new fork
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks
mkdir test-auto-detect
echo 'fn main() {}' > test-auto-detect\main.rs

# 3. Watch first terminal for detection message
# Should see: "New fork detected: test-auto-detect"

# 4. Stop watch mode (Ctrl+C)
```

**Expected Results**:
- ✅ Fork detected automatically
- ✅ Processing triggered
- ✅ Status updated
- ✅ No manual intervention needed

---

## 🧹 Cleanup After Tests

```powershell
cd D:\dev\workspaces\noa_ark_os

# Remove test forks
Remove-Item -Recurse -Force crc\drop-in\incoming\forks\test-simple-rust -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force crc\drop-in\incoming\forks\test-cargo-project -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force crc\drop-in\incoming\forks\test-multi-file -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force crc\drop-in\incoming\forks\test-auto-detect -ErrorAction SilentlyContinue

# Delete test branches
git branch -D fork/test-simple-rust 2>$null
git branch -D fork/test-cargo-project 2>$null
git branch -D fork/test-multi-file 2>$null
git branch -D fork/test-auto-detect 2>$null

# Verify cleanup
.\crc\detect-forks.ps1 -Mode list
```

---

## ✅ Success Criteria

### All Tests Must
- ✅ Complete without errors
- ✅ Generate valid metadata
- ✅ Create branches successfully
- ✅ Track status correctly
- ✅ Display accurate information

### Quality Checks
- ✅ File counting accurate
- ✅ Language detection correct
- ✅ Line counting works
- ✅ JSON valid and parseable
- ✅ Branches properly formatted

---

## 📊 Test Results Template

```markdown
## Test Execution Results

**Date**: [Date]
**Tester**: [Name]
**Environment**: Windows PowerShell

### Test 1: Simple Rust File
- Status: ✅ Pass / ❌ Fail
- Notes: 

### Test 2: Simple Cargo Project
- Status: ✅ Pass / ❌ Fail
- Notes:

### Test 3: Multiple Files
- Status: ✅ Pass / ❌ Fail
- Notes:

### Test 4: List Forks
- Status: ✅ Pass / ❌ Fail
- Notes:

### Test 5: Watch Mode
- Status: ✅ Pass / ❌ Fail
- Notes:

### Overall Result
- Total Tests: 5
- Passed: [X]
- Failed: [X]
- Success Rate: [X]%

### Issues Found
1. [Issue description]
2. [Issue description]

### Recommendations
1. [Recommendation]
2. [Recommendation]
```

---

## 🚀 Quick Test (1 Minute)

**Fastest way to validate system**:

```powershell
# Single command test
cd D:\dev\workspaces\noa_ark_os\crc
mkdir -Force drop-in\incoming\forks\quick-test; echo 'fn main() {}' > drop-in\incoming\forks\quick-test\main.rs; .\detect-forks.ps1 -Mode process -ForkName quick-test; .\detect-forks.ps1 -Mode list; Remove-Item -Recurse -Force drop-in\incoming\forks\quick-test
```

**Expected**: Fork processed, metadata generated, branch created, then cleaned up

---

**Status**: Test plan ready for execution  
**Next**: Run tests and validate all functionality  
**Time Required**: ~10 minutes for full test suite  
