use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn cli_bin() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../target/debug/agent_tools");
    path
}

#[test]
fn test_cli_read() {
    let bin = cli_bin();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let output = Command::new(&bin)
        .args(&["read", manifest.to_str().unwrap()])
        .output()
        .expect("failed to execute read");
    assert!(output.status.success(), "read command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("noa-tools-agent"),
        "expected package name in Cargo.toml"
    );
}

#[test]
fn test_cli_grep() {
    let bin = cli_bin();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(&bin)
        .args(&["grep", "noa-tools-agent", "--path", root.to_str().unwrap()])
        .output()
        .expect("failed to execute grep");
    assert!(output.status.success(), "grep command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Cargo.toml"),
        "expected match in Cargo.toml"
    );
}

#[test]
fn test_cli_glob() {
    let bin = cli_bin();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(&bin)
        .args(&["glob", "*.toml", "--path", root.to_str().unwrap()])
        .output()
        .expect("failed to execute glob");
    assert!(output.status.success(), "glob command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Cargo.toml"), "expected Cargo.toml match");
}

#[test]
fn test_cli_semantic() {
    let bin = cli_bin();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(&bin)
        .args(&["semantic", "offline", "--path", root.to_str().unwrap()])
        .output()
        .expect("failed to execute semantic");
    assert!(output.status.success(), "semantic command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Semantic returns JSON array
    assert!(
        stdout.starts_with('[') && stdout.trim_end().ends_with(']'),
        "expected JSON array output"
    );
}

#[test]
fn test_cli_todo() {
    let bin = cli_bin();
    let temp_root = std::env::temp_dir().join("noa_test_todo");
    let _ = fs::remove_dir_all(&temp_root);
    fs::create_dir_all(&temp_root).unwrap();

    // List (empty)
    let output = Command::new(&bin)
        .args(&[
            "--root",
            temp_root.to_str().unwrap(),
            "todo",
            "--op",
            "list",
        ])
        .output()
        .expect("failed list");
    assert!(output.status.success());

    // Add
    let output = Command::new(&bin)
        .args(&[
            "--root",
            temp_root.to_str().unwrap(),
            "todo",
            "--op",
            "add",
            "--title",
            "Test task",
            "--status",
            "not-started",
            "--details",
            "Testing",
        ])
        .output()
        .expect("failed add");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Test task"), "expected added task");

    // Cleanup
    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn test_cli_archive() {
    let bin = cli_bin();
    let temp_root = std::env::temp_dir().join("noa_test_archive");
    let _ = fs::remove_dir_all(&temp_root);
    fs::create_dir_all(&temp_root).unwrap();
    let test_file = temp_root.join("test.txt");
    fs::write(&test_file, "archive me").unwrap();

    let output = Command::new(&bin)
        .args(&["--root", temp_root.to_str().unwrap(), "archive", "test.txt"])
        .output()
        .expect("failed archive");
    assert!(output.status.success(), "archive command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("archive") && stdout.contains(".tar.zst"),
        "expected archive path"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_root);
}
