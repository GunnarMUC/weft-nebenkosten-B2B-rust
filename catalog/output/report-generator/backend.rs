//! ReportGenerator Node -- Generates structured audit reports from analysis results.
//!
//! Takes analysis results and compliance findings, produces a structured
//! JSON report (machine-readable) and a human-readable Markdown summary.
//! PDF generation can be added later via a sidecar.

use async_trait::async_trait;
use crate::node::{Node, NodeMetadata, NodeFeatures, PortDef, ExecutionContext, FieldDef};
use crate::{NodeResult, register_node};

#[derive(Default)]
pub struct ReportGeneratorNode;

#[async_trait]
impl Node for ReportGeneratorNode {
    fn node_type(&self) -> &'static str {
        "ReportGenerator"
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            label: "Prufbericht",
            inputs: vec![
                PortDef::new("analysisResult", "JsonDict", true),
                PortDef::new("findings", "List[JsonDict]", true),
                PortDef::new("metadata", "JsonDict", false),
            ],
            outputs: vec![
                PortDef::new("reportJson", "JsonDict", false),
                PortDef::new("reportMd", "String", false),
                PortDef::new("severity", "Number", false),
            ],
            features: NodeFeatures { ..Default::default() },
            fields: vec![
                FieldDef::select("format", vec!["json", "md", "both"]),
                FieldDef::select("template", vec!["gewerbe", "standard", "wohnraum"]),
            ],
        }
    }

    async fn execute(&self, ctx: ExecutionContext) -> NodeResult {
        let analysis = ctx.input.get("analysisResult").cloned()
            .unwrap_or(serde_json::json!({}));

        let findings: Vec<serde_json::Value> = ctx.input.get("findings")
            .and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let metadata = ctx.input.get("metadata").cloned()
            .unwrap_or(serde_json::json!({}));

        let template = ctx.config_str("template", "gewerbe");
        let format = ctx.config_str("format", "both");

        let severity = findings.iter()
            .filter_map(|f| f.get("severity").and_then(|s| s.as_u64()))
            .max().unwrap_or(0);

        let finding_count = findings.len();
        let critical = findings.iter()
            .filter(|f| f.get("severity").and_then(|s| s.as_u64()).unwrap_or(0) >= 4)
            .count();

        let report_json = serde_json::json!({
            "reportType": "nkcheck-industrie",
            "template": template,
            "generatedAt": chrono::Utc::now().to_rfc3339(),
            "severity": severity,
            "summary": {
                "totalFindings": finding_count,
                "criticalFindings": critical,
            },
            "analysis": analysis,
            "findings": findings,
            "metadata": metadata,
            "recommendations": build_recommendations(&findings),
        });

        let report_md = build_markdown(&report_json, &findings, template);

        NodeResult::completed(serde_json::json!({
            "reportJson": report_json,
            "reportMd": report_md,
            "severity": severity,
        }))
    }
}

fn build_recommendations(findings: &[serde_json::Value]) -> Vec<String> {
    let mut recs: Vec<String> = Vec::new();
    for f in findings {
        if let Some(rec) = f.get("recommendation").and_then(|v| v.as_str()) {
            recs.push(rec.to_string());
        }
    }
    recs.sort();
    recs.dedup();
    recs
}

fn severity_label(s: u64) -> &'static str {
    match s {
        5 => "Kritisch",
        4 => "Sehr hoch",
        3 => "Hoch",
        2 => "Mittel",
        _ => "Niedrig",
    }
}

fn build_markdown(report: &serde_json::Value, findings: &[serde_json::Value], template: &str) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Prufbericht -- NK-Check Industrie\n\n"));

    let severity = report["severity"].as_u64().unwrap_or(0);
    let total = report["summary"]["totalFindings"].as_u64().unwrap_or(0);
    let critical = report["summary"]["criticalFindings"].as_u64().unwrap_or(0);

    md.push_str(&format!("**Template:** {}\n", template));
    md.push_str(&format!("**Erstellt:** {}\n", report["generatedAt"].as_str().unwrap_or("")));
    md.push_str(&format!("**Gesamtschweregrad:** {} ({})\n\n", severity, severity_label(severity)));

    md.push_str("---\n\n## Zusammenfassung\n\n");
    md.push_str(&format!("- **{}** Findings insgesamt\n", total));
    md.push_str(&format!("- **{}** kritische Findings\n\n", critical));

    if !findings.is_empty() {
        md.push_str("## Findings\n\n");
        for f in findings {
            let sev = f.get("severity").and_then(|s| s.as_u64()).unwrap_or(0);
            let desc = f.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let law = f.get("legalRef").and_then(|v| v.as_str()).unwrap_or("");
            let rec = f.get("recommendation").and_then(|v| v.as_str()).unwrap_or("");

            md.push_str(&format!("### {} ({})\n\n", desc, severity_label(sev)));
            md.push_str(&format!("- **Schweregrad:** {}/5\n", sev));
            if !law.is_empty() {
                md.push_str(&format!("- **Normbezug:** {}\n", law));
            }
            if let Some(pos) = f.get("affectedPosition").and_then(|v| v.as_str()) {
                md.push_str(&format!("- **Position:** {}\n", pos));
            }
            if !rec.is_empty() {
                md.push_str(&format!("- **Empfehlung:** {}\n", rec));
            }
            md.push_str("\n");
        }
    }

    md.push_str("---\n\n");
    md.push_str("*Dieser Bericht ersetzt keine Rechtsberatung (§ 1 RDG).*\n");

    md
}

register_node!(ReportGeneratorNode);
