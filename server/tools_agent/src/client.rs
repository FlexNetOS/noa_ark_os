use anyhow::{anyhow, Context, Result};
use reqwest::Url;
use serde::de::DeserializeOwned;

use crate::api::{
    EditFileRequest, EditFileResponse, EmptyRequest, FilePatchRequest, FilePatchResponse,
    ListFilesRequest, ListFilesResponse, ReadFileRequest, ReadFileResponse, RunCommandRequest,
    RunCommandResponse, ToolResponse,
};

#[derive(Clone)]
pub struct ToolClient {
    base_url: Url,
    http: reqwest::Client,
}

impl ToolClient {
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let mut base_url = Url::parse(base_url.as_ref())
            .with_context(|| format!("invalid tool server url: {}", base_url.as_ref()))?;
        if !base_url.path().ends_with('/') {
            let mut path = base_url.path().to_string();
            if !path.ends_with('/') {
                path.push('/');
            }
            base_url.set_path(&path);
        }
        Ok(Self {
            base_url,
            http: reqwest::Client::new(),
        })
    }

    pub async fn run_command(&self, request: RunCommandRequest) -> Result<RunCommandResponse> {
        self.post("run_command", &request).await
    }

    pub async fn edit_file(&self, request: EditFileRequest) -> Result<EditFileResponse> {
        self.post("edit_file", &request).await
    }

    pub async fn apply_patch(&self, request: FilePatchRequest) -> Result<FilePatchResponse> {
        self.post("apply_patch", &request).await
    }

    pub async fn list_files(&self, request: ListFilesRequest) -> Result<ListFilesResponse> {
        self.post("list_files", &request).await
    }

    pub async fn read_file(&self, request: ReadFileRequest) -> Result<ReadFileResponse> {
        self.post("read_file", &request).await
    }

    pub async fn run_tests(&self) -> Result<RunCommandResponse> {
        self.post("run_tests", &EmptyRequest {}).await
    }

    pub async fn build_workspace(&self) -> Result<RunCommandResponse> {
        self.post("build_workspace", &EmptyRequest {}).await
    }

    async fn post<T, R>(&self, path: &str, payload: &T) -> Result<R>
    where
        T: serde::Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let url = self.base_url.join(path)?;
        let response = self
            .http
            .post(url)
            .json(payload)
            .send()
            .await
            .context("tool server request failed")?;
        if !response.status().is_success() {
            return Err(anyhow!("tool server returned status {}", response.status()));
        }
        let wrapper: ToolResponse<R> = response
            .json()
            .await
            .context("failed to decode tool server response")?;
        if wrapper.success {
            Ok(wrapper.payload)
        } else {
            Err(anyhow!("tool server reported failure"))
        }
    }
}
