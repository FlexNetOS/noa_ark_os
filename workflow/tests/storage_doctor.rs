use noa_workflow::{
    run_storage_doctor, PipelineInstrumentation, PipelineStorageLayout, StorageDoctorStatus,
};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use tempfile::{tempdir, TempDir};

static ENV_GUARD: Mutex<()> = Mutex::new(());

fn with_temp_root<F>(f: F)
where
    F: FnOnce(&TempDir),
{
    let guard = ENV_GUARD
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let temp = tempdir().expect("temporary workflow root");
    std::env::set_var("NOA_WORKFLOW_ROOT", temp.path());
    struct EnvReset<'a> {
        _guard: std::sync::MutexGuard<'a, ()>,
    }

    impl<'a> Drop for EnvReset<'a> {
        fn drop(&mut self) {
            std::env::remove_var("NOA_WORKFLOW_ROOT");
        }
    }

    let cleanup = EnvReset { _guard: guard };
    f(&temp);
    drop(temp);
    drop(cleanup);
}

#[test]
fn storage_doctor_reports_healthy_after_bootstrap() {
    with_temp_root(|_root| {
        PipelineInstrumentation::new().expect("failed to bootstrap instrumentation");
        let report = run_storage_doctor().expect("storage doctor execution");
        assert_eq!(report.status, StorageDoctorStatus::Healthy);
        assert!(report.is_healthy());
        assert!(report.drift.is_empty());
    });
}

#[test]
fn storage_doctor_detects_log_drift() {
    with_temp_root(|_root| {
        PipelineInstrumentation::new().expect("failed to bootstrap instrumentation");
        let layout = PipelineStorageLayout::new();
        let (index_path, storage_path) = layout.log_pair("relocation");
        let mut handle = OpenOptions::new()
            .append(true)
            .open(&index_path)
            .expect("open relocation index log");
        writeln!(handle, "{{\"drift\":true}}").expect("append drift entry");
        drop(handle);

        let report = run_storage_doctor().expect("storage doctor execution");
        let index_contents = std::fs::read_to_string(&index_path).expect("index contents");
        let storage_contents = std::fs::read_to_string(&storage_path).expect("storage contents");
        assert_ne!(index_contents, storage_contents);
        assert_eq!(report.status, StorageDoctorStatus::Degraded);
        assert!(report.drift.iter().any(|name| name == "relocation"));
        let relocation = report
            .mirrors
            .iter()
            .find(|mirror| mirror.name == "relocation")
            .expect("relocation mirror report");
        assert!(relocation.drift);
        assert!(relocation.index_exists);
        assert!(relocation.storage_exists);
        assert_eq!(relocation.index_genesis, Some(true));
        assert_eq!(relocation.storage_genesis, Some(true));
    });
}
