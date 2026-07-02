//! VllmInference Node -- Local LLM inference via vLLM OpenAI-compatible API.
//!
//! On-premises replacement for the cloud-only LlmInference node.
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
        let vllm_config = ctx.input.get("config")
            .and_then(|v| v.as_object())
            .map(|o| serde_json::Value::Object(o.clone()));

        let config_source = vllm_config.as_ref().unwrap_or(&ctx.config);

        let model = config_source.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("qwen2.5:32b");

        let base_url = config_source.get("baseUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost:8000");

        let system_prompt = ctx.input.get("systemPrompt")
            .and_then(|v| v.as_str())
            .or_else(|| config_source.get("systemPrompt").and_then(|v| v.as_str()))
            .unwrap_or("");

        let prompt = ctx.input.get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let temperature = config_source.get("temperature")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);

        let max_tokens = config_source.get("maxTokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(4096);

        let top_p = config_source.get("topP")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.9);

        let parse_json = config_source.get("parseJson")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

        let mut messages: Vec<serde_json::Value> = Vec::new();
        if !system_prompt.is_empty() {
            messages.push(serde_json::json!({"role": "system", "content": system_prompt}));
        }
        messages.push(serde_json::json!({"role": "user", "content": prompt}));

        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "top_p": top_p,
        });

        tracing::info!(
            "vLLM request: model={}, prompt_len={}, url={}",
            model, prompt.len(), url
        );

        match ctx.http_client.post(&url).json(&body).send().await {
            Ok(response) if response.status().is_success() => {
                let data: serde_json::Value = response.json().await
                    .unwrap_or(serde_json::json!({"error": "Failed to parse vLLM response"}));

                let text = data["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                let response_value = if parse_json {
                    let cleaned = text.trim();
                    serde_json::from_str(cleaned)
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
                let body = response.text().await.unwrap_or_default();
                NodeResult::failed(&format!("vLLM error: {}", body))
            }
            Err(e) => {
                NodeResult::failed(&format!("vLLM connection error: {}", e))
            }
        }
    }
}

register_node!(VllmInferenceNode);
