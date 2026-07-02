//! VllmInference Node -- Local LLM inference via vLLM OpenAI-compatible API.
//!
//! This node replaces the cloud-only LlmInference node for on-premises deployments.
//! Communicates via HTTP POST to a vLLM server (/v1/chat/completions).

use async_trait::async_trait;
use crate::node::{Node, NodeMetadata, NodeFeatures, PortDef, ExecutionContext, FieldDef};
use crate::{NodeResult, register_node};

#[derive(Default)]
pub struct VllmInferenceNode;

#[async_trait]
impl Node for VllmInferenceNode {
    fn node_type(&self) -> &'static str {
        "VllmInference"
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            label: "vLLM",
            inputs: vec![
                PortDef::new("prompt", "String", true),
                PortDef::new("systemPrompt", "String", false),
                PortDef::new("config", "Dict[String, String | Number | Boolean]", false),
            ],
            outputs: vec![
                PortDef::new("response", "String", false),
            ],
            features: NodeFeatures {
                canAddOutputPorts: true,
                ..Default::default()
            },
            fields: vec![
                FieldDef::text("model"),
                FieldDef::text("baseUrl"),
                FieldDef::textarea("systemPrompt"),
                FieldDef::number("maxTokens"),
                FieldDef::number("temperature"),
                FieldDef::number("topP"),
                FieldDef::checkbox("parseJson"),
            ],
        }
    }

    async fn execute(&self, ctx: ExecutionContext) -> NodeResult {
        use serde_json::json;

        let model = ctx.config.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("qwen2.5:32b");

        let base_url = ctx.config.get("baseUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost:8000");

        let system_prompt = ctx.input.get("systemPrompt")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let prompt = ctx.input.get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let temperature = ctx.config.get("temperature")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);

        let max_tokens = ctx.config.get("maxTokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(4096);

        let parse_json = ctx.config.get("parseJson")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let client = reqwest::Client::new();
        let url = format!("{}/v1/chat/completions", base_url);

        let mut messages = Vec::new();
        if !system_prompt.is_empty() {
            messages.push(json!({"role": "system", "content": system_prompt}));
        }
        messages.push(json!({"role": "user", "content": prompt}));

        let body = json!({
            "model": model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens,
        });

        tracing::info!("vLLM request: model={}, prompt_len={}, url={}", model, prompt.len(), url);

        match client.post(&url).json(&body).send().await {
            Ok(response) if response.status().is_success() => {
                let data: serde_json::Value = response.json().await
                    .unwrap_or(json!({"error": "Failed to parse vLLM response"}));

                let text = data["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                let response_value = if parse_json {
                    serde_json::from_str(&text)
                        .unwrap_or_else(|_| serde_json::Value::String(text.clone()))
                } else {
                    serde_json::Value::String(text)
                };

                let mut output = serde_json::Map::new();
                output.insert("response".to_string(), response_value.clone());

                if parse_json {
                    if let serde_json::Value::Object(obj) = &response_value {
                        for (key, val) in obj {
                            if key != "response" {
                                output.insert(key.clone(), val.clone());
                            }
                        }
                    }
                }

                NodeResult::completed(serde_json::Value::Object(output))
            }
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                let msg = format!("vLLM error ({}): {}", status, body);
                tracing::error!("{}", msg);
                NodeResult::failed(&msg)
            }
            Err(e) => {
                let msg = format!("vLLM connection error: {}", e);
                tracing::error!("{}", msg);
                NodeResult::failed(&msg)
            }
        }
    }
}

register_node!(VllmInferenceNode);
