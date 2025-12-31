use std::env;

use anyhow::{bail, Context, Result};
use noa_tools_agent::{
    api::{
        EditFileRequest, FilePatchHunk, FilePatchRequest, ListFilesRequest, ReadFileRequest,
        RunCommandRequest,
    },
    client::ToolClient,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};

#[derive(Deserialize)]
struct RpcRequest {
    #[serde(default)]
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

#[derive(Deserialize)]
struct CallToolParams {
    name: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

#[derive(Serialize)]
struct McpTool {
    name: &'static str,
    description: &'static str,
    input_schema: serde_json::Value,
}

#[derive(Deserialize)]
struct RunCommandArgs {
    command: String,
    #[serde(default)]
    args: Vec<String>,
}

#[derive(Deserialize)]
struct PathArgs {
    path: String,
}

#[derive(Deserialize)]
struct PatchArgs {
    path: String,
    hunks: Vec<FilePatchHunk>,
}

#[derive(Deserialize)]
struct EditArgs {
    path: String,
    contents: String,
    #[serde(default)]
    create_if_missing: bool,
}

#[derive(Deserialize)]
struct ListArgs {
    #[serde(default = "default_list_path")]
    path: String,
}

fn default_list_path() -> String {
    ".".to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let server =
        env::var("NOA_TOOLS_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8910/".to_string());
    let client = ToolClient::new(server)?;

    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = io::stdout();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let request: RpcRequest = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(err) => {
                let response = RpcResponse {
                    jsonrpc: "2.0",
                    id: None,
                    result: None,
                    error: Some(RpcError {
                        code: -32700,
                        message: format!("invalid request: {err}"),
                    }),
                };
                let encoded = serde_json::to_string(&response)?;
                stdout.write_all(encoded.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
                continue;
            }
        };

        let response = process_request(request, &client).await;
        let encoded = serde_json::to_string(&response)?;
        stdout.write_all(encoded.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }

    Ok(())
}

async fn process_request(request: RpcRequest, client: &ToolClient) -> RpcResponse {
    let result = handle_request(&request, client).await;
    match result {
        Ok(value) => RpcResponse {
            jsonrpc: "2.0",
            id: request.id,
            result: Some(value),
            error: None,
        },
        Err(err) => RpcResponse {
            jsonrpc: "2.0",
            id: request.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: err.to_string(),
            }),
        },
    }
}

async fn handle_request(request: &RpcRequest, client: &ToolClient) -> Result<serde_json::Value> {
    match request.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": "1.0",
            "serverName": "noa-mcp",
            "serverVersion": "0.1.0",
        })),
        "list_tools" => Ok(json!({ "tools": tool_registry() })),
        "call_tool" => {
            let params: CallToolParams = serde_json::from_value(request.params.clone())
                .context("call_tool params must be an object")?;
            execute_tool(params, client).await
        }
        other => bail!("unsupported method {other}"),
    }
}

async fn execute_tool(params: CallToolParams, client: &ToolClient) -> Result<serde_json::Value> {
    match params.name.as_str() {
        "noa.run_command" => {
            let args: RunCommandArgs = serde_json::from_value(params.arguments)
                .context("run_command arguments missing")?;
            let request = RunCommandRequest {
                command: args.command,
                args: args.args,
                environment: Default::default(),
                timeout_seconds: None,
            };
            let response = client.run_command(request).await?;
            Ok(serde_json::to_value(response)?)
        }
        "noa.read_file" => {
            let args: PathArgs =
                serde_json::from_value(params.arguments).context("read_file arguments missing")?;
            let response = client
                .read_file(ReadFileRequest { path: args.path })
                .await?;
            Ok(serde_json::to_value(response)?)
        }
        "noa.apply_patch" => {
            let args: PatchArgs = serde_json::from_value(params.arguments)
                .context("apply_patch arguments missing")?;
            let response = client
                .apply_patch(FilePatchRequest {
                    path: args.path,
                    hunks: args.hunks,
                })
                .await?;
            Ok(serde_json::to_value(response)?)
        }
        "noa.edit_file" => {
            let args: EditArgs =
                serde_json::from_value(params.arguments).context("edit_file arguments missing")?;
            let response = client
                .edit_file(EditFileRequest {
                    path: args.path,
                    contents: args.contents,
                    create_if_missing: args.create_if_missing,
                })
                .await?;
            Ok(serde_json::to_value(response)?)
        }
        "noa.list_files" => {
            let args: ListArgs =
                serde_json::from_value(params.arguments).context("list_files arguments missing")?;
            let response = client
                .list_files(ListFilesRequest { path: args.path })
                .await?;
            Ok(serde_json::to_value(response)?)
        }
        "noa.build" => {
            let response = client.build_workspace().await?;
            Ok(serde_json::to_value(response)?)
        }
        "noa.test" => {
            let response = client.run_tests().await?;
            Ok(serde_json::to_value(response)?)
        }
        other => bail!("unsupported tool {other}"),
    }
}

fn tool_registry() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "noa.run_command",
            description: "Execute allowlisted shell commands (see tools/allowed_commands.toml).",
            input_schema: json!({
                "type": "object",
                "required": ["command"],
                "properties": {
                    "command": { "type": "string" },
                    "args": { "type": "array", "items": { "type": "string" } }
                }
            }),
        },
        McpTool {
            name: "noa.read_file",
            description: "Read a UTF-8 file from the workspace root.",
            input_schema: json!({
                "type": "object",
                "required": ["path"],
                "properties": {"path": {"type": "string"}}
            }),
        },
        McpTool {
            name: "noa.apply_patch",
            description: "Apply structured hunks to a workspace file.",
            input_schema: json!({
                "type": "object",
                "required": ["path", "hunks"],
                "properties": {
                    "path": {"type": "string"},
                    "hunks": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["start_line", "end_line", "replacement"],
                            "properties": {
                                "start_line": {"type": "integer"},
                                "end_line": {"type": "integer"},
                                "replacement": {"type": "string"}
                            }
                        }
                    }
                }
            }),
        },
        McpTool {
            name: "noa.edit_file",
            description: "Replace the full contents of a workspace file.",
            input_schema: json!({
                "type": "object",
                "required": ["path", "contents"],
                "properties": {
                    "path": {"type": "string"},
                    "contents": {"type": "string"},
                    "create_if_missing": {"type": "boolean"}
                }
            }),
        },
        McpTool {
            name: "noa.list_files",
            description: "List directory entries within the workspace.",
            input_schema: json!({
                "type": "object",
                "properties": {"path": {"type": "string"}}
            }),
        },
        McpTool {
            name: "noa.build",
            description: "Run cargo build --workspace via the allowlisted runner.",
            input_schema: json!({"type": "object"}),
        },
        McpTool {
            name: "noa.test",
            description: "Run cargo test --workspace via the allowlisted runner.",
            input_schema: json!({"type": "object"}),
        },
    ]
}
