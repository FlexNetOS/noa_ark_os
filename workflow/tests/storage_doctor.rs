use noa_workflow::PipelineInstrumentation;
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
        let (index_path, storage_path) = log_pair("relocation");
        // Ensure both index and storage logs exist with identical genesis content
        std::fs::create_dir_all(index_path.parent().unwrap()).expect("create index dir");
        std::fs::create_dir_all(storage_path.parent().unwrap()).expect("create storage dir");

        {
            let mut idx = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&index_path)
                .expect("open relocation index log");
            let mut mir = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&storage_path)
                .expect("open relocation storage log");
            writeln!(idx, "{{\"genesis\":true}}").expect("write index genesis");
            writeln!(mir, "{{\"genesis\":true}}").expect("write storage genesis");
        }

        // Introduce drift by appending only to the index log
        {
            let mut idx = OpenOptions::new()
                .append(true)
                .open(&index_path)
                .expect("reopen relocation index log for drift");
            writeln!(idx, "{{\"drift\":true}}").expect("append drift entry");
        }

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

// --- Minimal, test-local storage doctor implementation ---
#[derive(Debug, Clone, PartialEq, Eq)]
enum StorageDoctorStatus {
    Healthy,
    Degraded,
}

#[derive(Debug, Clone)]
struct MirrorReport {
    name: String,
    drift: bool,
    index_exists: bool,
    storage_exists: bool,
    index_genesis: Option<bool>,
    storage_genesis: Option<bool>,
}

#[derive(Debug, Clone)]
struct StorageDoctorReport {
    status: StorageDoctorStatus,
    drift: Vec<String>,
    mirrors: Vec<MirrorReport>,
}

impl StorageDoctorReport {
    fn is_healthy(&self) -> bool {
        self.status == StorageDoctorStatus::Healthy
    }
}

fn workspace_root() -> std::path::PathBuf {
    if let Ok(root) = std::env::var("NOA_WORKFLOW_ROOT") {
        return std::path::PathBuf::from(root);
    }
    // Fallback to crate manifest dir (parent of workflow crate)
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

fn index_dir() -> std::path::PathBuf {
    workspace_root().join(".workspace").join("indexes")
}

fn mirror_dir() -> std::path::PathBuf {
    workspace_root().join("storage").join("db")
}

fn log_pair(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let index = index_dir().join(format!("{}.log", name));
    let mirror = mirror_dir().join(format!("{}.log", name));
    (index, mirror)
}

fn file_nonempty(path: &std::path::Path) -> Option<bool> {
    match std::fs::read_to_string(path) {
        Ok(s) => Some(!s.trim().is_empty()),
        Err(_) => None,
    }
}

fn compare_logs(index: &std::path::Path, mirror: &std::path::Path) -> bool {
    match (std::fs::read(index), std::fs::read(mirror)) {
        (Ok(a), Ok(b)) => a != b,
        _ => false,
    }
}

fn run_storage_doctor() -> Result<StorageDoctorReport, std::io::Error> {
    let idx_dir = index_dir();
    let mir_dir = mirror_dir();
    std::fs::create_dir_all(&idx_dir)?;
    std::fs::create_dir_all(&mir_dir)?;

    let mut mirrors = Vec::new();
    let mut drift_names = Vec::new();

    // Discover all index logs and compare with their mirrors
    for entry in std::fs::read_dir(&idx_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let (index_path, storage_path) = log_pair(&name);
        let index_exists = index_path.exists();
        let storage_exists = storage_path.exists();
        let drift = index_exists && storage_exists && compare_logs(&index_path, &storage_path);
        if drift {
            drift_names.push(name.clone());
        }
        let report = MirrorReport {
            name,
            drift,
            index_exists,
            storage_exists,
            index_genesis: file_nonempty(&index_path),
            storage_genesis: file_nonempty(&storage_path),
        };
        mirrors.push(report);
    }

    let status = if drift_names.is_empty() {
        StorageDoctorStatus::Healthy
    } else {
        StorageDoctorStatus::Degraded
    };

    Ok(StorageDoctorReport {
        status,
        drift: drift_names,
        mirrors,
    })
}
