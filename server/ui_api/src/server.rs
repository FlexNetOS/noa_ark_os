use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Multipart, Path as AxumPath, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use chrono::Utc;
use flate2::read::GzDecoder;
use noa_crc::cas::Cas;
use noa_crc::engine::Engine;
use noa_crc::graph::{CRCGraph, GraphNode, NodeKind};
use noa_crc::ir::Lane;
use noa_crc::{CRCState, CRCSystem, DropManifest, OriginalArtifact, Priority, SourceType};
use noa_workflow::{StageState, Workflow, WorkflowEngine, WorkflowState};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use tokio::fs;
use tokio::sync::RwLock;
use tokio::task;
use tokio_stream::StreamExt;
use uuid::Uuid;
use zip::ZipArchive;

use crate::schema::{PageEnvelope, ResumeToken};
use crate::session::{default_resume_token, SessionBridge};

pub trait DropRegistry: Send + Sync {
    fn register(
        &self,
        path: PathBuf,
        manifest: DropManifest,
        original_artifact: Option<OriginalArtifact>,
    ) -> Result<String, String>;
    fn get_state(&self, drop_id: &str) -> Option<CRCState>;
}

#[derive(Clone, Default)]
struct CrcDropRegistry {
    crc: CRCSystem,
}

impl DropRegistry for CrcDropRegistry {
    fn register(
        &self,
        path: PathBuf,
        manifest: DropManifest,
        original_artifact: Option<OriginalArtifact>,
    ) -> Result<String, String> {
        self.crc.register_drop(path, manifest, original_artifact)
    }

    fn get_state(&self, drop_id: &str) -> Option<CRCState> {
        self.crc.get_drop(drop_id).map(|drop| drop.state)
    }
}

#[derive(Clone)]
pub struct UiApiState {
    pages: Arc<RwLock<HashMap<String, PageEnvelope>>>,
    session: Arc<Mutex<Option<SessionBridge>>>,
    drop_registry: Arc<dyn DropRegistry>,
    drop_root: PathBuf,
    workflow_engine: Arc<WorkflowEngine>,
}

impl UiApiState {
    pub fn new() -> Self {
        let drop_registry: Arc<dyn DropRegistry> = Arc::new(CrcDropRegistry::default());
        let drop_root = env::var("NOA_UI_DROP_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("D:/dev/workspaces/noa_ark_os/crc/drop-in/incoming"));

        let workflow_engine = Arc::new(WorkflowEngine::new());
        let stream = workflow_engine.enable_streaming(128);

        Self {
            pages: Arc::new(RwLock::new(HashMap::new())),
            session: Arc::new(Mutex::new(Some(SessionBridge::new(stream)))),
            drop_registry,
            drop_root,
            workflow_engine,
        }
    }

    pub fn with_drop_registry<R>(mut self, registry: R) -> Self
    where
        R: DropRegistry + 'static,
    {
        self.drop_registry = Arc::new(registry) as Arc<dyn DropRegistry>;
        self
    }

    pub fn with_drop_root<P>(mut self, drop_root: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.drop_root = drop_root.into();
        self
    }

    pub fn pages(&self) -> &Arc<RwLock<HashMap<String, PageEnvelope>>> {
        &self.pages
    }

    pub fn session(&self) -> &Arc<Mutex<Option<SessionBridge>>> {
        &self.session
    }

    pub fn drop_registry(&self) -> Arc<dyn DropRegistry> {
        Arc::clone(&self.drop_registry)
    }

    pub fn drop_root(&self) -> &PathBuf {
        &self.drop_root
    }

    pub fn workflow_engine(&self) -> Arc<WorkflowEngine> {
        Arc::clone(&self.workflow_engine)
    }
}

impl Default for UiApiState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct UiApiServer {
    state: UiApiState,
}

impl UiApiServer {
    pub fn new() -> Self {
        Self {
            state: UiApiState::new(),
        }
    }

    pub fn with_session(self, bridge: SessionBridge) -> Self {
        if let Ok(mut session) = self.state.session.lock() {
            *session = Some(bridge);
        }
        self
    }

    pub fn with_drop_registry<R>(mut self, registry: R) -> Self
    where
        R: DropRegistry + 'static,
    {
        self.state = self.state.clone().with_drop_registry(registry);
        self
    }

    pub fn with_drop_root<P>(mut self, drop_root: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.state = self.state.clone().with_drop_root(drop_root);
        self
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/ui/pages/:page_id", get(Self::get_page))
            .route("/ui/pages/:page_id/events", get(Self::stream_events))
            .route("/ui/workflows", post(Self::start_workflow))
            .route("/healthz", get(Self::health))
            .route("/readyz", get(Self::ready))
            // Upload → Digest
            .route("/ui/drop-in/upload", post(Self::upload_drop))
            .route("/api/uploads", post(Self::upload_drop))
            .route("/ui/drop-in/panel", get(Self::upload_panel))
            // Capabilities surfacing
            .route("/api/capabilities", get(Self::get_capabilities))
            .with_state(self.state.clone())
    }

    pub fn state(&self) -> UiApiState {
        self.state.clone()
    }

    async fn health() -> impl IntoResponse {
        let payload = json!({
            "status": "ok",
            "timestamp": Utc::now().to_rfc3339(),
        });
        (StatusCode::OK, Json(payload))
    }

    async fn ready(State(state): State<UiApiState>) -> impl IntoResponse {
        let drop_root_exists = fs::try_exists(state.drop_root()).await.unwrap_or(false);
        let session_ready = state
            .session()
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false);

        let ready = drop_root_exists && session_ready;
        let status = if ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };

        let payload = json!({
            "status": if ready { "ready" } else { "initialising" },
            "drop_root": {
                "path": state.drop_root(),
                "exists": drop_root_exists,
            },
            "session_stream": session_ready,
            "timestamp": Utc::now().to_rfc3339(),
        });

        (status, Json(payload))
    }

    async fn upload_panel() -> impl IntoResponse {
        const HTML: &str = r#"<!doctype html><html><head><meta charset=\"utf-8\"><title>Upload → Digest</title></head>
<body style=\"font-family: system-ui; margin:2rem;\">
<h2>Upload → Digest</h2>
<form id=\"f\" method=\"post\" action=\"/ui/drop-in/upload\" enctype=\"multipart/form-data\">
  <label>Type:
    <select name=\"type\">
      <option value=\"repos\">Repo</option>
      <option value=\"forks\">Fork</option>
      <option value=\"mirrors\">Mirror</option>
      <option value=\"stale\">Stale</option>
    </select>
  </label>
  <br/><br/>
  <input type=\"file\" name=\"file\" required />
  <button type=\"submit\">Upload</button>
</form>
<pre id=\"out\"></pre>
<script>
const f=document.getElementById('f');
f.addEventListener('submit', async e=>{
  e.preventDefault();
  const fd=new FormData(f);
  const r=await fetch(f.action,{method:'POST',body:fd});
  const t=await r.text();
  document.getElementById('out').textContent=t;
});
</script>
</body></html>"#;
        ([("content-type", "text/html; charset=utf-8")], HTML)
    }

    async fn get_page(
        State(state): State<UiApiState>,
        AxumPath(page_id): AxumPath<String>,
    ) -> Json<PageEnvelope> {
        let mut pages = state.pages.write().await;
        let envelope = pages
            .entry(page_id.clone())
            .or_insert_with(|| PageEnvelope::with_sample(&page_id))
            .clone();
        Json(envelope)
    }

    async fn get_capabilities() -> Result<Json<JsonValue>, (StatusCode, Json<ErrorResponse>)> {
        let path = std::path::Path::new("registry/capabilities.json");
        match fs::read(path).await {
            Ok(bytes) => {
                let value: Result<JsonValue, _> = serde_json::from_slice(&bytes);
                match value {
                    Ok(v) => Ok(Json(v)),
                    Err(err) => Err(internal_error(format!(
                        "invalid JSON in capabilities.json: {err}"
                    ))),
                }
            }
            Err(_) => Ok(Json(serde_json::json!({ "capabilities": [] }))),
        }
    }

    async fn stream_events(
        ws: WebSocketUpgrade,
        State(state): State<UiApiState>,
        AxumPath(_page_id): AxumPath<String>,
    ) -> impl IntoResponse {
        let bridge = state.session.lock().ok().and_then(|guard| guard.clone());
        if let Some(bridge) = bridge {
            ws.on_upgrade(move |socket| handle_websocket(socket, bridge))
        } else {
            (StatusCode::NOT_FOUND, "workflow streaming disabled").into_response()
        }
    }

    async fn start_workflow(
        State(state): State<UiApiState>,
        Json(request): Json<WorkflowStartRequest>,
    ) -> Result<Json<WorkflowStartResponse>, (StatusCode, Json<ErrorResponse>)> {
        if request.workflow.name.trim().is_empty() {
            return Err(bad_request("workflow name cannot be empty".into()));
        }
        if request.workflow.stages.is_empty() {
            return Err(bad_request("workflow requires at least one stage".into()));
        }

        let workflow = request.workflow;
        let workflow_id = workflow.name.clone();
        let engine = state.workflow_engine();

        engine
            .load_workflow(workflow.clone())
            .map_err(|err| bad_request(format!("failed to load workflow: {err}")))?;

        let first_stage_id = workflow
            .stages
            .first()
            .map(|stage| slugify_stage(&stage.name))
            .unwrap_or_else(|| workflow_id.clone());

        let resume_token = Some(default_resume_token(&workflow_id, &first_stage_id));
        let stages = workflow
            .stages
            .iter()
            .map(|stage| WorkflowStageSnapshot {
                id: slugify_stage(&stage.name),
                name: stage.name.clone(),
                state: StageState::Pending,
            })
            .collect();

        let spawned_engine = engine.clone();
        let execution_id = workflow_id.clone();
        task::spawn(async move {
            let _ = spawned_engine.execute(&execution_id);
        });

        Ok(Json(WorkflowStartResponse {
            workflow_id,
            state: WorkflowState::Pending,
            resume_token,
            stages,
            started_at: Utc::now().to_rfc3339(),
        }))
    }

    async fn upload_drop(
        State(state): State<UiApiState>,
        mut multipart: Multipart,
    ) -> Result<Json<DropUploadResponse>, (StatusCode, Json<ErrorResponse>)> {
        let mut drop_type_value: Option<String> = None;
        let mut file_name: Option<String> = None;
        let mut file_bytes: Option<Bytes> = None;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|err| bad_request(format!("failed to read multipart field: {err}")))?
        {
            match field.name() {
                Some("type") | Some("drop_type") => {
                    let value = field
                        .text()
                        .await
                        .map_err(|err| bad_request(format!("invalid drop type field: {err}")))?;
                    drop_type_value = Some(value);
                }
                Some("file") | Some("upload") => {
                    file_name = field.file_name().map(|name| name.to_string());
                    let bytes = field.bytes().await.map_err(|err| {
                        internal_error(format!("failed to read upload contents: {err}"))
                    })?;
                    file_bytes = Some(bytes);
                }
                _ => {}
            }
        }

        let drop_type_value = drop_type_value
            .ok_or_else(|| bad_request("missing required form field: type".to_string()))?;
        let file_bytes = file_bytes
            .ok_or_else(|| bad_request("missing required form field: file".to_string()))?;

        let (source_type, directory_name) = parse_drop_type(&drop_type_value)
            .ok_or_else(|| bad_request(format!("unsupported drop type: {}", drop_type_value)))?;

        let drop_root = state.drop_root().clone();
        let target_dir = drop_root.join(directory_name);
        fs::create_dir_all(&target_dir)
            .await
            .map_err(|err| internal_error(format!("failed to prepare drop directory: {err}")))?;

        let sanitized_name = sanitize_file_name(file_name.as_deref());
        let file_path = target_dir.join(&sanitized_name);
        let file_bytes = file_bytes.to_vec();
        fs::write(&file_path, &file_bytes)
            .await
            .map_err(|err| internal_error(format!("failed to persist upload: {err}")))?;

        let mut metadata = HashMap::new();
        metadata.insert("original_filename".to_string(), sanitized_name.clone());
        metadata.insert("drop_type".to_string(), drop_type_value.clone());

        let timestamp = Utc::now().timestamp();
        let manifest = DropManifest {
            name: sanitized_name.clone(),
            source: "ui/upload".to_string(),
            source_type,
            timestamp: if timestamp >= 0 { timestamp as u64 } else { 0 },
            priority: Priority::Normal,
            metadata,
        };

        let registry = state.drop_registry();
        let drop_id = registry
            .register(file_path.clone(), manifest, None)
            .map_err(|err| internal_error(format!("failed to register drop: {err}")))?;

        let status = registry
            .get_state(&drop_id)
            .map(|state| format_crc_state(&state))
            .unwrap_or_else(|| format_crc_state(&CRCState::Incoming));

        let processing = prepare_upload_receipt(
            &file_path,
            &sanitized_name,
            &drop_type_value,
            &drop_root,
            &drop_id,
            &file_bytes,
        )
        .await
        .map_err(|err| internal_error(format!("failed to process upload: {err}")))?;

        Ok(Json(DropUploadResponse {
            drop_id,
            status,
            cas_keys: processing.cas_keys,
            receipt_path: processing.receipt_path,
            receipt_url: processing.receipt_url,
        }))
    }
}

impl Default for UiApiServer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
struct DropUploadResponse {
    drop_id: String,
    status: String,
    cas_keys: Vec<String>,
    receipt_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct WorkflowStartRequest {
    workflow: Workflow,
}

#[derive(Serialize, Deserialize)]
struct WorkflowStageSnapshot {
    id: String,
    name: String,
    state: StageState,
}

#[derive(Serialize, Deserialize)]
struct WorkflowStartResponse {
    workflow_id: String,
    state: WorkflowState,
    #[serde(skip_serializing_if = "Option::is_none")]
    resume_token: Option<ResumeToken>,
    stages: Vec<WorkflowStageSnapshot>,
    started_at: String,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    message: String,
}

fn bad_request(message: String) -> (StatusCode, Json<ErrorResponse>) {
    (StatusCode::BAD_REQUEST, Json(ErrorResponse { message }))
}

fn internal_error(message: String) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { message }),
    )
}

fn parse_drop_type(value: &str) -> Option<(SourceType, &'static str)> {
    let normalized = value.trim().to_lowercase();
    match normalized.as_str() {
        "repo" | "repos" | "external_repo" => Some((SourceType::ExternalRepo, "repos")),
        "fork" | "forks" => Some((SourceType::Fork, "forks")),
        "mirror" | "mirrors" => Some((SourceType::Mirror, "mirrors")),
        "stale" | "stale_codebase" => Some((SourceType::StaleCodebase, "stale")),
        "internal" => Some((SourceType::Internal, "internal")),
        _ => None,
    }
}

fn sanitize_file_name(file_name: Option<&str>) -> String {
    file_name
        .and_then(|name| Path::new(name).file_name().and_then(|value| value.to_str()))
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string())
        .unwrap_or_else(|| format!("upload-{}.bin", Uuid::new_v4()))
}

/// Slugifies a stage name for use in URLs or identifiers.
///
/// This implementation **must** stay in sync with the TypeScript `slugifyStage` function.
/// It lowercases, replaces non-alphanumerics with hyphens, and collapses consecutive hyphens.
fn slugify_stage(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn format_crc_state(state: &CRCState) -> String {
    match state {
        CRCState::InSandbox(model) => format!("in_sandbox::{:?}", model).to_lowercase(),
        other => format!("{:?}", other).to_lowercase(),
    }
}

#[derive(Clone, Serialize)]
struct UploadReceiptEntry {
    hash: String,
    path: String,
    size: u64,
    kind: String,
}

#[derive(Serialize)]
struct UploadReceiptDigest {
    checkpoint: String,
    executed_nodes: usize,
}

#[derive(Serialize)]
struct UploadReceiptDocument {
    drop_id: String,
    drop_type: String,
    original_name: String,
    cas_objects: Vec<UploadReceiptEntry>,
    digest: UploadReceiptDigest,
    created_at: String,
}

struct UploadProcessingResult {
    cas_keys: Vec<String>,
    receipt_path: String,
    receipt_url: Option<String>,
}

#[derive(Clone, Copy)]
enum ArchiveKind {
    Zip,
    Tar,
    TarGz,
}

async fn prepare_upload_receipt(
    file_path: &Path,
    original_name: &str,
    drop_type: &str,
    drop_root: &Path,
    drop_id: &str,
    original_bytes: &[u8],
) -> Result<UploadProcessingResult> {
    let cas = Cas::from_env().context("initializing CAS")?;

    let mut cas_objects = Vec::new();

    let original_hash = cas
        .put_bytes(original_bytes)
        .context("storing upload in CAS")?;
    let original_meta = fs::metadata(file_path)
        .await
        .context("reading upload metadata")?;
    cas_objects.push(UploadReceiptEntry {
        hash: original_hash,
        path: relative_path(file_path, drop_root),
        size: original_meta.len(),
        kind: "upload".to_string(),
    });

    if let Some(kind) = detect_archive(original_name) {
        let extracted = extract_archive(file_path, drop_root, drop_id, kind).await?;
        for artifact in extracted {
            let bytes = fs::read(&artifact)
                .await
                .with_context(|| format!("reading extracted artifact {}", artifact.display()))?;
            let hash = cas
                .put_bytes(&bytes)
                .with_context(|| format!("storing extracted artifact {}", artifact.display()))?;
            let meta = fs::metadata(&artifact)
                .await
                .with_context(|| format!("reading metadata for {}", artifact.display()))?;
            cas_objects.push(UploadReceiptEntry {
                hash,
                path: relative_path(&artifact, drop_root),
                size: meta.len(),
                kind: "extracted".to_string(),
            });
        }
    }

    let mut graph = CRCGraph::new();
    graph.add_node(GraphNode::new(
        format!("digest::{}", drop_id),
        NodeKind::Analyze,
        Lane::Fast,
    ));
    let engine = Engine::new(graph);
    let digest_dir = drop_root.join("receipts").join(drop_id).join("digest");
    fs::create_dir_all(&digest_dir)
        .await
        .with_context(|| format!("creating digest directory {}", digest_dir.display()))?;
    let summary = engine
        .run(&digest_dir)
        .await
        .context("running CRC digest engine")?;

    let receipt_dir = drop_root.join("receipts");
    fs::create_dir_all(&receipt_dir)
        .await
        .with_context(|| format!("creating receipt directory {}", receipt_dir.display()))?;
    let receipt_path = receipt_dir.join(format!("{drop_id}.receipt.json"));

    let receipt = UploadReceiptDocument {
        drop_id: drop_id.to_string(),
        drop_type: drop_type.to_string(),
        original_name: original_name.to_string(),
        cas_objects: cas_objects.clone(),
        digest: UploadReceiptDigest {
            checkpoint: summary.checkpoint.to_string_lossy().to_string(),
            executed_nodes: summary.executed.len(),
        },
        created_at: Utc::now().to_rfc3339(),
    };

    let serialized = serde_json::to_vec_pretty(&receipt).context("serializing upload receipt")?;
    fs::write(&receipt_path, serialized)
        .await
        .with_context(|| format!("writing receipt {}", receipt_path.display()))?;

    let cas_keys = cas_objects.iter().map(|entry| entry.hash.clone()).collect();
    let receipt_path_str = receipt_path.to_string_lossy().to_string();
    Ok(UploadProcessingResult {
        cas_keys,
        receipt_path: receipt_path_str.clone(),
        receipt_url: Some(format!("file://{}", receipt_path_str)),
    })
}

fn detect_archive(file_name: &str) -> Option<ArchiveKind> {
    let lowered = file_name.to_lowercase();
    if lowered.ends_with(".tar.gz") || lowered.ends_with(".tgz") {
        Some(ArchiveKind::TarGz)
    } else if lowered.ends_with(".tar") {
        Some(ArchiveKind::Tar)
    } else if lowered.ends_with(".zip") {
        Some(ArchiveKind::Zip)
    } else {
        None
    }
}

async fn extract_archive(
    archive_path: &Path,
    drop_root: &Path,
    drop_id: &str,
    kind: ArchiveKind,
) -> Result<Vec<PathBuf>> {
    let target = drop_root.join("extracted").join(drop_id);
    fs::create_dir_all(&target)
        .await
        .with_context(|| format!("creating extraction directory {}", target.display()))?;
    let archive_path = archive_path.to_path_buf();
    let target_clone = target.clone();
    let extracted = task::spawn_blocking(move || match kind {
        ArchiveKind::Zip => extract_zip(&archive_path, &target_clone),
        ArchiveKind::Tar => extract_tar(&archive_path, &target_clone, false),
        ArchiveKind::TarGz => extract_tar(&archive_path, &target_clone, true),
    })
    .await
    .context("joining archive extraction task")?;

    let extracted = extracted?;
    Ok(extracted)
}

fn extract_zip(archive_path: &Path, target: &Path) -> Result<Vec<PathBuf>> {
    let file = std::fs::File::open(archive_path)
        .with_context(|| format!("opening zip archive {}", archive_path.display()))?;
    let mut archive = ZipArchive::new(file).context("parsing zip archive")?;
    let mut extracted = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).context("reading zip entry")?;
        if entry.is_dir() {
            continue;
        }
        let Some(path) = entry.enclosed_name().map(|path| path.to_owned()) else {
            continue;
        };
        let out_path = target.join(path);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating directory {}", parent.display()))?;
        }
        let mut output = std::fs::File::create(&out_path)
            .with_context(|| format!("creating {}", out_path.display()))?;
        std::io::copy(&mut entry, &mut output)
            .with_context(|| format!("extracting {}", out_path.display()))?;
        extracted.push(out_path);
    }
    Ok(extracted)
}

fn extract_tar(archive_path: &Path, target: &Path, gz: bool) -> Result<Vec<PathBuf>> {
    let file = std::fs::File::open(archive_path)
        .with_context(|| format!("opening tar archive {}", archive_path.display()))?;
    let reader: Box<dyn std::io::Read> = if gz {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };
    let mut archive = tar::Archive::new(reader);
    let mut extracted = Vec::new();
    for entry in archive.entries().context("iterating tar entries")? {
        let mut entry = entry.context("reading tar entry")?;
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let relative = entry.path().context("reading tar entry path")?.into_owned();
        if relative
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            continue;
        }
        let out_path = target.join(&relative);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating directory {}", parent.display()))?;
        }
        entry
            .unpack(&out_path)
            .with_context(|| format!("extracting {}", out_path.display()))?;
        extracted.push(out_path);
    }
    Ok(extracted)
}

fn relative_path(path: &Path, drop_root: &Path) -> String {
    path.strip_prefix(drop_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

async fn handle_websocket(mut socket: WebSocket, bridge: SessionBridge) {
    let mut events = bridge.subscribe();
    while let Some(event) = events.next().await {
        let Ok(event) = event.map(SessionBridge::map_event) else {
            continue;
        };

        match serde_json::to_string(&event) {
            Ok(payload) => {
                if socket.send(Message::Text(payload)).await.is_err() {
                    break;
                }
            }
            Err(error) => {
                let _ = socket
                    .send(Message::Text(format!("{{\"error\":\"{}\"}}", error)))
                    .await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode as HttpStatus};
    use http_body_util::BodyExt;
    use noa_workflow::{Stage, StageType, Task, Workflow};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::sync::Mutex as StdMutex;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct MockRegistry {
        drop_id: String,
        state: Option<CRCState>,
        registered_path: Arc<StdMutex<Option<PathBuf>>>,
        registered_manifest: Arc<StdMutex<Option<DropManifest>>>,
        fail_with: Option<String>,
    }

    impl MockRegistry {
        fn success(drop_id: &str, state: CRCState) -> Self {
            Self {
                drop_id: drop_id.to_string(),
                state: Some(state),
                registered_path: Arc::new(StdMutex::new(None)),
                registered_manifest: Arc::new(StdMutex::new(None)),
                fail_with: None,
            }
        }

        fn failing(message: &str) -> Self {
            Self {
                drop_id: "failing".to_string(),
                state: None,
                registered_path: Arc::new(StdMutex::new(None)),
                registered_manifest: Arc::new(StdMutex::new(None)),
                fail_with: Some(message.to_string()),
            }
        }

        fn recorded_path(&self) -> Option<PathBuf> {
            self.registered_path.lock().unwrap().clone()
        }

        fn recorded_manifest(&self) -> Option<DropManifest> {
            self.registered_manifest.lock().unwrap().clone()
        }
    }

    #[derive(Serialize)]
    struct WorkflowStartRequestPayload<'a> {
        #[serde(borrow)]
        workflow: &'a Workflow,
    }

    impl<'a> WorkflowStartRequestPayload<'a> {
        fn new(workflow: &'a Workflow) -> Self {
            Self { workflow }
        }

        fn into_body(self) -> Vec<u8> {
            serde_json::to_vec(&self).expect("serialize workflow request")
        }
    }

    #[derive(Deserialize)]
    struct WorkflowStartResponsePayload {
        #[serde(flatten)]
        response: WorkflowStartResponse,
    }

    impl WorkflowStartResponsePayload {
        fn from_slice(bytes: &[u8]) -> Self {
            serde_json::from_slice(bytes).expect("parse workflow response")
        }

        fn into_inner(self) -> WorkflowStartResponse {
            self.response
        }
    }

    impl DropRegistry for MockRegistry {
        fn register(
            &self,
            path: PathBuf,
            manifest: DropManifest,
            _original_artifact: Option<OriginalArtifact>,
        ) -> Result<String, String> {
            *self.registered_path.lock().unwrap() = Some(path);
            *self.registered_manifest.lock().unwrap() = Some(manifest.clone());
            if let Some(error) = &self.fail_with {
                return Err(error.clone());
            }
            Ok(self.drop_id.clone())
        }

        fn get_state(&self, _drop_id: &str) -> Option<CRCState> {
            self.state.clone()
        }
    }

    #[derive(Deserialize)]
    struct UploadResponseBody {
        drop_id: String,
        status: String,
        cas_keys: Vec<String>,
        receipt_path: String,
        receipt_url: Option<String>,
    }

    fn multipart_request(boundary: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/ui/drop-in/upload")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn build_multipart_body(boundary: &str, parts: &[(&str, &str, Option<&str>)]) -> String {
        let mut body = String::new();
        for (name, value, filename) in parts {
            body.push_str(&format!("--{boundary}\r\n"));
            if let Some(filename) = filename {
                body.push_str(&format!(
                    "Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n"
                ));
                body.push_str("Content-Type: application/octet-stream\r\n\r\n");
                body.push_str(value);
                body.push_str("\r\n");
            } else {
                body.push_str(&format!(
                    "Content-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
                ));
            }
        }
        body.push_str(&format!("--{boundary}--\r\n"));
        body
    }

    #[tokio::test]
    async fn start_workflow_returns_initial_state() {
        let server = UiApiServer::new();
        let router = server.router();

        let workflow = Workflow {
            name: "plan-test".into(),
            version: "1.0".into(),
            stages: vec![Stage {
                name: "Assess".into(),
                stage_type: StageType::Sequential,
                depends_on: vec![],
                tasks: Vec::<Task>::new(),
            }],
        };

        let request = Request::builder()
            .method("POST")
            .uri("/ui/workflows")
            .header("content-type", "application/json")
            .body(Body::from(
                WorkflowStartRequestPayload::new(&workflow).into_body(),
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), HttpStatus::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let payload = WorkflowStartResponsePayload::from_slice(&bytes).into_inner();

        assert_eq!(payload.workflow_id, "plan-test");
        assert_eq!(payload.state, WorkflowState::Pending);
        assert_eq!(payload.stages.len(), 1);
        assert_eq!(payload.stages[0].id, slugify_stage("Assess"));
        assert!(payload.resume_token.is_some());
    }

    #[tokio::test]
    async fn upload_drop_persists_and_registers() {
        let boundary = "TESTBOUNDARY";
        let temp_dir = tempfile::tempdir().unwrap();
        let drop_root = temp_dir.path().to_path_buf();
        std::env::set_var("CRC_CAS_DIR", drop_root.join("cas"));

        let registry = MockRegistry::success("drop-123", CRCState::Incoming);
        let server = UiApiServer::new()
            .with_drop_root(drop_root.clone())
            .with_drop_registry(registry.clone());
        let router = server.router();

        let body = build_multipart_body(
            boundary,
            &[
                ("type", "repos", None),
                ("file", "hello-world", Some("example.txt")),
            ],
        );
        let request = multipart_request(boundary, &body);

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), HttpStatus::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let parsed: UploadResponseBody = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed.drop_id, "drop-123");
        assert_eq!(parsed.status, "incoming");
        assert!(!parsed.cas_keys.is_empty());
        let expected_url = format!("file://{}", parsed.receipt_path);
        assert_eq!(
            parsed.receipt_url.as_deref(),
            Some(expected_url.as_str()),
            "receipt_url should point to the generated receipt"
        );

        let receipt_path = PathBuf::from(&parsed.receipt_path);
        assert!(receipt_path.exists());
        let receipt: Value =
            serde_json::from_str(&std::fs::read_to_string(&receipt_path).unwrap()).unwrap();
        assert_eq!(
            receipt.get("drop_id").and_then(|v| v.as_str()),
            Some("drop-123")
        );
        assert!(
            receipt
                .get("cas_objects")
                .and_then(|v| v.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0)
                >= 1
        );

        let saved_path = drop_root.join("repos").join("example.txt");
        let saved = fs::read(&saved_path).await.unwrap();
        assert_eq!(saved, b"hello-world");

        let recorded_path = registry.recorded_path().unwrap();
        assert!(recorded_path.ends_with("example.txt"));
        let manifest = registry.recorded_manifest().unwrap();
        assert!(matches!(manifest.source_type, SourceType::ExternalRepo));

        std::env::remove_var("CRC_CAS_DIR");
    }

    #[tokio::test]
    async fn upload_drop_rejects_unknown_type() {
        let boundary = "TESTBOUNDARY";
        let temp_dir = tempfile::tempdir().unwrap();
        let drop_root = temp_dir.path().to_path_buf();

        let registry = MockRegistry::success("drop-ignored", CRCState::Incoming);
        let server = UiApiServer::new()
            .with_drop_root(drop_root)
            .with_drop_registry(registry.clone());
        let router = server.router();

        let body = build_multipart_body(
            boundary,
            &[
                ("type", "unknown", None),
                ("file", "payload", Some("example.txt")),
            ],
        );
        let request = multipart_request(boundary, &body);

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), HttpStatus::BAD_REQUEST);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let error: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
        assert!(error.message.contains("unsupported drop type"));
        assert!(registry.recorded_path().is_none());
    }

    #[tokio::test]
    async fn upload_drop_propagates_registration_error() {
        let boundary = "TESTBOUNDARY";
        let temp_dir = tempfile::tempdir().unwrap();
        let drop_root = temp_dir.path().to_path_buf();

        let registry = MockRegistry::failing("boom");
        let server = UiApiServer::new()
            .with_drop_root(drop_root)
            .with_drop_registry(registry);
        let router = server.router();

        let body = build_multipart_body(
            boundary,
            &[
                ("type", "repos", None),
                ("file", "payload", Some("example.txt")),
            ],
        );
        let request = multipart_request(boundary, &body);

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), HttpStatus::INTERNAL_SERVER_ERROR);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let error: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
        assert!(error.message.contains("failed to register drop"));
    }
}
