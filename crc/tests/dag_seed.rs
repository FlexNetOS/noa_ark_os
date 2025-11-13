use anyhow::Result;
use noa_crc::orchestrator::{JobPlan, JobState, Orchestrator};

#[tokio::test]
async fn dag_seed_produces_checkpoint_artifact() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let checkpoint_dir = temp_dir.path().join("ckpt");

    let mut orchestrator = Orchestrator::new();
    orchestrator.enqueue(JobPlan::simple("seed", &checkpoint_dir));

    let job_record = orchestrator
        .run_next()
        .await?
        .expect("job should have executed");
    assert!(matches!(job_record.state, JobState::Succeeded));

    assert!(checkpoint_dir.exists(), "checkpoint directory created");

    let mut snapshot_found = false;
    for entry in std::fs::read_dir(&checkpoint_dir)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with("snapshot-")
        {
            snapshot_found = true;
            break;
        }
    }

    assert!(snapshot_found, "orchestrator should persist a snapshot");

    Ok(())
}
