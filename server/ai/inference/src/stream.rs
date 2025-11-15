use std::pin::Pin;

use anyhow::{Context, Result};
use async_stream::try_stream;
use futures::{Stream, StreamExt};
use reqwest::Response;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionChunk {
    pub content: String,
    pub is_final: bool,
    pub tokens_evaluated: Option<usize>,
    pub tokens_predicted: Option<usize>,
}

pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<CompletionChunk>> + Send>>;

pub fn parse_sse_stream(
    response: Response,
    parser: fn(&str) -> Option<CompletionChunk>,
) -> CompletionStream {
    Box::pin(try_stream! {
        let mut buffer = String::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read streaming chunk")?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            loop {
                let delimiter = buffer.find("\n\n");
                let Some(index) = delimiter else { break; };
                let event = buffer[..index].to_string();
                buffer.drain(..index + 2);
                if event.trim().is_empty() {
                    continue;
                }

                for line in event.lines() {
                    if let Some(data) = line.strip_prefix("data:") {
                        let payload = data.trim();
                        if payload == "[DONE]" {
                            yield CompletionChunk {
                                content: String::new(),
                                is_final: true,
                                tokens_evaluated: None,
                                tokens_predicted: None,
                            };
                            return;
                        }

                        if let Some(chunk) = parser(payload) {
                            let is_final = chunk.is_final;
                            yield chunk;
                            if is_final {
                                return;
                            }
                        }
                    }
                }
            }
        }
    })
}

pub fn parse_json_lines_stream(response: Response) -> CompletionStream {
    Box::pin(try_stream! {
        let mut buffer = String::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read streaming chunk")?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            loop {
                let Some(index) = buffer.find('\n') else { break; };
                let line = buffer[..index].to_string();
                buffer.drain(..index + 1);
                if line.trim().is_empty() {
                    continue;
                }

                let chunk: serde_json::Value = serde_json::from_str(&line)
                    .context("Failed to parse streaming JSON chunk")?;
                let content = chunk
                    .get("content")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
                let is_final = chunk
                    .get("done")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
                let tokens_evaluated = chunk
                    .get("tokens_evaluated")
                    .and_then(|value| value.as_u64())
                    .map(|value| value as usize);
                let tokens_predicted = chunk
                    .get("tokens_predicted")
                    .and_then(|value| value.as_u64())
                    .map(|value| value as usize);

                yield CompletionChunk {
                    content,
                    is_final,
                    tokens_evaluated,
                    tokens_predicted,
                };

                if is_final {
                    return;
                }
            }
        }

        if !buffer.trim().is_empty() {
            let chunk: serde_json::Value = serde_json::from_str(buffer.trim())
                .context("Failed to parse trailing streaming JSON chunk")?;
            let content = chunk
                .get("content")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let is_final = chunk
                .get("done")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            let tokens_evaluated = chunk
                .get("tokens_evaluated")
                .and_then(|value| value.as_u64())
                .map(|value| value as usize);
            let tokens_predicted = chunk
                .get("tokens_predicted")
                .and_then(|value| value.as_u64())
                .map(|value| value as usize);

            yield CompletionChunk {
                content,
                is_final,
                tokens_evaluated,
                tokens_predicted,
            };
        }
    })
}
