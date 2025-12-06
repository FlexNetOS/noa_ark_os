use chrono::Utc;
use std::process::Command;

fn main() {
    // Preserve existing proto compilation.
    tonic_build::configure()
        .compile(&["proto/ui_schema.proto"], &["proto"])
        .expect("failed to compile ui schema proto");

    // Embed short git commit hash for health/version reporting.
    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=GIT_HASH={}", hash);
    let ts = Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", ts);
}
