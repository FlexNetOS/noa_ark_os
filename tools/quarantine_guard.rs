use anyhow::{anyhow, Context, Result};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const QUARANTINE_MARKER: &str = "archive/quarantine/";
const ALLOW_MARKER: &str = "QUARANTINE_GUARD_ALLOW";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub path: PathBuf,
    pub line: usize,
    pub content: String,
}

pub fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let files = gather_paths(&args)?;
    if files.is_empty() {
        return Ok(());
    }

    let violations = check_files(&files)?;
    if violations.is_empty() {
        return Ok(());
    }

    eprintln!("❌ Quarantine guard detected references to quarantined bundles:");
    for v in &violations {
        eprintln!("  {}:{} -> {}", v.path.display(), v.line, v.content.trim());
    }
    Err(anyhow!(
        "{} quarantined reference(s) detected",
        violations.len()
    ))
}

fn gather_paths(args: &[String]) -> Result<Vec<PathBuf>> {
    if !args.is_empty() {
        return Ok(args.iter().map(PathBuf::from).collect());
    }

    let mut files = BTreeSet::new();
    for candidate in [
        &["diff", "--name-only", "--cached"][..],
        &["diff", "--name-only", "HEAD"][..],
        &["ls-files"][..],
    ] {
        let output = git_output(candidate)?;
        for line in output.lines().map(str::trim).filter(|l| !l.is_empty()) {
            files.insert(PathBuf::from(line));
        }
        if !files.is_empty() {
            break;
        }
    }

    Ok(files.into_iter().collect())
}

fn git_output(args: &[&str]) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    let output = cmd.output().with_context(|| format!("failed to run git {:?}", args))?;
    if !output.status.success() {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn check_files(files: &[PathBuf]) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for path in files {
        if should_skip(path) {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for (idx, line) in content.lines().enumerate() {
            if !line.contains(QUARANTINE_MARKER) {
                continue;
            }
            if line.contains(ALLOW_MARKER) {
                continue;
            }
            violations.push(Violation {
                path: path.clone(),
                line: idx + 1,
                content: line.trim().to_string(),
            });
        }
    }
    Ok(violations)
}

fn should_skip(path: &Path) -> bool {
    if !path.exists() {
        return true;
    }
    if path.is_dir() {
        return true;
    }
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if !is_code_extension(ext) {
            return true;
        }
    } else {
        // Files without an extension are generally build scripts; scan them anyway.
    }
    let path_str = path.to_string_lossy();
    if path_str.contains(QUARANTINE_MARKER) {
        return true;
    }
    false
}

fn is_code_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "rs"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "py"
            | "sh"
            | "bash"
            | "zsh"
            | "ps1"
            | "psm1"
            | "go"
            | "java"
            | "kt"
            | "scala"
            | "swift"
            | "c"
            | "h"
            | "hpp"
            | "hh"
            | "cpp"
            | "cxx"
            | "cs"
            | "rb"
            | "php"
            | "lua"
            | "pl"
            | "sql"
            | "toml"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn detects_quarantine_reference_in_code() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("src.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(
            file,
            "use crate::legacy::component; // archive/quarantine/legacy@1234"
        )
        .unwrap();

        let violations = check_files(&[file_path]).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].content.contains(QUARANTINE_MARKER));
    }

    #[test]
    fn ignores_marked_allowances() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(
            file,
            "// archive/quarantine/foo@deadbeef QUARANTINE_GUARD_ALLOW"
        )
        .unwrap();

        let violations = check_files(&[file_path]).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn skips_non_code_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("README.md");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "archive/quarantine/foo@deadbeef").unwrap();

        let violations = check_files(&[file_path]).unwrap();
        assert!(violations.is_empty());
    }
}
