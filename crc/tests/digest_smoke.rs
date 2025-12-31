use std::path::Path;

use anyhow::Result;
use noa_crc::digestors::api::ApiDigestor;
use noa_crc::digestors::bin::BinaryDigestor;
use noa_crc::digestors::config::ConfigDigestor;
use noa_crc::digestors::git::GitDigestor;
use noa_crc::digestors::sbom::SbomDigestor;
use noa_crc::digestors::{AssetRecord, Digestor};
use serde_json::Value;
use tokio::fs;
use walkdir::WalkDir;

fn build_digestors() -> Vec<Box<dyn Digestor>> {
    vec![
        Box::new(GitDigestor::default()),
        Box::new(ConfigDigestor),
        Box::new(ApiDigestor),
        Box::new(SbomDigestor),
        Box::new(BinaryDigestor),
    ]
}

fn seed_repository(root: &Path) -> Result<()> {
    std::fs::create_dir_all(root.join(".git"))?;
    std::fs::write(root.join(".git/HEAD"), "ref: refs/heads/main")?;
    std::fs::write(root.join("config.toml"), "title = \"demo\"")?;

    std::fs::create_dir_all(root.join("apis"))?;
    std::fs::write(
        root.join("apis/service.openapi.json"),
        "{\"openapi\":\"3.0.0\",\"info\":{}}",
    )?;

    std::fs::write(
        root.join("component.sbom.json"),
        "{\"bomFormat\":\"CycloneDX\"}",
    )?;

    std::fs::create_dir_all(root.join("bin"))?;
    std::fs::write(root.join("bin/tool.bin"), [0_u8, 1, 2, 3])?;

    Ok(())
}

fn collect_assets(root: &Path) -> Result<Vec<AssetRecord>> {
    let mut assets = Vec::new();
    for digestor in build_digestors() {
        let mut records = digestor.digest(root)?;
        assets.append(&mut records);
    }
    Ok(assets)
}

#[tokio::test]
async fn digest_smoke_generates_report_with_assets() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();
    seed_repository(root)?;

    let assets = collect_assets(root)?;
    assert!(!assets.is_empty(), "digestors should yield assets");

    let total_files = WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .count();
    assert!(total_files > 0);

    let coverage = if total_files == 0 {
        1.0
    } else {
        (assets.len() as f32 / total_files as f32).min(1.0)
    };

    let report = serde_json::json!({
        "root": root.to_string_lossy(),
        "assets": assets,
        "coverage": coverage,
        "trust_average": assets.iter().map(|a| a.trust as f64).sum::<f64>() / assets.len().max(1) as f64,
    });

    let report_path = temp_dir.path().join("ingest.json");
    fs::write(&report_path, serde_json::to_vec_pretty(&report)?).await?;

    let data = fs::read(&report_path).await?;
    let parsed: Value = serde_json::from_slice(&data)?;
    let parsed_assets = parsed
        .get("assets")
        .and_then(|value| value.as_array())
        .expect("report contains assets array");
    assert!(!parsed_assets.is_empty());

    Ok(())
}
