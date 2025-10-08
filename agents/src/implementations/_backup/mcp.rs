#[cfg(feature = "mcp")]
pub mod tools {
    use anyhow::Result;
    use serde_json::Value;
    use tracing::{info, warn};

    #[derive(Clone, Debug)]
    pub struct Tool {
        pub name: String,
        pub description: String,
        pub parameters: Value,
    }

    impl Tool {
        pub fn new(name: &str, description: &str, parameters: Value) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                parameters,
            }
        }
    }

    pub fn get_available_tools() -> Vec<Tool> {
        vec![
            Tool::new(
                "fs_read",
                "Read a UTF-8 file from disk",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path to read"
                        }
                    },
                    "required": ["path"]
                }),
            ),
            Tool::new(
                "fs_write",
                "Write content to a file",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path to write"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write"
                        }
                    },
                    "required": ["path", "content"]
                }),
            ),
            Tool::new(
                "http_get",
                "Fetch content from a URL",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "URL to fetch"
                        }
                    },
                    "required": ["url"]
                }),
            ),
            Tool::new(
                "http_post",
                "POST data to a URL",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "URL to POST to"
                        },
                        "data": {
                            "type": "object",
                            "description": "JSON data to POST"
                        }
                    },
                    "required": ["url", "data"]
                }),
            ),
        ]
    }

    pub async fn execute_tool(name: &str, args: &Value) -> Result<Value> {
        info!("Executing MCP tool: {} with args: {}", name, args);

        match name {
            "fs_read" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

                // Security check - basic path traversal prevention
                if path.contains("..") || path.starts_with('/') {
                    return Err(anyhow::anyhow!("Invalid path: path traversal not allowed"));
                }

                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", path, e))?;

                Ok(Value::String(content))
            }

            "fs_write" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;

                // Security check
                if path.contains("..") || path.starts_with('/') {
                    return Err(anyhow::anyhow!("Invalid path: path traversal not allowed"));
                }

                // Create parent directories if needed
                if let Some(parent) = std::path::Path::new(path).parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to create directories: {}", e))?;
                }

                tokio::fs::write(path, content)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to write file {}: {}", path, e))?;

                Ok(serde_json::json!({"status": "success", "path": path}))
            }

            "http_get" => {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;

                let client = reqwest::Client::new();
                let response = client
                    .get(url)
                    .timeout(std::time::Duration::from_secs(30))
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("HTTP GET failed: {}", e))?;

                let text = response
                    .text()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

                Ok(Value::String(text))
            }

            "http_post" => {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;

                let data = args
                    .get("data")
                    .ok_or_else(|| anyhow::anyhow!("Missing 'data' parameter"))?;

                let client = reqwest::Client::new();
                let response = client
                    .post(url)
                    .json(data)
                    .timeout(std::time::Duration::from_secs(30))
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("HTTP POST failed: {}", e))?;

                let response_text = response
                    .text()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

                // Try to parse as JSON, fallback to string
                match serde_json::from_str::<Value>(&response_text) {
                    Ok(json) => Ok(json),
                    Err(_) => Ok(Value::String(response_text)),
                }
            }

            _ => {
                warn!("Unknown MCP tool: {}", name);
                Err(anyhow::anyhow!("Unknown tool: {}", name))
            }
        }
    }
}

#[cfg(not(feature = "mcp"))]
pub mod tools {
    use anyhow::Result;
    use serde_json::Value;

    #[derive(Clone, Debug)]
    pub struct Tool {
        pub name: String,
        pub description: String,
        pub parameters: Value,
    }

    pub fn get_available_tools() -> Vec<Tool> {
        vec![]
    }

    pub async fn execute_tool(_name: &str, _args: &Value) -> Result<Value> {
        Err(anyhow::anyhow!("MCP tools not enabled in this build"))
    }
}
