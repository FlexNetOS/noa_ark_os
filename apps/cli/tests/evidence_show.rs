use std::collections::HashMap;
use std::sync::{Mutex, Once};
use std::thread;
use std::time::Duration;

use noa_workflow::{PipelineInstrumentation, SecurityScanStatus, Stage, StageType, Task};
use predicates::prelude::*;
use serde_json::json;
use tempfile::TempDir;

static INIT_SECRET: Once = Once::new();
static LEDGER_LOCK: Mutex<()> = Mutex::new(());

fn ensure_policy_secret() {
    INIT_SECRET.call_once(|| {
        std::env::set_var("NOA_POLICY_SECRET", "test-policy-secret");
    });
}

fn sample_stage() -> Stage {
    Stage {
        name: "build".to_string(),
        stage_type: StageType::Sequential,
        depends_on: Vec::new(),
        tasks: vec![Task {
            agent: "noop-agent".to_string(),
            action: "noop".to_string(),
            parameters: HashMap::new(),
            agent_role: None,
            tool_requirements: Vec::new(),
        }],
    }
}

fn initialise_ledger() -> anyhow::Result<(
    TempDir,
    PipelineInstrumentation,
    std::sync::MutexGuard<'static, ()>,
)> {
    ensure_policy_secret();
    let guard = LEDGER_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let workspace = TempDir::new()?;
    std::env::set_var("NOA_WORKFLOW_ROOT", workspace.path());
    let instrumentation = PipelineInstrumentation::new()?;
    Ok((workspace, instrumentation, guard))
}

#[test]
fn evidence_show_filters_and_verifies_signatures() -> anyhow::Result<()> {
    let (workspace, instrumentation, _guard) = initialise_ledger()?;
    let stage = sample_stage();
    instrumentation.log_stage_receipt("wf-alpha", &stage, &[json!({ "artifact": "alpha" })])?;
    thread::sleep(Duration::from_millis(5));
    instrumentation.log_security_scan(
        "wf-alpha",
        "scanner",
        SecurityScanStatus::Passed,
        Vec::new(),
        None,
        json!({ "notes": "clean" }),
    )?;

    let ledger_path = workspace.path().join("storage/db/evidence/ledger.jsonl");
    assert!(ledger_path.exists(), "ledger missing at {:?}", ledger_path);

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("noa");
    cmd.current_dir(workspace.path())
        .env("NOA_WORKFLOW_ROOT", workspace.path())
        .env("NOA_POLICY_SECRET", "test-policy-secret")
        .args([
            "evidence",
            "--workflow",
            "wf-alpha",
            "--kind",
            "stage-receipt",
            "--verify-signatures",
        ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("kind=StageReceipt"))
        .stdout(predicate::str::contains("signature=verified"))
        .stdout(predicate::str::contains("\"workflow_id\": \"wf-alpha\""))
        .stdout(predicate::str::contains("SecurityScan").not());

    Ok(())
}

#[test]
fn evidence_show_supports_timestamp_filters() -> anyhow::Result<()> {
    let (workspace, instrumentation, _guard) = initialise_ledger()?;
    let stage = sample_stage();
    let receipt =
        instrumentation.log_stage_receipt("wf-beta", &stage, &[json!({ "artifact": "beta" })])?;
    thread::sleep(Duration::from_millis(5));
    instrumentation.log_security_scan(
        "wf-beta",
        "scanner",
        SecurityScanStatus::Passed,
        Vec::new(),
        None,
        json!({}),
    )?;

    let ledger_path = workspace.path().join("storage/db/evidence/ledger.jsonl");
    assert!(ledger_path.exists(), "ledger missing at {:?}", ledger_path);

    let since = (receipt.generated_at + 1).to_string();

    let mut cmd_since = assert_cmd::cargo::cargo_bin_cmd!("noa");
    cmd_since
        .current_dir(workspace.path())
        .env("NOA_WORKFLOW_ROOT", workspace.path())
        .env("NOA_POLICY_SECRET", "test-policy-secret")
        .args([
            "evidence",
            "--kind",
            "security-scan",
            "--since",
            &since,
            "--verify-signatures",
        ]);

    cmd_since
        .assert()
        .success()
        .stdout(predicate::str::contains("kind=SecurityScan"))
        .stdout(predicate::str::contains("signature=verified"))
        .stdout(predicate::str::contains("StageReceipt").not());

    let until = receipt.generated_at.to_string();
    let mut cmd_until = assert_cmd::cargo::cargo_bin_cmd!("noa");
    cmd_until
        .current_dir(workspace.path())
        .env("NOA_WORKFLOW_ROOT", workspace.path())
        .env("NOA_POLICY_SECRET", "test-policy-secret")
        .args([
            "evidence",
            "--workflow",
            "wf-beta",
            "--kind",
            "stage-receipt",
            "--until",
            &until,
        ]);

    cmd_until
        .assert()
        .success()
        .stdout(predicate::str::contains("kind=StageReceipt"))
        .stdout(predicate::str::contains("SecurityScan").not());

    Ok(())
}
