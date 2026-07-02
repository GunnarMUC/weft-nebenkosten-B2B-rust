//! JsonlIngestion Node -- Trigger that watches for JSONL files.
//!
//! Monitors a directory for new .jsonl files (produced by big-pdf-data-chunker).
//! Each file triggers a project execution, streaming chunks via ForEach.

use async_trait::async_trait;
use crate::node::{
    Node, NodeMetadata, NodeFeatures, PortDef, ExecutionContext, FieldDef, TriggerContext,
    TriggerStartConfig, TriggerHandle, TriggerError, WeftType,
};
use crate::{NodeResult, register_node};
use crate::registry::ProjectTrigger;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Default)]
pub struct JsonlIngestionNode;

#[async_trait]
impl Node for JsonlIngestionNode {
    fn node_type(&self) -> &'static str {
        "JsonlIngestion"
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            label: "JSONL-Eingang",
            inputs: vec![],
            outputs: vec![
                PortDef::typed("chunks", WeftType::list(WeftType::json_dict()), false),
                PortDef::new("metadata", "JsonDict", false),
            ],
            features: NodeFeatures {
                isTrigger: true,
                ..Default::default()
            },
            fields: vec![
                FieldDef::text("directory"),
                FieldDef::text("pattern"),
                FieldDef::number("pollInterval"),
            ],
        }
    }

    async fn execute(&self, ctx: ExecutionContext) -> NodeResult {
        let dir = ctx.config_str("directory", "/data/chunks");
        let pattern = ctx.config_str("pattern", "*.jsonl");

        tracing::info!("JsonlIngestion: watching {} for {}", dir, pattern);

        // In the initial execute, return an empty result to pass through
        NodeResult::completed(serde_json::json!({
            "chunks": [],
            "metadata": {
                "directory": dir,
                "pattern": pattern,
                "status": "watching",
            },
        }))
    }

    async fn keep_alive(
        &self,
        config: TriggerStartConfig,
        ctx: TriggerContext,
    ) -> Result<TriggerHandle, TriggerError> {
        let dir = config.config.get("directory")
            .and_then(|v| v.as_str())
            .unwrap_or("/data/chunks")
            .to_string();

        let pattern = config.config.get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("*.jsonl")
            .to_string();

        let interval = config.config.get("pollInterval")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as u64;

        tracing::info!(
            "JsonlIngestion trigger started: dir={}, pattern={}, interval={}s",
            dir, pattern, interval
        );

        let (tx, mut rx) = mpsc::channel::<serde_json::Value>(32);
        let trigger_tx = ctx.trigger_sender.clone();
        let project_id = config.projectId.clone();
        let trigger_id = config.triggerId.clone();

        tokio::spawn(async move {
            let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();

            loop {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        tracing::info!("JsonlIngestion trigger shutting down");
                        break;
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(interval)) => {
                        match scan_directory(&dir, &pattern, &mut processed).await {
                            Ok(chunks) if !chunks.is_empty() => {
                                tracing::info!(
                                    "JsonlIngestion: {} new chunk(s) from {}",
                                    chunks.len(), dir
                                );
                                let _ = trigger_tx.send(serde_json::json!({
                                    "projectId": project_id,
                                    "triggerId": trigger_id,
                                    "chunks": chunks,
                                    "metadata": {
                                        "directory": dir,
                                        "pattern": pattern,
                                    },
                                })).await;
                            }
                            Ok(_) => {} // no new files
                            Err(e) => {
                                tracing::error!("JsonlIngestion scan error: {}", e);
                            }
                        }
                    }
                    _msg = rx.recv() => {
                        // stop signal
                        break;
                    }
                }
            }
        });

        Ok(TriggerHandle {
            stop_sender: tx,
            trigger_id: config.triggerId,
        })
    }
}

async fn scan_directory(
    dir: &str,
    pattern: &str,
    processed: &mut std::collections::HashSet<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let path = PathBuf::from(dir);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let entries = match std::fs::read_dir(&path) {
        Ok(e) => e,
        Err(e) => return Err(format!("Cannot read directory '{}': {}", dir, e)),
    };

    let mut chunks = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let fname = entry.file_name().to_string_lossy().to_string();

        let matches_pattern = match pattern {
            p if p.starts_with("*.") => fname.ends_with(&p[1..]),
            _ => fname == *pattern,
        };

        if !matches_pattern {
            continue;
        }

        let file_path = entry.path();
        if processed.contains(&file_path.to_string_lossy().to_string()) {
            continue;
        }

        match tokio::fs::read_to_string(&file_path).await {
            Ok(content) => {
                processed.insert(file_path.to_string_lossy().to_string());
                let doc_id = format!("{}", uuid::Uuid::new_v4());
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    match serde_json::from_str::<serde_json::Value>(line) {
                        Ok(mut chunk) => {
                            if !chunk.is_object() { continue; }
                            if let Some(obj) = chunk.as_object_mut() {
                                obj.insert("doc_id".to_string(), serde_json::json!(doc_id));
                            }
                            chunks.push(chunk);
                        }
                        Err(_) => continue,
                    }
                }
                tracing::info!(
                    "JsonlIngestion: processed {} -> {} chunks",
                    fname, chunks.len()
                );
            }
            Err(e) => {
                tracing::warn!("Cannot read {}: {}", fname, e);
            }
        }
        break; // Process one file at a time
    }

    Ok(chunks)
}

register_node!(JsonlIngestionNode);
