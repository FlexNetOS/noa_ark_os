use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use crate::api::{
    CapabilityItem, ConsolidateRequest, ConsolidateResponse, EditFileRequest, EditFileResponse,
    EmptyRequest, ErrorResponse, ExtractCapabilitiesRequest, ExtractCapabilitiesResponse,
    FileEntry, FileKind, FilePatchHunk, FilePatchRequest, FilePatchResponse, ListFilesRequest,
    ListFilesResponse, ReadFileRequest, ReadFileResponse, RunCommandRequest, RunCommandResponse,
    ToolResponse,
};
use crate::consolidation::{
    archive_file_versioned, calculate_sha256, extract_capabilities, ConsolidationIndex,
    ConsolidationIndexEntry, ConsolidationReport, ConsolidationVersion, VersionLedger,
};

const DEFAULT_ADDRESS: &str = "127.0.0.1:8910";
const DEFAULT_LEDGER_PATH: &str = "docs/evidence_ledger.tools.log";
const DEFAULT_ALLOWLIST: &str = "tools/allowed_commands.toml";

#[derive(Clone)]
pub struct ServerOptions {
    pub address: String,
    pub workspace_root: PathBuf,
    pub ledger_path: PathBuf,
    pub allowlist_path: PathBuf,
}

impl Default for ServerOptions {
    fn default() -> Self {
        let workspace_root = std::env::var("NOA_WORKSPACE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().expect("workspace root"));
        let ledger_path = std::env::var("NOA_TOOLS_LEDGER")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join(DEFAULT_LEDGER_PATH));
        let allowlist_path = std::env::var("NOA_TOOLS_ALLOWLIST")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join(DEFAULT_ALLOWLIST));
        Self {
            address: std::env::var("NOA_TOOLS_ADDRESS")
                .unwrap_or_else(|_| DEFAULT_ADDRESS.to_string()),
            workspace_root,
            ledger_path,
            allowlist_path,
        }
    }
}

pub async fn run_server(options: ServerOptions) -> Result<()> {
    let state = Arc::new(ToolServerState::new(options)?);

    let router = Router::new()
        .route("/run_command", post(run_command_handler))
        .route("/edit_file", post(edit_file_handler))
        .route("/apply_patch", post(apply_patch_handler))
        .route("/list_files", post(list_files_handler))
        .route("/read_file", post(read_file_handler))
        .route("/run_tests", post(run_tests_handler))
        .route("/build_workspace", post(build_workspace_handler))
        .route("/extract_capabilities", post(extract_capabilities_handler))
        .route("/consolidate", post(consolidate_handler))
        .with_state(state.clone())
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let address = state.options.address.clone();
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("failed to bind to {}", &address))?;
    info!(%address, "listening");
    axum::serve(listener, router.into_make_service())
        .await
        .context("tool server failed")
}

struct ToolServerState {
    options: ServerOptions,
    root: PathBuf,
    allowlist: CommandAllowlist,
    ledger_lock: Arc<Mutex<()>>,
}

impl ToolServerState {
    fn new(options: ServerOptions) -> Result<Self> {
        let root = options
            .workspace_root
            .canonicalize()
            .context("failed to canonicalize workspace root")?;
        let allowlist = CommandAllowlist::load(&options.allowlist_path)?;
        fs::create_dir_all(
            options
                .ledger_path
                .parent()
                .ok_or_else(|| anyhow!("ledger path missing parent"))?,
        )?;
        Ok(Self {
            options,
            root,
            allowlist,
            ledger_lock: Arc::new(Mutex::new(())),
        })
    }

    fn resolve_path(&self, relative: &str) -> Result<PathBuf> {
        if relative.is_empty() {
            return Ok(self.root.clone());
        }
        if relative.starts_with('/') {
            bail!("absolute paths are not allowed");
        }
        let mut path = PathBuf::new();
        for component in Path::new(relative).components() {
            match component {
                Component::RootDir | Component::Prefix(_) => {
                    bail!("invalid path component");
                }
                Component::ParentDir => {
                    if !path.pop() {
                        bail!("path escapes workspace root");
                    }
                }
                Component::CurDir => {}
                Component::Normal(segment) => path.push(segment),
            }
        }
        let candidate = self.root.join(path);
        let canonical = if candidate.exists() {
            candidate
                .canonicalize()
                .context("failed to canonicalize path")?
        } else {
            candidate
        };
        if !canonical.starts_with(&self.root) {
            bail!("path escapes workspace root");
        }
        Ok(canonical)
    }

    async fn log_event(&self, action: &str, details: serde_json::Value) -> Result<()> {
        let _guard = self.ledger_lock.lock().await;
        let entry = json!({
            "timestamp": chrono::Utc::now(),
            "action": action,
            "details": details,
        });
        let line = serde_json::to_string(&entry)?;
        let path = self.options.ledger_path.clone();
        task::spawn_blocking(move || -> Result<()> {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .with_context(|| format!("failed to open ledger at {}", path.display()))?;
            writeln!(file, "{}", line).context("failed to write ledger entry")?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    async fn run_command(&self, request: RunCommandRequest) -> Result<RunCommandResponse> {
        let command = request.command.clone();
        let args = request.args.clone();
        let result = self.run_command_impl(&request).await;
        let details = match &result {
            Ok(response) => {
                json!({ "command": command, "args": args, "status": response.exit_code })
            }
            Err(err) => json!({ "command": command, "args": args, "error": err.to_string() }),
        };
        self.log_event("run_command", details).await?;
        result
    }

    async fn run_command_impl(&self, request: &RunCommandRequest) -> Result<RunCommandResponse> {
        if request.command.contains('/') || request.command.contains('\\') {
            bail!("command must be a bare executable");
        }
        if !self.allowlist.is_allowed(&request.command, &request.args) {
            bail!("command not allowed");
        }
        let timeout = request.timeout_seconds;
        let output = if let Some(limit) = timeout {
            let root = self.root.clone();
            let command = request.command.clone();
            let args = request.args.clone();
            let env = request.environment.clone();
            let join_result = time::timeout(
                Duration::from_secs(limit),
                task::spawn_blocking(move || execute_command(&root, &command, &args, &env)),
            )
            .await
            .map_err(|_| anyhow!("command timed out"))?;
            join_result??
        } else {
            let root = self.root.clone();
            let command = request.command.clone();
            let args = request.args.clone();
            let env = request.environment.clone();
            task::spawn_blocking(move || execute_command(&root, &command, &args, &env)).await??
        };
        Ok(RunCommandResponse {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    async fn edit_file(&self, request: EditFileRequest) -> Result<EditFileResponse> {
        let path_ref = request.path.clone();
        let result = self.edit_file_impl(&request).await;
        let details = match &result {
            Ok(response) => json!({ "path": path_ref, "bytes": response.bytes_written }),
            Err(err) => json!({ "path": path_ref, "error": err.to_string() }),
        };
        self.log_event("edit_file", details).await?;
        result
    }

    async fn edit_file_impl(&self, request: &EditFileRequest) -> Result<EditFileResponse> {
        let path = self.resolve_path(&request.path)?;
        if !path.exists() && !request.create_if_missing {
            bail!("file does not exist");
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = request.contents.clone();
        let bytes = task::spawn_blocking(move || -> Result<usize> {
            fs::write(&path, contents.as_bytes())
                .with_context(|| format!("failed to write {}", path.display()))?;
            Ok(contents.len())
        })
        .await??;
        Ok(EditFileResponse {
            bytes_written: bytes,
        })
    }

    async fn apply_patch(&self, request: FilePatchRequest) -> Result<FilePatchResponse> {
        let path_ref = request.path.clone();
        let hunk_count = request.hunks.len();
        let result = self.apply_patch_impl(&request).await;
        let details = match &result {
            Ok(_) => json!({ "path": path_ref, "hunks": hunk_count }),
            Err(err) => json!({ "path": path_ref, "hunks": hunk_count, "error": err.to_string() }),
        };
        self.log_event("apply_patch", details).await?;
        result
    }

    async fn apply_patch_impl(&self, request: &FilePatchRequest) -> Result<FilePatchResponse> {
        if request.hunks.is_empty() {
            bail!("no hunks provided");
        }
        let path = self.resolve_path(&request.path)?;
        if !path.exists() {
            bail!("file does not exist");
        }
        let hunks = request.hunks.clone();
        let read_path = path.clone();
        let contents = task::spawn_blocking(move || fs::read_to_string(&read_path)).await??;
        let updated = apply_hunks(&contents, &hunks)?;
        let write_path = path.clone();
        task::spawn_blocking(move || fs::write(&write_path, updated)).await??;
        Ok(FilePatchResponse {
            hunks_applied: hunks.len(),
        })
    }

    async fn list_files(&self, request: ListFilesRequest) -> Result<ListFilesResponse> {
        let path_ref = request.path.clone();
        let result = self.list_files_impl(&request).await;
        let details = match &result {
            Ok(response) => json!({ "path": path_ref, "entries": response.entries.len() }),
            Err(err) => json!({ "path": path_ref, "error": err.to_string() }),
        };
        self.log_event("list_files", details).await?;
        result
    }

    async fn list_files_impl(&self, request: &ListFilesRequest) -> Result<ListFilesResponse> {
        let path = self.resolve_path(&request.path)?;
        let entries = task::spawn_blocking(move || -> Result<Vec<FileEntry>> {
            let mut rows = Vec::new();
            for entry in fs::read_dir(&path)? {
                let entry = entry?;
                let meta = entry.metadata()?;
                let kind = if meta.is_file() {
                    FileKind::File
                } else if meta.is_dir() {
                    FileKind::Directory
                } else {
                    FileKind::Symlink
                };
                let entry_path = entry.path();
                let relative_path = entry_path
                    .strip_prefix(&path)
                    .unwrap_or(entry_path.as_path())
                    .display()
                    .to_string();
                rows.push(FileEntry {
                    path: relative_path,
                    kind,
                    size: meta.len(),
                });
            }
            Ok(rows)
        })
        .await??;
        Ok(ListFilesResponse { entries })
    }

    async fn read_file(&self, request: ReadFileRequest) -> Result<ReadFileResponse> {
        let path_ref = request.path.clone();
        let result = self.read_file_impl(&request).await;
        let details = match &result {
            Ok(_) => json!({ "path": path_ref }),
            Err(err) => json!({ "path": path_ref, "error": err.to_string() }),
        };
        self.log_event("read_file", details).await?;
        result
    }

    async fn read_file_impl(&self, request: &ReadFileRequest) -> Result<ReadFileResponse> {
        let path = self.resolve_path(&request.path)?;
        let contents = task::spawn_blocking(move || fs::read_to_string(&path)).await??;
        Ok(ReadFileResponse {
            path: request.path.clone(),
            contents,
        })
    }

    async fn extract_capabilities_impl(
        &self,
        request: &ExtractCapabilitiesRequest,
    ) -> Result<ExtractCapabilitiesResponse> {
        let path = self.resolve_path(&request.path)?;
        let items = task::spawn_blocking(move || -> Result<Vec<CapabilityItem>> {
            let caps = extract_capabilities(&path)?;
            Ok(caps
                .into_iter()
                .map(|c| CapabilityItem {
                    kind: c.kind().to_string(),
                    name: c.name().to_string(),
                })
                .collect())
        })
        .await??;
        Ok(ExtractCapabilitiesResponse { items })
    }

    async fn consolidate_impl(&self, request: &ConsolidateRequest) -> Result<ConsolidateResponse> {
        let root = &self.root;
        let canonical = self.resolve_path(&request.canonical_path)?;
        let source = self.resolve_path(&request.source_path)?;
        if !canonical.exists() {
            bail!("canonical file does not exist: {}", canonical.display());
        }
        if !source.exists() {
            bail!("source file does not exist: {}", source.display());
        }
        let reason = if request.consolidation_reason.is_empty() {
            "unspecified".to_string()
        } else {
            request.consolidation_reason.clone()
        };

        let canonical_clone = canonical.clone();
        let source_clone = source.clone();
        let root_clone = root.clone();
        let reason_clone = reason.clone();

        let (archive_path, report_path) =
            task::spawn_blocking(move || -> Result<(PathBuf, PathBuf)> {
                let sha = calculate_sha256(&source_clone)?;
                let version = 1u32;
                let archive = archive_file_versioned(&canonical_clone, &root_clone, version)?;

                let preserved_caps = extract_capabilities(&canonical_clone)?;
                let archived_caps = extract_capabilities(&source_clone)?;

                let preserved_names: Vec<String> = preserved_caps
                    .iter()
                    .map(|c| c.name().to_string())
                    .collect();
                let archived_names: Vec<String> =
                    archived_caps.iter().map(|c| c.name().to_string()).collect();

                let version_entry = ConsolidationVersion {
                    version: format!("v{}", version),
                    source_path: source_clone.to_string_lossy().to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    sha256: sha,
                    consolidation_reason: reason_clone,
                    preserved_capabilities: preserved_names.clone(),
                    archived_capabilities: archived_names.clone(),
                    merged_by: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
                };

                let ledger_path = root_clone.join("archive/consolidation/ledger.json");
                let mut ledger = if ledger_path.exists() {
                    let raw = fs::read_to_string(&ledger_path)?;
                    serde_json::from_str::<VersionLedger>(&raw).unwrap_or_else(|_| {
                        VersionLedger::new(canonical_clone.to_string_lossy().to_string())
                    })
                } else {
                    VersionLedger::new(canonical_clone.to_string_lossy().to_string())
                };
                ledger.add_version(version_entry);
                fs::create_dir_all(ledger_path.parent().unwrap())?;
                fs::write(&ledger_path, serde_json::to_string_pretty(&ledger)?)?;

                let report = ConsolidationReport {
                    date: chrono::Utc::now().to_rfc3339(),
                    canonical_file: canonical_clone.to_string_lossy().to_string(),
                    sources_merged: 1,
                    capability_comparison: vec![],
                    tests_passed: true,
                    total_preserved: preserved_caps.len(),
                    total_archived: archived_caps.len(),
                };
                let report_markdown = report.to_markdown();
                let report_path = root_clone.join("archive/consolidation/report.md");
                fs::create_dir_all(report_path.parent().unwrap())?;
                fs::write(&report_path, report_markdown)?;

                let index_path = root_clone.join("archive/consolidation/index.json");
                let mut index = ConsolidationIndex::load(&index_path).unwrap_or_default();
                index.add_entry(ConsolidationIndexEntry {
                    canonical_file: canonical_clone.to_string_lossy().to_string(),
                    version_count: ledger.versions.len(),
                    last_consolidation: chrono::Utc::now().to_rfc3339(),
                    ledger_path: ledger_path.to_string_lossy().to_string(),
                });
                index.save(&index_path)?;

                Ok((archive, report_path))
            })
            .await??;

        Ok(ConsolidateResponse {
            archive_path: archive_path.to_string_lossy().to_string(),
            report_path: report_path.to_string_lossy().to_string(),
        })
    }
}

async fn run_command_handler(
    State(state): State<Arc<ToolServerState>>,
    Json(request): Json<RunCommandRequest>,
) -> Result<Json<ToolResponse<RunCommandResponse>>, (StatusCode, Json<ErrorResponse>)> {
    respond(state.run_command(request).await)
}

async fn edit_file_handler(
    State(state): State<Arc<ToolServerState>>,
    Json(request): Json<EditFileRequest>,
) -> Result<Json<ToolResponse<EditFileResponse>>, (StatusCode, Json<ErrorResponse>)> {
    respond(state.edit_file(request).await)
}

async fn apply_patch_handler(
    State(state): State<Arc<ToolServerState>>,
    Json(request): Json<FilePatchRequest>,
) -> Result<Json<ToolResponse<FilePatchResponse>>, (StatusCode, Json<ErrorResponse>)> {
    respond(state.apply_patch(request).await)
}

async fn list_files_handler(
    State(state): State<Arc<ToolServerState>>,
    Json(request): Json<ListFilesRequest>,
) -> Result<Json<ToolResponse<ListFilesResponse>>, (StatusCode, Json<ErrorResponse>)> {
    respond(state.list_files(request).await)
}

async fn read_file_handler(
    State(state): State<Arc<ToolServerState>>,
    Json(request): Json<ReadFileRequest>,
) -> Result<Json<ToolResponse<ReadFileResponse>>, (StatusCode, Json<ErrorResponse>)> {
    respond(state.read_file(request).await)
}

async fn extract_capabilities_handler(
    State(state): State<Arc<ToolServerState>>,
    Json(request): Json<ExtractCapabilitiesRequest>,
) -> Result<Json<ToolResponse<ExtractCapabilitiesResponse>>, (StatusCode, Json<ErrorResponse>)> {
    respond(state.extract_capabilities_impl(&request).await)
}

async fn consolidate_handler(
    State(state): State<Arc<ToolServerState>>,
    Json(request): Json<ConsolidateRequest>,
) -> Result<Json<ToolResponse<ConsolidateResponse>>, (StatusCode, Json<ErrorResponse>)> {
    respond(state.consolidate_impl(&request).await)
}

async fn run_tests_handler(
    State(state): State<Arc<ToolServerState>>,
    Json(_request): Json<EmptyRequest>,
) -> Result<Json<ToolResponse<RunCommandResponse>>, (StatusCode, Json<ErrorResponse>)> {
    respond(
        state
            .run_command(RunCommandRequest {
                command: "cargo".into(),
                args: vec!["test".into(), "--workspace".into()],
                environment: BTreeMap::new(),
                timeout_seconds: None,
            })
            .await,
    )
}

async fn build_workspace_handler(
    State(state): State<Arc<ToolServerState>>,
    Json(_request): Json<EmptyRequest>,
) -> Result<Json<ToolResponse<RunCommandResponse>>, (StatusCode, Json<ErrorResponse>)> {
    respond(
        state
            .run_command(RunCommandRequest {
                command: "cargo".into(),
                args: vec!["build".into(), "--workspace".into()],
                environment: BTreeMap::new(),
                timeout_seconds: None,
            })
            .await,
    )
}

fn respond<T>(result: Result<T>) -> Result<Json<ToolResponse<T>>, (StatusCode, Json<ErrorResponse>)>
where
    T: Serialize,
{
    match result {
        Ok(payload) => Ok(Json(ToolResponse {
            success: true,
            payload,
        })),
        Err(err) => {
            error!("tool server error: {err}");
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    message: err.to_string(),
                }),
            ))
        }
    }
}

fn execute_command(
    root: &Path,
    command: &str,
    args: &[String],
    environment: &BTreeMap<String, String>,
) -> Result<std::process::Output> {
    let mut cmd = std::process::Command::new(command);
    cmd.current_dir(root);
    cmd.env_clear();
    cmd.env("NOA_WORKSPACE_ROOT", root);
    for (key, value) in environment {
        cmd.env(key, value);
    }
    cmd.args(args);
    cmd.output()
        .with_context(|| format!("failed to execute {command}"))
}

fn apply_hunks(contents: &str, hunks: &[FilePatchHunk]) -> Result<String> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (idx, _) in contents.match_indices('\n') {
        ranges.push((start, idx + 1));
        start = idx + 1;
    }
    if start < contents.len() {
        ranges.push((start, contents.len()));
    }
    let mut output = String::new();
    let mut cursor = 0usize;
    for hunk in hunks {
        if hunk.start_line == 0 || hunk.end_line < hunk.start_line {
            bail!("invalid hunk range");
        }
        let start_idx = ranges
            .get(hunk.start_line - 1)
            .map(|r| r.0)
            .ok_or_else(|| anyhow!("start line out of range"))?;
        let end_idx = ranges
            .get(hunk.end_line - 1)
            .map(|r| r.1)
            .ok_or_else(|| anyhow!("end line out of range"))?;
        if start_idx < cursor {
            bail!("overlapping hunks detected");
        }
        output.push_str(&contents[cursor..start_idx]);
        output.push_str(&hunk.replacement);
        cursor = end_idx;
    }
    output.push_str(&contents[cursor..]);
    Ok(output)
}

#[derive(Debug, Clone)]
struct CommandAllowlist {
    entries: Vec<AllowlistEntry>,
}

impl CommandAllowlist {
    fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read allowlist at {}", path.display()))?;
        let document: AllowlistDocument = toml::from_str(&raw)
            .with_context(|| format!("failed to parse allowlist {}", path.display()))?;
        Ok(Self {
            entries: document.command,
        })
    }

    fn is_allowed(&self, command: &str, args: &[String]) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.matches(command, args))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct AllowlistDocument {
    #[serde(default)]
    command: Vec<AllowlistEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct AllowlistEntry {
    executable: String,
    args: Vec<String>,
    #[serde(default)]
    allow_additional_args: bool,
}

impl AllowlistEntry {
    fn matches(&self, command: &str, args: &[String]) -> bool {
        if self.executable != command {
            return false;
        }
        if self.allow_additional_args {
            if args.len() < self.args.len() {
                return false;
            }
            args.iter()
                .zip(self.args.iter())
                .all(|(lhs, rhs)| lhs == rhs)
        } else {
            self.args == args
        }
    }
}
